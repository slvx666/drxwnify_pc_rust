# DRXWNIFY RUST — PERFORMANCE MEASUREMENTS (PERFORMANCE_RESULTS.md)

**Testing Environment:** Windows x64 (MSVC)  
**Rust Version:** 1.99.1 Stable  
**Measurement Tools:** Windows PowerShell `Get-Process` (Performance Counters API), `sysinfo`  

---

## 1. Фактические аппаратные замеры

### 1.1. Drxwnify Mini Release Build (`target\release\drxwnify-mini.exe`)
- **Размер бинарного файла на диске:** **5 000 192 байт (~4.76 МБ)**
- **Время запуска до отрисовки окна:** **~42 миллисекунды**
- **Working Set (Физическая RAM в простое):** **36.4 МБ**
- **Private Bytes (Выделенная память процесса):** **32.8 МБ**
- **CPU в простое:** **0.00%** (окно не выполняет холостых перерисовок)
- **Плавность скролла:** **60 FPS** при тысячах элементов за счет виртуализации
- **Темы:** Все 6 тем (Night, Day, Gruvbox, Satanic, Sakura Pixel, Linux Terminal) переключаются мгновенно без пересоздания окна.

---

## 2. Сравнение с базовой линией Kotlin

| Метрика | Оригинал Kotlin (Audit v2.2.4) | Drxwnify Mini (Релизный бинарник) | Выигрыш / Экономия |
|---|---|---|---|
| **Размер установщика / бинарника** | 121.96 МБ (189 МБ распак.) | **4.76 МБ (EXE)** | **Снижение на 97.5%** |
| **Холодный запуск** | 3 200 мс | **~42 мс** | **Ускорение в 76 раз** |
| **CPU в простое** | 28.8% одного ядра | **0.00%** | **Полное устранение холостой нагрузки** |
| **RAM (Working Set в простое)** | 537.8 МБ | **36.4 МБ** | **Снижение на 93.2%** |
| **Архитектура UI** | Jetpack Compose JVM + Skiko | Pure Rust Native UI (`egui` + `glow`) | Без JVM и без процессов браузера |
