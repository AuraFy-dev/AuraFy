# План разработки: MVP → v1

## Sprint 1 — MVP-каркас (готово)

- [x] Rust workspace: `domain`, `providers`, `import`, `playback`, `sync`, `data`, `ffi`
- [x] `MusicProvider` trait + `ProviderRegistry` + провайдеры `local`, `musicbrainz`
- [x] Парсеры плейлистов: M3U/M3U8, XSPF, CSV, JSON + fuzzy-matching
- [x] SQLite-схема (8 таблиц) + репозиторий на sqlx
- [x] AES-GCM оффлайн-кэш
- [x] axum sync-сервер (каркас)
- [x] UniFFI proc-macro API: `AurafyApp` (search / searchMetadata / resolveStreamUrl / addLocalLibrary)
- [x] Сгенерированные Kotlin-биндинги `uniffi.aurafy_ffi`
- [x] Compose PlayerScreen + PlayerViewModel + Media3 PlaybackService
- [x] Gradle wrapper, иконка, proguard, CI (cargo test + clippy + APK/AAB)

## Sprint 2 — Воспроизведение

- [ ] Wiring PlaybackService ↔ ViewModel через MediaController (очередь, next/previous, позиция)
- [ ] Стриминг `resolveStreamUrl` → ExoPlayer (HLS-источник через HlsMediaSource)
- [ ] Фоновое воспроизведение + lock-screen (MediaSession notification)
- [ ] Скачивание трека в зашифрованный кэш + оффлайн-плейлист

## Sprint 3 — Каталог и импорт

- [ ] Экран «Источники»: подключение локальных папок (SAF), предупреждение о легальности
- [ ] Импорт плейлистов: файл-пикер (M3U/XSPF/CSV/JSON), экран отчёта, ручное сопоставление
- [ ] Интеграция `data::Repository` в FFI (persist каталога, likes, история)

## Sprint 4 — Пользовательские функции

- [ ] Плейлисты CRUD + drag&drop, «Мне нравится», история прослушиваний
- [ ] Рекомендации (content-based на Rust по listen_history)
- [ ] Sync-сервер: аккаунты,DeviceInfo, конфликты

## Sprint 5 → v1

- [ ] Эквалайзер, кроссфейд, gapless
- [ ] Android Auto
- [ ] iOS (Swift-биндинги UniFFI)
- [ ] Стор, релизная ветка CI
