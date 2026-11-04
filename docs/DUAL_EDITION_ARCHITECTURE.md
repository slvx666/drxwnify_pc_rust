# DRXWNIFY — DUAL EDITION MASTER ARCHITECTURE (DUAL_EDITION_ARCHITECTURE.md)

**Target Environment:** Windows 10 (Build 19041+) and Windows 11 x64  
**Workspace Root:** `Drxwnify/rust-rewrite/`  
**Edition 1:** `Drxwnify Full` (Tauri 2.0 + DirectComposition + WebView2, 100% visual recreation)  
**Edition 2:** `Drxwnify Mini` (Pure Rust Native UI / egui+glow, high performance, 6 distinct themes)  
**Shared Core:** Modular Rust Workspace (`crates/*`), linked statically as library code  
**Status:** Canonical Dual Edition Architecture

---

## 1. Концепция двух редакций и единого ядра (Single Core, Dual Editions)

Drxwnify переходит на архитектуру с единым Rust Core и двумя независимыми пользовательскими приложениями:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        DRXWNIFY RUST WORKSPACE                         │
├───────────────────────────────────┬────────────────────────────────────┤
│         DRXWNIFY FULL             │          DRXWNIFY MINI             │
│   (Tauri 2 + DirectComposition)   │    (Native Rust UI / egui+glow)    │
│  - 100% визуальный клон оригинала │  - Минимальное потребление ОЗУ/CPU │
│  - 9 тем x 2 (18 палитр)          │  - 6 самостоятельных тем           │
│  - 8 визуальных стилей Look       │  - Сохранение 3-панельной схемы    │
│  - 9 процедурных Canvas фонов     │  - Событийная модель без поллинга  │
│  - Аппаратный Haze Blur           │  - Нулевая отрисовка в простое     │
│  - AppId: com.drxwnify.desktop    │  - AppId: com.drxwnify.mini        │
│  - Dir: %APPDATA%\DrxwnifyRust\   │  - Dir: %APPDATA%\DrxwnifyMini\    │
└─────────────────┬─────────────────┴──────────────────┬─────────────────┘
                  │                                    │
                  │        Direct Library Linkage      │
                  ▼                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                    SHARED RUST CORE (crates/*)                         │
│                                                                        │
│  ┌───────────────────────┐ ┌──────────────────────┐ ┌───────────────┐ │
│  │ drxwnify-core         │ │ drxwnify-audio       │ │ drxwnify-dsp  │ │
│  │ - Models, Paths, Logs │ │ - FFmpeg, WASAPI CPAL│ │ - RMS, FFT 32 │ │
│  └───────────────────────┘ └──────────────────────┘ └───────────────┘ │
│  ┌───────────────────────┐ ┌──────────────────────┐ ┌───────────────┐ │
│  │ drxwnify-playback     │ │ drxwnify-resolver    │ │drxwnify-sources││
│  │ - Queue 500, Shuffle  │ │ - Stream Racer       │ │- YT, VK, SC,  │ │
│  │ - SleepTimer, Radio   │ │ - Source Upgrades    │ │  FLAC, Bandcamp│ │
│  └───────────────────────┘ └──────────────────────┘ └───────────────┘ │
│  ┌───────────────────────┐ ┌──────────────────────┐ ┌───────────────┐ │
│  │ drxwnify-catalog      │ │ drxwnify-storage     │ │drxwnify-lyrics│ │
│  │ - Spotify, Yandex     │ │ - Atomic JSON, Cache │ │- LRC, KuGou,  │ │
│  │ - AppLinks, Reviews   │ │ - Downloads, Backup  │ │  Translate    │ │
│  └───────────────────────┘ └──────────────────────┘ └───────────────┘ │
│  ┌───────────────────────┐ ┌──────────────────────┐ ┌───────────────┐ │
│  │ drxwnify-nostr        │ │ drxwnify-win32       │ │drxwnify-discord││
│  │ - Secp256k1, RelayPool│ │ - SMTC, Hotkeys, Tray│ │- NamedPipe RPC│ │
│  │ - ChatHub, Stickers   │ │ - SingleInstance     │ │               │ │
│  └───────────────────────┘ └──────────────────────┘ └───────────────┘ │
└────────────────────────────────────────────────────────────────────────┘
```

### 1.1. Принципы изоляции и связности
1. **Прямое связывание (Direct Library Linking):**
   - Никаких фоновых HTTP/gRPC серверов или демонов-посредников.
   - Обе редакции (`drxwnify-full` и `drxwnify-mini`) компилируются слинкованными напрямую с крейтами ядра.
   - Вызовы Core осуществляются через нативные методы и асинхронные каналы (`tokio::sync::mpsc`, `broadcast`).
2. **Аудиопотоки вне IPC:**
   - Аудиоданные декодируются и направляются в Windows WASAPI исключительно в нативном потоке Core.
   - В UI передаются только легковесные события метаданных (статус воспроизведения, текущая миллисекунда, 32 логарифмические полосы FFT с частотой 20 Гц).
3. **Абсолютная независимость процессов:**
   - Full и Mini имеют разные `AppUserModelID`:
     - Full: `com.drxwnify.desktop`
     - Mini: `com.drxwnify.mini`
   - Разные порты `SingleInstance`:
     - Full: `127.0.0.1:47834`
     - Mini: `127.0.0.1:47835`
     - (Оригинал Kotlin слушал `47833`)
   - Раздельные рабочие директории:
     - Full: `%APPDATA%\DrxwnifyRust\`
     - Mini: `%APPDATA%\DrxwnifyMini\`
   - Обе редакции могут работать одновременно на одном компьютере без малейших конфликтов или перезаписи данных.

---

## 2. Сравнительная характеристика редакций

| Параметр | Drxwnify Full | Drxwnify Mini |
|---|---|---|
| **Главный приоритет** | 100% визуальное воспроизведение оригинала | Экстремальная производительность, минимум ОЗУ и CPU |
| **UI Технология** | Tauri 2.0 (DirectComposition + WebView2) | Pure Rust Native UI (`egui` + `glow` аппаратно) |
| **Компоновка** | Классическая 3-панельная (Шелл оригинала) | Сохраненная 3-панельная с компактными карточками |
| **Темы оформления** | 9 тем x 2 режима (18 палитр) | 6 специализированных тем (Night, Day, Gruvbox, Satanic, Sakura Pixel, Linux Terminal) |
| **Визуальные стили** | 8 стилей Look (Terminal, Glass, Neon, etc.) | Единая плоская дизайн-система без лишних слоев |
| **Процедурная графика**| 9 Canvas 2D/WebGL бэкдропов | Статичные легковесные акценты, 0 FPS в покое |
| **Эффект стекла (Blur)**| Аппаратный `backdrop-filter: blur(28px)` | Отсутствует (плоские контрастные поверхности) |
| **Память в покое (RAM)**| **85 – 120 МБ** | **25 – 45 МБ** |
| **CPU в покое** | **< 0.1%** (RAF stop) | **0.0%** (Event-driven repaint) |
| **Холодный старт** | **180 – 300 мс** | **40 – 80 мс** |
| **Размер инсталлятора**| **~12 – 15 МБ MSI** | **~8 – 11 МБ MSI** |
| **Функциональность** | **100% (по матрице)** | **100% (абсолютный паритет с Full)** |

---

## 3. Архитектура общего ядра (`crates/*`)

Каждый крейт выполняет строго разграниченную роль:

1. **`drxwnify-core`**:
   - Общие доменные модели: `Track`, `AlbumRef`, `ArtistRef`, `Playlist`, `Lyrics`, `User`.
   - Конфигурация путей: структура `AppPaths`, разделяющая Full и Mini.
   - Глобальные ошибки: `DrxwnifyError`, `Result<T>`.
   - Система логирования на базе `tracing`.
2. **`drxwnify-audio`**:
   - Декодер FFmpeg (libavformat, libavcodec, libswresample) для Opus, AAC, MP3, FLAC, Vorbis, HLS AES-128.
   - Вывод звука WASAPI через `cpal`.
   - DSP: перцептивная громкость $x^{1.7}$, стереобаланс, динамический RMS нормализатор, детектор тишины.
   - RustFFT 1024-точечный спектрограф (32 полосы).
   - Watchdog защиты от зависаний стрима (12 секунд).
3. **`drxwnify-playback`**:
   - Автомат состояний плеера: Idle, Loading, Playing, Paused, Stopped.
   - Очередь до 500 треков, Shuffle, Repeat (Off, All, One), Endless Queue (Radio).
   - Таймер сна с плавным угасанием звука за 20 секунд.
4. **`drxwnify-sources`**:
   - YouTube InnerTube (ротация VisionOS, Android VR 1.65, 1.43).
   - ВКонтакте API 5.116 (прямой MP3 и HLS AES-128).
   - SoundCloud (скрапинг client_id, фильтр длительности 8с).
   - Lossless Hi-Fi зеркала (валидация ISRC).
   - Bandcamp (fuzzy search, mp3-128).
   - Audius (ротация нод discovery).
5. **`drxwnify-resolver`**:
   - Конкурентный Stream Racer (`FuturesUnordered`).
   - Динамические бюджеты времени (speed 700ms, quality 2000ms, patient 8000ms).
   - Предложения апгрейда качества (`SourceUpgrades`).
   - Сохранение ручного выбора версий (`matches.json`).
6. **`drxwnify-catalog`**:
   - Spotify GQL и Web API.
   - Яндекс Музыка API.
   - Парсер ссылок `AppLinks` (`drxwnify://`, `spotify.com`, `music.yandex.ru`, `pages.dev`).
   - Ночные обзоры `drxwnslvt` и персональные рекомендации.
7. **`drxwnify-storage`**:
   - Атомарное сохранение JSON через `.tmp` и переименование.
   - Управление медиатекой: `library.json`, `settings.json`, `accounts.json`, `queue.json`.
   - Дисковый аудиокэш (`cache/audio/`) и фоновый префетч с `attachLocal`.
   - Менеджер загрузок и ремуксинг потоков без перекодирования.
   - Импорт резервной копии Android (`.backup`, `song.db`, SharedPreferences).
8. **`drxwnify-lyrics`**:
   - Каскадный поиск LRC: LRCLIB -> BetterLyrics -> KuGou.
   - Парсинг таймкодов `[mm:ss.xx]`.
   - Пакетный перевод через Google Translate с кэшированием.
9. **`drxwnify-nostr`**:
   - Криптография Secp256k1, Schnorr BIP-340, Bech32 (npub/nsec).
   - Пул WebSocket релеев (`damus`, `nos.lol`, `primal`, `nostr.mom`, `snort`).
   - ChatHub: личные сообщения (Kind 4 DM, NIP-04 AES-256-CBC), индикатор набора, реакции.
   - 36 WebP стикеров.
   - Совместное прослушивание (Kind 30078).
10. **`drxwnify-win32`**:
    - Windows System Media Transport Controls (SMTC).
    - Глобальные горячие клавиши через Win32 `RegisterHotKey`.
    - Системный трей и управление окном.
    - SingleInstance через TCP loopback сокет.
11. **`drxwnify-discord`**:
    - Discord Rich Presence через локальный Named Pipe `\\.\pipe\discord-ipc-0..9`.

---

## 4. Обмен данными между редакциями (Data Portability)

Для удобства пользователей реализован инструмент экспорта/импорта между редакциями:
- Из Full в Mini: копирование медиатеки, избранного, плейлистов и настроек источников.
- Из Mini в Full: обратная синхронизация без риска повреждения оригинальных данных.
- Форматы данных полностью унифицированы: `library.json`, `accounts.json`, `matches.json` идентичны для обоих приложений.
