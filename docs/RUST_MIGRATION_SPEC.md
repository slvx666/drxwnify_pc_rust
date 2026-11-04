# RUST REWRITE — MASTER MIGRATION SPECIFICATION (RUST_MIGRATION_SPEC.md)

**Target Application:** Drxwnify Desktop (Music Player & Social Client)  
**Current Baseline:** Kotlin 2.3.20 / Compose Multiplatform Desktop 1.12.2 / OpenJDK 21 / FFmpeg 8.1.2 (JavaCV 1.5.14) / Version 0.3.0 beta (Audited baseline v2.2.4)  
**Target Architecture:** Rust Native Windows Application (Target: `x86_64-pc-windows-msvc`)  
**Target Environment:** Windows 10 (Build 19041+) and Windows 11 x64  
**Status:** Canonical Blueprint & Architectural Specification for Full Rewrite

---

## 1. Введение и цели миграции

### 1.1. Текущее состояние приложения
Drxwnify — полнофункциональный десктопный музыкальный плеер с гибридной архитектурой каталога (индексация Spotify, Yandex Music, YouTube Music) и многопоточным поиском аудиопотоков в реальном времени (YouTube InnerTube, VK Music API 5.116, SoundCloud, Lossless Hi-Fi mirrors, Bandcamp, Audius). Приложение содержит развитую социальную подсистему на базе протокола Nostr (децентрализованный чат ChatHub, обмен треками, индикация статуса совместного прослушивания, система стикеров) и богатый графический интерфейс с 9 темами оформления (каждая в ночном и дневном вариантах — 18 конфигураций палитр), 8 визуальными стилями (`Look`), процедурными фонами на Canvas и динамическим размытием панелей (Haze Blur).

В текущей версии на базе JVM / Compose Desktop приложение имеет существенные ресурсные накладные расходы, зафиксированные в `pc/audit/AUDIT_REPORT.md`:
1. **CPU в простое:** **2.40% общего времени CPU на 12-ядерном процессоре** (28.8% одного логического ядра) из-за непрерывного 30 FPS цикла `rememberClock` и фильтра Гаусса HazeBlur над процедурным Canvas.
2. **Оперативная память:** **537.8 МБ Working Set** (865.3 МБ Private Bytes), из которых более 240 МБ занимает видеодрайвер Direct3D, 65.7 МБ — библиотеки FFmpeg, и десятки мегабайт — неуправляемые C++ объекты Skia.
3. **Размер установщика:** **121.96 МБ MSI** (189.44 МБ в установленном виде), вызванный включением полной среды OpenJDK 21 JRE (54.8 МБ), неоптимальных бандлов FFmpeg с лишними кодеками (29.0 МБ) и кроссплатформенных библиотек SQLite JDBC со всеми ОС (13.65 МБ).

### 1.2. Цели переписывания на Rust
1. **Полная независимость:** Самостоятельный проект в изолированной директории `rust-rewrite/`, не требующий Java Virtual Machine (JVM), Kotlin-рантайма или внешних фоновых процессов.
2. **Абсолютная функциональная эквивалентность:** 100% сохранение всех пользовательских функций, сетевых интеграций, источников музыки, Nostr-чата, Discord RPC, глобальных хоткеев и Windows SMTC.
3. **Визуальная идентичность:** Точное воспроизведение всех 9 тем, 8 стилей `Look`, эффекта матового стекла (Haze Blur), процедурной графики (падающая сакура, звездный атлас, готическая часовня с руническим кругом, ретровейв-сетка, частицы пыли, капли глубины) и 27 шрифтов.
4. **Радикальная оптимизация:**
   - Потребление RAM в простое: **80–120 МБ** (снижение на 75–85%).
   - Нагрузка на CPU в простое: **< 0.2%** от одного ядра (вместо 28.8%).
   - Размер MSI установщика: **10–18 МБ** (снижение на 85–90%).
   - Холодный старт приложения: **150–350 мс** (вместо 2.5–4.0 секунд).

---

## 2. Архитектура текущей системы (Kotlin / Compose)

### 2.1. Инвентаризация модулей и зависимостей оригинала

```
┌────────────────────────────────────────────────────────────────────────┐
│               PRESENTATION LAYER (Jetpack Compose Desktop)             │
│  - Shell.kt, Components.kt, Overlays.kt, Nav.kt, Menu.kt, Ti.kt        │
│  - 12 Screens: Home, Search, Liked, Downloads, History, LocalPlaylist, │
│    RemotePlaylist, Album, Artist, Discography, Reviews, Friends        │
│  - Themes: Theme.kt (9 тем x 2 режима), Backdrops.kt, Background.kt,   │
│    Occult.kt (готические шейдеры/глифы), Fonts.kt (27 OTF/TTF)         │
│  - Glass Effect: dev.chrisbanes.haze:haze 2.0.1 (Direct3D 11/12)       │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ StateFlow / Events
┌───────────────────────────────────▼────────────────────────────────────┐
│                    DOMAIN LAYER (Playback & Media Core)                │
│  - Player.kt (State machine, Queue 500 tracks, Shuffle, Repeat, Radio) │
│  - AudioEngine.kt (FFmpegFrameGrabber, 48kHz S16 Stereo, Volume^1.7,   │
│    Balance, Silence Skip, Normalization RMS, 32-band FFT Spectrogram)  │
│  - StreamResolver.kt (Racer pattern, concurrent multi-source fallback) │
│  - SourceUpgrades.kt (Dynamic bitrate upgrade notifications)           │
│  - AudioCache.kt (Background prefetch & hot-swap to local file)        │
│  - Lyrics.kt (LRCLIB, BetterLyrics, KuGou, Google Translate batching)  │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Async I/O (OkHttp / Ktor)
┌───────────────────────────────────▼────────────────────────────────────┐
│                 DATA & INTEGRATION LAYER (Network, Win32)              │
│  - Store.kt (Atomic JSON persistence in %APPDATA%\Drxwnify)            │
│  - Music Sources: InnerTube, VkSource, SoundCloud, Lossless, Bandcamp  │
│  - Catalogs: SpotifyCatalog, YandexCatalog, YouTubeCatalog, AppLinks   │
│  - Social: Nostr.kt, Secp256k1.kt, RelayPool.kt (5 WSS), ChatHub.kt    │
│  - Windows Native: WindowsApp.kt (AppId/Styles), Hotkeys.kt (JNA Win32)│
│    MediaOverlay.kt (WinRT SMTC via combase), DiscordRpc.kt (NamedPipe) │
│  - SingleInstance.kt (TCP loopback 127.0.0.1:47833)                    │
│  - PhoneBackup.kt (Zip, Room song.db SQLite import, PB DataStore)      │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Архитектурный выбор UI-технологии в Rust

Вопрос выбора GUI в Rust является критическим. Десктопное приложение Drxwnify сочетает:
- Высокую плотность динамических компонентов управления медиатекой.
- Настоящее размытие по Гауссу поверх анимированного процедурного фона (Haze Blur).
- Сложные математические холсты (Canvas 2D: кривые Безье, 48 рунических засечек с независимым вращением, 170 звезд с мерцанием, частицы пыли и сакуры).
- 27 кастомных гарнитур шрифтов и векторные иконки Tabler.
- Адаптивные плавающие панели, ресайз и контекстные меню.

### 3.1. Сравнительный анализ кандидатов

| Критерий | Slint (v1.8+) | Iced (v0.13+) | egui (v0.29+) | Tauri v2.0 (Rust Core + WebUI) | Dioxus Desktop |
|---|---|---|---|---|---|
| **Тип архитектуры** | Pure Rust Native UI | Pure Rust Native UI | Immediate Mode Pure Rust | Rust Core + Native WebView2 | Rust + Wry (WebView2) |
| **Потребление RAM в простое** | **25–40 МБ** | 50–80 МБ | 40–70 МБ | **85–125 МБ** | 90–135 МБ |
| **CPU в покое** | **< 0.1%** | 0.2–0.5% | 1.0–3.0% (требует reactive hacks) | **< 0.1% (RAF stops on idle)** | < 0.2% |
| **Поддержка Haze Blur / Glass** | ❌ Нет нативного (требует кастомный C++ Skia бэкенд) | ❌ Отсутствует (только wgpu шейдеры поверх) | ❌ Крайне сложно (дополнительные фреймбуферы) |  **Нативная аппаратная поддержка (`backdrop-filter: blur()`)** |  Нативная (`backdrop-filter`) |
| **Процедурная графика Canvas** | Ограничено DSL | Требует wgpu render pipeline | Ограничено треугольниками egui Painter |  **100% эквивалент Canvas 2D / WebGL2** |  Canvas 2D / WebGL2 |
| **Кастомные шрифты (27 шт.)** | Ручная регистрация | Через iced::font | Загрузка в egui FontDefinitions |  **Нативный CSS `@font-face`** |  CSS `@font-face` |
| **Сложные списки (5000+ треков)** | VirtualizedListView | Виртуализация через scrollable | `ScrollArea::show_rows` |  Виртуализация (TanStack Virtual) |  Виртуализация |
| **Размер инсталлятора MSI** | **6–9 МБ** | 8–12 МБ | 7–10 МБ | **9–15 МБ (WebView2 уже в Windows)** | 10–16 МБ |
| **Зрелость экосистемы компонентов** | Средняя (свой DSL `.slint`) | Развивающаяся | Высокая, но не медийная | **Максимальная (весь мир современного UI)** | Средняя |
| **Скорость запуска** | **60–100 мс** | 100–180 мс | 80–140 мс | **180–320 мс** | 200–350 мс |

### 3.2. Главное архитектурное решение: Tauri 2.0 (Rust Core + Svelte 5 / TypeScript)

**Рекомендация:** **Tauri 2.0** признан **единственным решением**, способным обеспечить **100% визуальную идентичность без компромиссов** в разумные сроки разработки:
1. **Эффект матового стекла (Haze Blur):** В Compose Desktop библиотека `dev.chrisbanes.haze` использует нативный Skia Gaussian Blur по текстуре Direct3D. В веб-движке Chromium (WebView2, встроенном в Windows 10/11) свойство CSS `backdrop-filter: blur(28px) saturate(180%)` выполняется аппаратно на GPU через DirectComposition без какой-либо нагрузки на процессор. Ни один pure-Rust GUI (ни Slint, ни Iced, ни egui) не обладает встроенным композитингом произвольного размытия с наложением полупрозрачных панелей.
2. **Процедурные фоны:** Все 9 алгоритмов генерации (`SakuraNightSky`, `StarAtlas`, `CozyRoom`, `Ash`, `Retro`, `Depth`, `GlyphField`, `Mist`) переносятся на стандартный аппаратный HTML5 Canvas 2D Context метод в метод с сохранением всех тригонометрических формул и шума Перлина.
3. **Ресурсы и установщик:** В отличие от Electron (который тащит 140 МБ Node.js и Chromium), Tauri 2 на Windows 10/11 x64 использует системный `WebView2Loader.dll` и встроенный системный рантайм. Итоговый MSI установщик занимает всего **~11-14 МБ**, а потребление оперативной памяти составляет **90–120 МБ** (в 5 раз меньше текущего Kotlin-приложения).
4. **Роль Rust:** 100% тяжелой логики, сетевых соединений, криптографии Nostr, управления очередью, аудиопайплайна WASAPI, декодирования FFmpeg, взаимодействия с Windows API и файловой системы работает на нативном многопоточном Rust Core. Интерфейс является тонким слоем представления (Presentation Layer), общающимся с ядром через строго типизированные бинарные IPC-команды Tauri (`tauri::command` / `tauri::emit`).

#### Альтернатива Pure-Rust (Slint 1.8 + Skia Backend)
Если обязательным требованием является компиляция GUI без участия WebView2, единственным кандидатом является **Slint с бэкендом `slint-skia`**:
- *Компромиссы:* Потребуется ручное написание C++ обертки вокруг Skia для воспроизведения шейдеров Haze, полный отказ от стандартного набора виджетов Slint в пользу ручной отрисовки через Skia Canvas, значительное усложнение стилизации текста и кастомных скроллбаров. Затраты времени на разработку возрастают в 3–4 раза.

---

## 4. Архитектура ядра Rust (Backend Core)

Будущий проект проектируется как модульный Cargo Workspace:

```text
drxwnify-rust/
├── Cargo.toml                  # Workspace Root
├── crates/
│   ├── drxwnify-core/          # Доменные модели, пути, настройки, логирование
│   ├── drxwnify-audio/         # Движок WASAPI, декодеры FFmpeg, нормализация, FFT
│   ├── drxwnify-playback/      # Очередь, радио, перемешивание, таймер сна
│   ├── drxwnify-resolver/      # Мульти-источники, конкурентный опрос, апгрейды
│   ├── drxwnify-sources/       # YouTube, VK, SoundCloud, Lossless, Bandcamp, Audius
│   ├── drxwnify-catalog/       # Spotify GQL, Yandex Music, YouTube Music, AppLinks
│   ├── drxwnify-storage/       # JSON Store, AudioCache, Downloads, PhoneBackup
│   ├── drxwnify-lyrics/        # LRCLIB, BetterLyrics, KuGou, Google Translate
│   ├── drxwnify-nostr/         # Secp256k1, RelayPool, ChatHub, Stickers
│   ├── drxwnify-win32/         # AppUserModelID, WindowStyles, SMTC, Hotkeys, SingleInstance
│   └── drxwnify-app/           # Tauri App Binary, IPC Bridge, UI Assets
```

---

## 5. Аудиоархитектура Rust

Полная схема аудиопайплайна:

```
[Remote Stream: HTTP/HLS AES-128]  ──┐
                                     ├──> [FFmpeg Demuxer / Decoder] ──> [Audio Resampler]
[Local File: .opus/.m4a/.mp3/.flac] ─┘      (libavformat / libavcodec)     (libswresample)
                                                                                  │
                                                                   48 kHz S16 Stereo PCM
                                                                                  │
                                                                                  ▼
                                                                     [Pre-processing Stage]
                                                                     - Skip Silence Gate
                                                                     - Dynamic RMS Normalizer
                                                                     - Volume Curve (x^1.7)
                                                                     - Stereo Balance Balance
                                                                     - Crossfade / Fade Out
                                                                                  │
                                               ┌──────────────────────────────────┴──────────────────────────────────┐
                                               ▼                                                                     ▼
                                    [Ring Buffer (1024 frames)]                                          [Audio Output Buffer]
                                               │                                                                     │
                                    [Hann Windowed RustFFT]                                              [WASAPI Device via CPAL]
                                               │                                                                     │
                                    [32 Log Bands (40Hz-20kHz)]                                          [Hardware Audio Mixer]
                                               │
                                    [Tauri IPC Event @ 20Hz] ──> [PlayerBar Spectrogram]
```

### 5.1. Ключевые требования к реализации
1. **Форматы и кодеки:** Полная поддержка Opus, AAC, MP3, FLAC, Vorbis, а также протокола Apple HTTP Live Streaming (HLS `.m3u8`) с сегментами, зашифрованными AES-128 (требуется для аудиозаписей ВКонтакте).
2. **Бесшовное переключение на кэш (`attachLocal`):** При старте удаленного трека параллельная фоновая задача `tokio::spawn` начинает загрузку трека в дисковый кэш. Как только файл скачан, активный `FFmpegFrameGrabber` бесшовно подменяется на локальный файл на той же миллисекунде воспроизведения, исключая дальнейшие сетевые задержки и делая перемотку мгновенной.
3. **Watchdog защиты от зависаний:** Демонический поток с проверкой тайм-аута в 12 секунд. При отсутствии декодированных сэмплов поток принудительно обрывается и переключается на YouTube резерв.

---

## 6. Сетевая архитектура и многопоточность

### 6.1. Асинхронный сетевой стек
- **Tokio Runtime:** Bounded Thread Pool с 4–6 рабочими потоками для асинхронного I/O.
- **HTTP Клиент (`reqwest`):** Пул соединений с поддержкой HTTP/2, сжатия Brotli/Gzip, настраиваемых заголовков User-Agent (эмуляция мобильных клиентов VK, YouTube VisionOS/Android VR).
- **TLS:** `rustls-tls` для исключения внешней зависимости от OpenSSL DLL или `native-tls` для работы через системное хранилище сертификатов Windows Schannel.
- **Nostr WebSocket:** Пул соединений на базе `tokio-tungstenite` к 5 независимым публичным релеям (`damus.io`, `nos.lol`, `primal.net`, `nostr.mom`, `snort.social`).

---

## 7. Совместимость данных и миграция

Новая версия Drxwnify Rust проектируется с **нулевым риском повреждения данных**:
1. **Изолированный рабочий каталог:** Приложение в режиме разработки сохраняет данные в `%APPDATA%\DrxwnifyRust`.
2. **Встроенный импорт оригинальных данных:**
   - Чтение существующих файлов `%APPDATA%\Drxwnify\settings.json`, `library.json`, `matches.json`, `friends.json`, `chats.json`, `queue.json`.
   - Структуры данных в `drxwnify-storage` используют атрибуты `#[serde(default)]` и строгую совместимость полей, гарантируя 100% прозрачную миграцию без необходимости конвертеров.
3. **Импорт резервных копий Android (`.backup`):** Распаковка архива через `zip`, парсинг базы данных Room `song.db` через встроенную `rusqlite` и импорт XML-файлов настроек.

---

## 8. Безопасность и надежность

1. **Безопасное хранение секретов:** Приватный ключ Nostr (`nostrSecret`), токены VK, Yandex, Spotify сохраняются в зашифрованном виде с использованием Windows DPAPI (`CryptProtectData`) через crate `windows::Win32::Security::Cryptography`.
2. **Отказоустойчивость:** Все внешние запросы (каталоги, источники аудио, релеи Nostr) изолированы в корутинах `tokio` с таймаутами от 5 до 12 секунд. Сбой одного релея или источника не блокирует воспроизведение.

---

## 9. План сборки и дистрибуции MSI

- **Сборщик:** `tauri build --bundles msi` на базе WiX Toolset 3.11/4.0.
- **Компоненты пакета:**
  1. `Drxwnify.exe` — самостоятельный исполняемый файл (статически скомпилированный Rust с упакованными ресурсами фронтенда).
  2. Набор DLL FFmpeg (минимальная LGPL конфигурация: `avcodec.dll`, `avformat.dll`, `avutil.dll`, `swresample.dll` — общий вес ~12 МБ).
  3. Иконки приложения (`icon.ico`).
- **Отсутствующие компоненты:** JVM (экономия 55 МБ), Java Sound, лишние видеокодеки FFmpeg (экономия 22 МБ), неиспользуемые нативные библиотеки sqlite-jdbc под Linux/Mac (экономия 24 МБ).
- **Итоговый размер дистрибутива:** **~12–16 МБ MSI**.
