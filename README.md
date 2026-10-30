# AuraFy

Кроссплатформенное (Android-first) музыкальное приложение: Rust-ядро + Kotlin/Jetpack Compose UI.

## ⚠️ Юридические ограничения (обязательно к прочтению)

**AuraFy не распространяет защищённый авторским правом контент и не содержит встроенного доступа к пиратским каталогам.** Прямое распространение/стриминг защищённой музыки без лицензии незаконно.

Приложение спроектировано как плеер-агрегатор со сменными провайдерами:

- **Легальные источники подключает пользователь сам**: локальные файлы (собственная библиотека), собственный стриминговый сервер, легальные API с подпиской (если у провайдера есть публичный API и вы соблюдаете его ToS).
- Встроенные провайдеры: `local` (локальные файлы) и `musicbrainz` (открытые метаданные, CC0 — без аудио).
- В UI (экран «Источники») отображается предупреждение о легальности; добавление стороннего провайдера требует явного подтверждения пользователя.

## Структура

- `rustcore/` — Rust workspace: `domain`, `providers`, `playlist_import`, `playback`, `sync`, `data`, `ffi` (UniFFI).
- `android/` — Kotlin + Jetpack Compose + Media3.
- `.github/workflows/` — CI.

## Сборка

### Rust-ядро (хост-тесты)
```bash
cd rustcore
cargo test
cargo build --release
```

### Android (нужны Android SDK + NDK)
```bash
cd rustcore/ffi
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o ../../android/app/src/main/jniLibs build --release
cd ../../android
./gradlew assembleDebug   # APK: app/build/outputs/apk/debug/
./gradlew bundleRelease   # AAB: app/build/outputs/bundle/release/
```

Kotlin-биндинги UniFFI генерируются из собранной cdylib (host-сборки):
```bash
cd rustcore/ffi
cargo run --bin uniffi-bindgen --release -- \
  ../target/release/libaurafy_ffi.so \
  ../../android/app/src/main/java/
```

## Импорт плейлистов

Форматы: M3U/M3U8, XSPF, CSV, JSON, ссылки. Парсеры — в `rustcore/import`; новые сервисы добавляются реализацией трейта `PlaylistSource`. Сопоставление — fuzzy (название/артист/длительность + ISRC), с отчётом найдено/не найдено и ручным маппингом.

## Документация

- `docs/architecture.md` — архитектура и схема БД
- `docs/roadmap.md` — план по спринтам (MVP → v1)

## Статус: MVP-каркас (сборка зелёная)

- Rust-ядро: `cargo test --workspace` — 22 теста, все проходят
- Android: полный набор файлов сборки (gradle wrapper, иконка, proguard, JNA, UniFFI-биндинги)
- CI: тесты Rust + clippy, сборка APK/AAB с cargo-ndk

