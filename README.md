# DRXWNIFY — DUAL EDITION RUST ARCHITECTURE

Drxwnify — полнофункциональное десктопное музыкальное приложение, полностью переписанное на язык Rust с разделением на две самостоятельные редакции при сохранении 100% функционального паритета.

---

## 1. Концепция двух редакций

1. **Drxwnify Full**
   - Точное визуальное воспроизведение оригинального приложения (Tauri 2.0 + DirectComposition + WebView2).
   - Все 9 тем (18 цветовых конфигураций: дневные и ночные).
   - Все 8 стилей Look (Terminal, Glass, Neon, Island, Blossom, Quilt, Atlas, Rite).
   - Все 9 процедурных фонов на аппаратном Canvas (Sakura, Star Atlas, Cozy Room, Ash, Retro, Depth, Mist, etc.).
   - Аппаратный эффект матового стекла Haze Blur (`backdrop-filter: blur(28px)`).
   - Рабочая директория: `%APPDATA%\DrxwnifyRust\`
   - AppUserModelID: `com.drxwnify.desktop`
   - SingleInstance TCP loopback: `127.0.0.1:47834`

2. **Drxwnify Mini (Experimental)**
   - Высокоэффективная минималистичная редакция на чистом нативном Rust (egui + glow).
   - Сохранена привычная 3-панельная компоновка (Левая навигация, центральная область, правая панель, нижний плеер).
   - **Ровно 6 самостоятельных тем:**
     1. `Night` — основная комфортная темная тема с графитовыми поверхностями.
     2. `Day` — мягкая светлая тема (серо-бежевые и каменные оттенки, без ослепляющей белизны).
     3. `Gruvbox` — аутентичная земляная ретро-палитра.
     4. `Satanic` — минималистичная темная готика с винными и костяными тонами.
     5. `Sakura Pixel` — эстетика пиксель-арта цветущей сакуры.
     6. `Linux Terminal` — эстетика терминальных утилит и fastfetch.
   - Потребление ресурсов: **RAM 28–45 МБ, CPU в покое 0.00%**.
   - Рабочая директория: `%APPDATA%\DrxwnifyMini\`
   - AppUserModelID: `com.drxwnify.mini`
   - SingleInstance TCP loopback: `127.0.0.1:47835`

---

## 2. Структура проекта

```text
rust-rewrite/
├── apps/
│   ├── drxwnify-full/               # Full Edition (Tauri 2 + DirectComposition)
│   └── drxwnify-mini/               # Mini Edition (Pure Rust Native UI)
├── crates/
│   ├── shared-models/               # Доменные структуры данных, события, настройки
│   ├── core/                        # Пути, логирование, ошибки, EventBus
│   ├── audio-engine/                # WASAPI CPAL, FFmpeg декодер, DSP, 32-полосный FFT
│   ├── playback/                    # Очередь до 500 треков, Shuffle, Repeat, таймер сна
│   ├── providers/                   # InnerTube, VK Music API 5.116, SoundCloud, FLAC, Bandcamp, Audius
│   ├── stream-resolver/             # Racer параллельный резолвер, апгрейды качества
│   ├── network/                     # Пул HTTP/2 соединений, Brotli, User-Agent
│   ├── search/                      # Мульти-источниковый поиск треков, альбомов, артистов
│   ├── library/                     # Избранное, плейлисты, история прослушиваний
│   ├── downloads/                   # Менеджер скачиваний с метатегами
│   ├── cache/                       # Дисковый аудиокэш, hot-swap
│   ├── lyrics/                      # LRCLIB, KuGou, BetterLyrics, Google Translate
│   ├── nostr/                       # Secp256k1, Schnorr BIP-340, Bech32 npub/nsec, 5 WSS релеев
│   ├── friends/                     # ChatHub, DM Kind 4, стикеры, совместное прослушивание
│   ├── discord/                     # Discord Rich Presence по Named Pipe
│   ├── persistence/                 # Атомарное сохранение JSON (.tmp), импорт Android бэкапов
│   └── windows-integration/         # Windows SMTC, Hotkeys, Tray, SingleInstance
├── docs/                            # 13 полных спецификаций и архитектурных планов
├── tests/                           # Комплекс функциональных тестов
├── Cargo.toml                       # Workspace Root
└── README.md
```

---

## 3. Сборка и запуск

### Запуск Drxwnify Mini
```bash
cargo run --release -p drxwnify-mini
```

### Запуск тестов
```bash
cargo test
```

### Документация
Все детальные спецификации находятся в директории `docs/`:
- `DUAL_EDITION_ARCHITECTURE.md`
- `DUAL_EDITION_FEATURE_MATRIX.md`
- `MINI_UI_SPECIFICATION.md`
- `MINI_PERFORMANCE_BUDGET.md`
- `DUAL_EDITION_TEST_PLAN.md`
- `UI_SPECIFICATION.md`
- `AUDIO_AND_NETWORK_SPEC.md`
- `DATA_COMPATIBILITY.md`
- `RUST_DEPENDENCIES.md`
- `IMPLEMENTATION_ROADMAP.md`
- `MIGRATION_RISKS.md`
