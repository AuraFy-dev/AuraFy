-- AuraFy core schema (SQLite via sqlx)
CREATE TABLE IF NOT EXISTS tracks (
    id            TEXT PRIMARY KEY,
    title         TEXT NOT NULL,
    artists       TEXT NOT NULL,            -- JSON array
    album         TEXT,
    duration_secs REAL,
    artwork_url   TEXT,
    isrc          TEXT,
    source        TEXT NOT NULL,
    external_ids  TEXT NOT NULL DEFAULT '[]', -- JSON array
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_tracks_source ON tracks(source);
CREATE INDEX IF NOT EXISTS idx_tracks_isrc ON tracks(isrc);

CREATE TABLE IF NOT EXISTS playlists (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    description TEXT,
    owner       TEXT,
    source      TEXT NOT NULL DEFAULT 'user',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS playlist_tracks (
    playlist_id TEXT NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    track_id    TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    position    INTEGER NOT NULL,
    PRIMARY KEY (playlist_id, track_id, position)
);
CREATE INDEX IF NOT EXISTS idx_pt_playlist ON playlist_tracks(playlist_id, position);

CREATE TABLE IF NOT EXISTS likes (
    track_id    TEXT PRIMARY KEY REFERENCES tracks(id) ON DELETE CASCADE,
    liked_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS listen_history (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    track_id   TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    played_at  TEXT NOT NULL DEFAULT (datetime('now')),
    position_secs REAL NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_history_track ON listen_history(track_id);

CREATE TABLE IF NOT EXISTS offline_cache (
    track_id     TEXT PRIMARY KEY REFERENCES tracks(id) ON DELETE CASCADE,
    file_path    TEXT NOT NULL,
    size_bytes   INTEGER NOT NULL,
    downloaded_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS import_reports (
    id         TEXT PRIMARY KEY,
    origin     TEXT NOT NULL,
    source_service TEXT NOT NULL,
    found      INTEGER NOT NULL,
    not_found  INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sync_state (
    device_id  TEXT PRIMARY KEY,
    last_synced_at TEXT NOT NULL,
    entity     TEXT NOT NULL,
    checksum   TEXT
);
