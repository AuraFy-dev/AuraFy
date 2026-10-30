# Архитектура AuraFy

## Обзор

Rust-ядро (domain + data + playback + import + providers + sync) связывается с Kotlin/Jetpack Compose UI через UniFFI (proc-macro, библиотечный режим). Аудио-вывод — Media3/ExoPlayer на стороне Android; Rust отвечает за бизнес-логику, каталог, кэш, БД и (перспектива) декодеры через FFI.

```mermaid
graph TB
    subgraph Kotlin["Kotlin / Jetpack Compose"]
        UI["PlayerScreen (Compose)"]
        VM["PlayerViewModel"]
        SVC["PlaybackService (Media3 + MediaSession)"]
    end
    subgraph FFI["UniFFI (JNA)"]
        B["uniffi.aurafy_ffi.kt"]
    end
    subgraph Rust["rustcore (workspace)"]
        FFI_C["aurafy-ffi (AurafyApp)"]
        DOM["domain (entities, matching)"]
        PROV["providers (registry, local, musicbrainz)"]
        IMP["playlist_import (parsers, importer)"]
        DATA["data (sqlx + SQLite)"]
        PLAY["playback (crypto, offline_cache)"]
        SYNC["sync (axum-сервер)"]
    end
    UI --> VM
    VM --> B
    SVC --> B
    B --> FFI_C
    FFI_C --> DOM
    FFI_C --> PROV --> DOM
    FFI_C --> IMP
    FFI_C --> DATA
    FFI_C --> PLAY
    FFI_C --> SYNC
```

## Слои

| Слой | Крейт | Ответственность |
|------|-------|-----------------|
| Domain | `domain` | `Track`, `Playlist`, `MusicProvider` trait, fuzzy-matching (strsim + Unicode NFKD-нормализация) |
| Data | `data` | SQLite через sqlx, миграции, репозиторий |
| Providers | `providers` | `ProviderRegistry`, `LocalLibraryProvider` (файлы пользователя), `MusicBrainzProvider` (открытые метаданные) |
| Import | `playlist_import` | `PlaylistSource` trait, парсеры M3U/XSPF/CSV/JSON, сопоставление, отчёты |
| Playback | `playback` | AES-GCM шифрование оффлайн-кэша |
| Sync | `sync` | REST-сервер на axum для синхронизации между устройствами |
| FFI | `aurafy-ffi` | `AurafyApp` — facade для Kotlin: `search`, `searchMetadata`, `resolveStreamUrl`, `addLocalLibrary` |

## Ключевые решения

- **UI: Kotlin + Compose (не Flutter).** Media3/ExoPlayer, MediaSession (lock-screen, Bluetooth, Android Auto) — first-class Android API; FFI через JNA проще, чем через dart:ffi + плагины; Rust-ядро остаётся переиспользуемым (iOS через Swift-биндинги UniFFI).
- **UniFFI proc-macro + library mode.** Биндинги генерируются из собранной cdylib (`ffi/src/bin/uniffi-bindgen.rs`), UDL не нужен.
- **Блокирующий мост в FFI.** `AurafyApp` бриджит tokio-runtime; на Kotlin вызовы оборачиваются в `Dispatchers.IO`.
- **Провайдеры сменные.** Пользователь подключает свои легальные источники (локальные файлы, свой сервер, API с подпиской). Встроены только `local` и `musicbrainz` (метаданные, без аудио).

## Схема БД (SQLite, `migrations/0001_init.sql`)

- `tracks` — каталог треков (artists/external_ids как JSON)
- `playlists`, `playlist_tracks` (position)
- `likes`, `listen_history`
- `offline_cache` (зашифрованные файлы)
- `import_reports`, `sync_state`

## Юридическая рамка

AuraFy **не распространяет защищённый контент**. Все аудио-источники подключает пользователь и несёт ответственность за их легальность. Встроенный провайдер — только локальные файлы пользователя и открытые метаданные MusicBrainz (CC0).
