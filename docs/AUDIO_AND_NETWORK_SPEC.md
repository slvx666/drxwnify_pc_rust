# RUST REWRITE — AUDIO & NETWORK ARCHITECTURE SPECIFICATION (AUDIO_AND_NETWORK_SPEC.md)

**Целевой модуль аудио:** `drxwnify-audio` (FFmpeg FFI + CPAL WASAPI + RustFFT)  
**Целевой модуль сети:** `drxwnify-resolver`, `drxwnify-sources`, `drxwnify-nostr` (Tokio + Reqwest + Tungstenite)  
**Статус:** Полная техническая спецификация для реализации на Rust

---

## 1. Архитектура аудиосистемы (Audio Engine)

Аудиосистема Drxwnify отвечает за непрерывное потоковое воспроизведение удаленных и локальных треков с аппаратным выводом через Windows WASAPI, динамической обработкой звука (нормализация, срез тишины, баланс, громкость) и спектральным анализом в реальном времени.

### 1.1. Схема аудиопайплайна в Rust

```
┌────────────────────────────────────────────────────────────────────────┐
│                        ВХОДНОЙ АУДИОПОТОК                              │
│  - Remote HTTP/HTTPS Stream (YouTube WebM/Opus, AAC MP4)               │
│  - Remote HLS Playlist .m3u8 с AES-128 шифрованием (VK Music)          │
│  - Remote FLAC Stream (Lossless Mirrors) / MP3 (SoundCloud, Bandcamp)  │
│  - Локальный кэшированный файл (%APPDATA%\Drxwnify\cache\audio)        │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼────────────────────────────────────┐
│              ДЕКОДЕР И ДЕМУКСЕР (drxwnify-audio::decoder)              │
│  - libavformat (avformat_open_input, av_read_frame)                    │
│    Параметры: reconnect=1, reconnect_streamed=1, rw_timeout=8000000    │
│  - libavcodec (avcodec_send_packet, avcodec_receive_frame)             │
│  - libswresample (SwrContext): ресемплинг в 48000 Гц, S16, Stereo      │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Непрерывные чанки 48 kHz S16 PCM
┌───────────────────────────────────▼────────────────────────────────────┐
│            ЦИФРОВАЯ ОБРАБОТКА СИГНАЛА (drxwnify-audio::dsp)            │
│  1. Skip Silence Gate: отсечение тишины в начале (<20с, peak < 300)    │
│  2. Dynamic RMS Normalizer: скользящий RMS, гейн в диапазоне 0.5–2.2   │
│  3. Crossfade & Fade Out: линейное затухание (таймер сна / смена)      │
│  4. Volume Curve: gain = (volume)^1.7 * fade * effNormGain             │
│  5. Stereo Balance: раздельное масштабирование gainL / gainR           │
└─────────────────┬──────────────────────────────────┬───────────────────┘
                  │                                  │
┌─────────────────▼──────────────────┐ ┌─────────────▼──────────────────┐
│   КОЛЬЦЕВОЙ БУФЕР FFT (1024 сэмпла)│ │  ВЫВОД ЗВУКА (drxwnify-audio)  │
│  - Окно Ханна (Hann Window)         │ │  - CPAL WASAPI Shared Mode     │
│  - 1024-point RustFFT              │ │  - Буфер: 48000 * 2ch * 2 bytes│
│  - 32 логарифмические полосы       │ │    (размер ~200 мс)            │
│  - Фильтр спада: 0.82*prev + 0.18*v│ │  - Вывод в активное аудио-     │
│  - Эмит события уровней раз в 50 мс│ │    устройство Windows          │
└────────────────────────────────────┘ └────────────────────────────────┘
```

### 1.2. Декодирование и управление форматами
В оригинале используется `org.bytedeco:ffmpeg:8.1.2` (`FFmpegFrameGrabber`).
В Rust-версии используется **`ffmpeg-next`** (или прямой FFI биндинг к `libavcodec`, `libavformat`, `libswresample` минимальной LGPL сборки без лицензионно проблемных видеокодеков):

1. **Опции открытия стрима:**
   ```rust
   let mut options = ffmpeg::Dictionary::new();
   options.set("reconnect", "1");
   options.set("reconnect_streamed", "1");
   options.set("reconnect_on_network_error", "1");
   options.set("reconnect_delay_max", "2");
   options.set("rw_timeout", "8000000"); // 8 секунд сокетного тайм-аута
   options.set("user_agent", user_agent);
   if !headers.is_empty() {
       options.set("headers", headers_string);
   }
   ```
2. **HLS и AES-128 (VK Music):**
   При открытии URL вида `https://.../index.m3u8` FFmpeg автоматически обрабатывает теги `#EXT-X-KEY:METHOD=AES-128,URI="..."`. Сетевой стек FFmpeg запрашивает ключ шифрования и дешифрует криптографические блоки на лету.
3. **Ресемплинг:** Все форматы приводятся к единому стандарту аудиовывода:
   - Частота дискретизации: **48 000 Гц** (`rate = 48000`)
   - Формат сэмплов: **16-bit signed integer (S16LE)**
   - Каналы: **2 (Stereo Interleaved)**

### 1.3. Алгоритмы цифровой обработки (DSP)

#### 1. Кривая громкости ($x^{1.7}$)
В `AudioEngine.kt:404` используется нелинейная перцептивная кривая:
$$\text{gain} = \text{volume}^{1.7} \times \text{fade} \times \text{effNormGain} \times \text{crossfadeFade}$$
где `volume` $\in [0.0, 1.0]$. При значении `volume = 0` звук полностью отключается.

#### 2. Стереобаланс (Stereo Balance)
Параметр `balance` $\in [-1.0, 1.0]$:
- $\text{gain}_L = \text{gain} \times (1.0 - \max(0.0, \text{balance}))$
- $\text{gain}_R = \text{gain} \times (1.0 + \min(0.0, \text{balance}))$
При балансе `-1.0` левый канал звучит на 100%, правый заглушен на 0%. При балансе `+1.0` правый канал звучит на 100%, левый заглушен.

#### 3. Динамическая нормализация (Dynamic RMS Normalizer)
В `AudioEngine.kt:382-393`:
1. Для каждого аудиофрейма из $N$ сэмплов вычисляется среднеквадратичное значение:
   $$\text{sumSq} = \sum (s_1^2 + s_2^2)$$
   $$\text{frameRms} = \frac{\sqrt{\text{sumSq} / (2N)}}{32768.0}$$
2. Если $\text{frameRms} > 0.005$, обновляется сглаженное значение:
   $$\text{smoothedRms} = \text{smoothedRms} \times 0.96 + \text{frameRms} \times 0.04$$
3. Вычисляется целевой коэффициент усиления с ограничением:
   $$\text{targetGain} = \operatorname{clamp}\left(\frac{0.16}{\max(0.01, \text{smoothedRms})},\ 0.5,\ 2.2\right)$$
4. Нормализованный гейн сглаживается для исключения резких скачков:
   $$\text{normGain} = \text{normGain} \times 0.94 + \text{targetGain} \times 0.06$$

#### 4. Пропуск тишины (Skip Silence)
В `AudioEngine.kt:363-380`:
- **Начало трека (Lead-in):** Если текущая позиция $< 20\,000$ мс и пиковая амплитуда $\max(|s_1|, |s_2|) < 300$, фреймы отбрасываются (drop), пока звук не начнется.
- **Конец трека (Tail):** Если длительность известна и позиция $> \text{duration} - 10\,000$ мс, и пиковая амплитуда $< 250$ непрерывно в течение 35 фреймов (~700 мс), трек завершается досрочно, не заставляя пользователя слушать длинную тишину в аутро.

#### 5. 32-полосный спектрограф (RustFFT)
В `AudioEngine.kt:426-494`:
- Кольцевой буфер на 1024 стерео-сэмпла (4096 байт).
- Применение окна Ханна:
  $$w(i) = 0.5 - 0.5 \times \cos\left(\frac{2\pi i}{1023}\right)$$
- 1024-точечное комплексное БПФ через крейт `rustfft`.
- Разбиение на 32 логарифмические частотные полосы от 40 Гц до 20 000 Гц:
  $$f_b = 40.0 \times \left(\frac{20000.0}{40.0}\right)^{\frac{b}{32.0}}$$
- Расчет уровня полосы в децибелах: $\text{db} = 20 \times \log_{10}(\text{acc} + 10^{-9})$, нормализация $v = \operatorname{clamp}\left(\frac{\text{db} + 60}{60},\ 0.0,\ 1.0\right)$.
- Пиковый спад уровней: $\text{smooth}[b] = \max(v,\ \text{smooth}[b] \times 0.82 + v \times 0.18)$.
- Эмит состояния уровней в UI происходит строго с частотой **20 Гц** (каждые 50 мс), и полностью выключается при сворачивании окна приложения (`visualsWanted = false`).

### 1.4. Фоновое кэширование и бесшовный Hot-Swap (`attachLocal`)
1. При старте любого удаленного URL поток начинает декодироваться из сети.
2. Одновременно запускается асинхронная задача `AudioCache::prefetch`: поток пишется на диск во временный файл `%APPDATA%\Drxwnify\cache\audio\<track_id>.part`.
3. По завершении скачивания файл переименовывается в `.audio`, и создается метафайл `.json`.
4. Вызывается `AudioEngine::attach_local(path)`: активный декодер запоминает путь.
5. При любой последующей перемотке или при обрыве удаленного соединения декодер мгновенно переоткрывает локальный файл на той же миллисекунде, гарантируя отсутствие сетевых лагов.

### 1.5. Watchdog защиты от зависаний стрима
В `AudioEngine.kt:85-93`:
Отдельный таймер `tokio::time::interval(Duration::from_secs(1))` проверяет отметку времени `last_sample_instant`. Если воспроизведение не на паузе, стрим удаленный (HTTP) и сэмплы не поступали более **12 секунд** (`STALL_MS = 12000`):
- Поток объявляется зависшим (`s.hung()`).
- Поток принудительно обрывается.
- Если стрим был из ВКонтакте, хост CDN заносится в черный список на 15 минут (`CdnHosts::mark_down`), и трек мгновенно перезапускается через YouTube с той же секунды.

---

## 2. Архитектура сетевой подсистемы

### 2.1. Конкурентный резолвер аудиопотоков (Racer Pattern)
В `StreamResolver.kt:102-147` реализован алгоритм соревновательного поиска аудиопотока:

```
Track Resolution Request
   │
   ├─► [Check Downloads] ──(Есть локальный файл)──► Скачано (Мгновенно)
   ├─► [Check AudioCache] ──(Есть в кэше)──────────► Кэш (Мгновенно)
   │
   └─► ЗАПУСК ГОНКИ ИСТОЧНИКОВ (Concurrent Racers):
         ├── Racer 1: YouTube Music InnerTube (160 kbps, timeout 4s)
         ├── Racer 2: VK Music API 5.116 (320 kbps, timeout 700ms..2s)
         ├── Racer 3: Lossless Mirrors (1411 kbps, timeout 9s)
         ├── Racer 4: SoundCloud (160 kbps, timeout 8s)
         ├── Racer 5: Bandcamp (128 kbps, timeout 8s)
         └── Racer 6: Audius Network (128 kbps, timeout 7s)
```

**Правила победы:**
- Бюджет ожидания VK: в режиме `speed` — 700 мс, в режиме `quality` — 2000 мс, в режиме `patient` (для скачивания) — 8000 мс.
- Если есть готовый ответ с максимальным битрейтом среди оставшихся участников — поиск немедленно завершается.
- При равенстве битрейта побеждает источник с более высоким приоритетом в настройках пользователя (`Prefs.settings.sources`).
- Если источник с высоким качеством (например, VK 320 kbps или Lossless FLAC) ответил позже, чем началось воспроизведение с YouTube, создается предложение апгрейда (`SourceUpgrades::offer`). Пользователь видит зеленую стрелку в PlayerBar и может переключиться в один клик.

### 2.2. Спецификация музыкальных источников

#### 1. YouTube Music (InnerTube API)
- **Клиенты эмуляции:**
  - `YouTubeClient.VISIONOS` (Apple Vision Pro) — возвращает прямые presigned URL на цельные аудиофайлы WebM/MP4.
  - `YouTubeClient.ANDROID_VR_1_65_10` — резервный клиент.
  - `YouTubeClient.ANDROID_VR_1_43_32` — третий резервный клиент.
- **Целевые itag:**
  - `itag 251`: WebM / Opus ~160 kbps (наивысший приоритет)
  - `itag 140`: MP4 / AAC ~128 kbps
  - `itag 250`: WebM / Opus ~70 kbps (режим saver)
  - `itag 249`: WebM / Opus ~50 kbps
  - `itag 18`: Muxed MP4 360p / AAC ~96 kbps (аварийный fallback)

#### 2. ВКонтакте (VK Music API 5.116)
- **Протокол:** `https://api.vk.com/method/audio.search`
- **Заголовки:** User-Agent официального Android-клиента VK (`VKAndroidApp/5.52-4543...`).
- **Сравнение треков (`likeness`):**
  - Очистка строк от скобок, спецсимволов и пробелов.
  - Порог сходства названия $\ge 0.7$, артиста $\ge 0.5$.
  - Штраф за разницу длительности: отклонение до 30 секунд.
- **Работа со ссылками:**
  - Регулярное выражение `/[0-9a-f]+(/audios)?/([0-9a-f]+)/index\.m3u8` автоматически конвертируется в прямой MP3 файл, если он доступен на том же сервере.
  - Если прямого MP3 нет, воспроизводится HLS `.m3u8`.

#### 3. SoundCloud
- **Извлечение Client ID:** Динамическое сканирование HTML страницы `https://soundcloud.com` и парсинг скриптов для извлечения валидного `client_id` (с кэшированием в `soundcloud.json`).
- **Эндпоинт:** `https://api-v2.soundcloud.com/search/tracks?q=...&client_id=...`
- **Строгий фильтр (ProviderGate):**
  - Максимальная разница длительности: **8 секунд** (`UPLOAD_DURATION_TOLERANCE_S = 8`). Защита от попадания ускоренных (sped-up), замедленных (slowed+reverb) и диджейских версий.
  - Исключение стоп-слов: `remix`, `edit`, `flip`, `instrumental`, `cover`, если их нет в оригинальном названии.

#### 4. Lossless Hi-Fi Mirrors
- **Эндпоинты:** Список публичных зеркал загружается динамически из RemoteConfig (`https://drxwnify.pages.dev/config.json`).
- **Поиск:** `GET /search/?s=<artist+title>`
- **Валидация:** Поиск точного совпадения по международному идентификатору **ISRC** трека. Если ISRC совпал — трек принимается со 100% уверенностью.
- **Качество:** Полноразмерный FLAC 1411 kbps.

#### 5. Bandcamp
- **Эндпоинт поиска:** `https://bandcamp.com/api/fuzzysearch/1/autocomplete`
- **Парсинг аудио:** Извлечение JSON метаданных со страницы трека, получение прямой ссылки `mp3-128`.

#### 6. Audius Network
- **Обнаружение нод:** Запрос к `https://api.audius.co`, случайный выбор одной из активных discovery-нод (с TTL 1 час).
- **Поиск:** `GET /v1/tracks/search?query=...`
- **Поток:** `GET /v1/tracks/<id>/stream` -> MP3 128 kbps.

---

## 3. Сетевая подсистема текстов песен (Lyrics)

В `Lyrics.kt:1-120` реализован многоступенчатый сервис текстов:

1. **Каскадный опрос источников:**
   - Шаг 1: **LRCLIB** (`https://lrclib.net/api/get?artist_name=...&track_name=...&duration=...`)
   - Шаг 2: **BetterLyrics**
   - Шаг 3: **KuGou**
2. **Парсинг таймкодов:**
   Регулярное выражение `\[(\d+):(\d+)(?:[.:](\d+))?\]`:
   - Минуты $\times 60\,000$ + Секунды $\times 1000$ + Миллисекунды.
3. **Пакетный машинный перевод (Google Translate):**
   Если включена настройка `translateLyrics`:
   - Строки текста группируются пачками по **15 строк** (`chunked(15)`).
   - Формируется единый запрос к `https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl=ru&dt=t&q=...`.
   - Ответ распарсивается и кэшируется в `translationCache`.

---

## 4. Социальная сеть и криптография Nostr

В `Nostr.kt`, `RelayPool.kt`, `Secp256k1.kt`, `ChatHub.kt`:

1. **Криптографический стек:**
   - **Кривая Secp256k1:** Генерация ключевой пары (32-байтный приватный ключ, 32-байтный публичный x-only ключ по BIP-340).
   - **Schnorr Signatures:** Подпись 32-байтного SHA-256 хеша сериализованного события Nostr (NIP-01).
   - **NIP-04 Шифрование сообщений:**
     - Вычисление общего секрета через ECDH: $S = d_A \times P_B$.
     - Извлечение 32 байт x-координаты.
     - Шифрование сообщения алгоритмом **AES-256-CBC** со случайным IV (16 байт).
     - Формат контента: `<base64_ciphertext>?iv=<base64_iv>`.
2. **Пул публичных релеев:**
   - Соединения по WebSocket (WSS) с 5 серверами:
     1. `wss://relay.damus.io`
     2. `wss://nos.lol`
     3. `wss://relay.primal.net`
     4. `wss://nostr.mom`
     5. `wss://relay.snort.social`
   - Пинг каждые 30 секунд. Автоматическое переподключение при разрыве.
   - Тайм-аут простоя: закрытие неиспользуемых сокетов через 90 секунд без активности.
3. **Типы событий Drxwnify Nostr:**
   - `Kind 0`: Метаданные профиля (имя, аватар).
   - `Kind 4`: Личные зашифрованные сообщения (DM) ChatHub.
   - `Kind 30078`: Параметризованные заменяемые события (`d = drxw:now`, `drxw:history`, `drxw:playlists`, `drxw:likes`).
   - `Kind 21004`: Индикатор набора текста («печатает...»).

---

## 5. Интеграция с Discord Rich Presence

В `DiscordRpc.kt:1-183`:
- Подключение к локальному именованному каналу Windows: `\\.\pipe\discord-ipc-0` ... `\\.\pipe\discord-ipc-9`.
- **Протокол пакетов:**
  - 4 байта Opcode (Little Endian): `0` = Handshake, `1` = Frame, `2` = Close.
  - 4 байта Длина JSON тела (Little Endian).
  - Строка JSON в UTF-8.
- **Handshake (Opcode 0):** `{"v": 1, "client_id": "1411019391843172514"}`.
- **Отправка активности (Opcode 1):**
  - `cmd = SET_ACTIVITY`, `details = track.title`, `state = track.artistLine`.
  - `timestamps.start` / `timestamps.end` (в секундах).
  - `assets.large_image = track.cover`, `assets.small_image = "play" / "pause"`.
  - Кнопка: `{"label": "Слушать в Drxwnify", "url": "https://drxwnify.pages.dev"}`.
