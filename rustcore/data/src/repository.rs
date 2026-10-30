use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

use domain::{Playlist, PlaylistTrack, Track};

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub struct Repository {
    pool: SqlitePool,
}

impl Repository {
    pub async fn open(path: &str) -> Result<Self, RepoError> {
        let opts = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(true);
        // Each pooled connection to ":memory:" gets its own private database,
        // so migrations would land on only one of them. Force a single
        // connection in that case.
        let max = if path == ":memory:" { 1 } else { 5 };
        let pool = SqlitePoolOptions::new().max_connections(max).connect_with(opts).await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), RepoError> {
        let sql = include_str!("../../migrations/0001_init.sql");
        for statement in sql.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed).execute(&self.pool).await?;
            }
        }
        Ok(())
    }

    pub async fn upsert_track(&self, track: &Track) -> Result<(), RepoError> {
        sqlx::query(
            "INSERT INTO tracks (id, title, artists, album, duration_secs, artwork_url, isrc, source, external_ids)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET title=excluded.title, artists=excluded.artists,
             album=excluded.album, duration_secs=excluded.duration_secs,
             artwork_url=excluded.artwork_url, isrc=excluded.isrc, source=excluded.source,
             external_ids=excluded.external_ids",
        )
        .bind(&track.id)
        .bind(&track.title)
        .bind(serde_json::to_string(&track.artists)?)
        .bind(&track.album)
        .bind(track.duration_secs)
        .bind(&track.artwork_url)
        .bind(&track.isrc)
        .bind(&track.source)
        .bind(serde_json::to_string(&track.external_ids)?)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_track(&self, id: &str) -> Result<Option<Track>, RepoError> {
        let row: Option<TrackRow> = sqlx::query_as(
            "SELECT id, title, artists, album, duration_secs, artwork_url, isrc, source, external_ids FROM tracks WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(TryInto::try_into).transpose().map_err(Into::into)
    }

    pub async fn search_catalog(&self, query: &str, limit: u32) -> Result<Vec<Track>, RepoError> {
        let pattern = format!("%{}%", query.to_lowercase());
        let rows: Vec<TrackRow> = sqlx::query_as(
            "SELECT id, title, artists, album, duration_secs, artwork_url, isrc, source, external_ids
             FROM tracks WHERE lower(title) LIKE ? OR lower(artists) LIKE ? LIMIT ?",
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(TryInto::try_into).collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub async fn create_playlist(&self, title: &str) -> Result<String, RepoError> {
        let id = domain::new_id();
        sqlx::query("INSERT INTO playlists (id, title, source) VALUES (?, ?, 'user')")
            .bind(&id)
            .bind(title)
            .execute(&self.pool)
            .await?;
        Ok(id)
    }

    pub async fn add_track_to_playlist(
        &self,
        playlist_id: &str,
        track_id: &str,
        position: u32,
    ) -> Result<(), RepoError> {
        sqlx::query(
            "INSERT INTO playlist_tracks (playlist_id, track_id, position) VALUES (?, ?, ?)",
        )
        .bind(playlist_id)
        .bind(track_id)
        .bind(position)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_playlist(&self, id: &str) -> Result<Option<Playlist>, RepoError> {
        let playlist: Option<(String, String, Option<String>, Option<String>, String)> =
            sqlx::query_as("SELECT id, title, description, owner, source FROM playlists WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;
        let Some((pid, title, description, owner, source)) = playlist else {
            return Ok(None);
        };
        let rows: Vec<TrackRow> = sqlx::query_as(
            "SELECT t.id, t.title, t.artists, t.album, t.duration_secs, t.artwork_url, t.isrc, t.source, t.external_ids
             FROM playlist_tracks pt JOIN tracks t ON t.id = pt.track_id
             WHERE pt.playlist_id = ? ORDER BY pt.position",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;
        let tracks: Vec<PlaylistTrack> = rows
            .into_iter()
            .enumerate()
            .map(|(i, row)| {
                let track: Track = row.try_into().unwrap();
                PlaylistTrack {
                    position: i as u32,
                    track,
                }
            })
            .collect();
        Ok(Some(Playlist {
            id: pid,
            title,
            description,
            owner,
            tracks,
            source,
        }))
    }

    pub async fn toggle_like(&self, track_id: &str, liked: bool) -> Result<(), RepoError> {
        if liked {
            sqlx::query("INSERT OR IGNORE INTO likes (track_id) VALUES (?)")
                .bind(track_id)
                .execute(&self.pool)
                .await?;
        } else {
            sqlx::query("DELETE FROM likes WHERE track_id = ?")
                .bind(track_id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    pub async fn liked_track_ids(&self) -> Result<Vec<String>, RepoError> {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT track_id FROM likes ORDER BY liked_at DESC").fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    pub async fn record_listen(&self, track_id: &str, position_secs: f64) -> Result<(), RepoError> {
        sqlx::query("INSERT INTO listen_history (track_id, position_secs) VALUES (?, ?)")
            .bind(track_id)
            .bind(position_secs)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn most_played(&self, limit: u32) -> Result<Vec<Track>, RepoError> {
        let rows: Vec<TrackRow> = sqlx::query_as(
            "SELECT t.id, t.title, t.artists, t.album, t.duration_secs, t.artwork_url, t.isrc, t.source, t.external_ids
             FROM (SELECT track_id, COUNT(*) c FROM listen_history GROUP BY track_id ORDER BY c DESC LIMIT ?) h
             JOIN tracks t ON t.id = h.track_id",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(TryInto::try_into).collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub async fn all_tracks(&self) -> Result<Vec<Track>, RepoError> {
        let rows: Vec<TrackRow> = sqlx::query_as(
            "SELECT id, title, artists, album, duration_secs, artwork_url, isrc, source, external_ids FROM tracks",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(TryInto::try_into).collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
}

#[derive(sqlx::FromRow)]
struct TrackRow {
    id: String,
    title: String,
    artists: String,
    album: Option<String>,
    duration_secs: Option<f64>,
    artwork_url: Option<String>,
    isrc: Option<String>,
    source: String,
    external_ids: String,
}

impl TryFrom<TrackRow> for Track {
    type Error = serde_json::Error;

    fn try_from(row: TrackRow) -> Result<Self, Self::Error> {
        Ok(Track {
            id: row.id,
            title: row.title,
            artists: serde_json::from_str(&row.artists)?,
            album: row.album,
            duration_secs: row.duration_secs,
            artwork_url: row.artwork_url,
            isrc: row.isrc,
            external_ids: serde_json::from_str(&row.external_ids)?,
            source: row.source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn track_and_playlist_roundtrip() {
        let repo = Repository::open(":memory:").await.unwrap();
        repo.migrate().await.unwrap();
        let track = Track {
            id: "t1".into(),
            title: "Enter Sandman".into(),
            artists: vec!["Metallica".into()],
            album: Some("Metallica".into()),
            duration_secs: Some(331.0),
            artwork_url: None,
            isrc: None,
            external_ids: Vec::new(),
            source: "local".into(),
        };
        repo.upsert_track(&track).await.unwrap();
        let loaded = repo.get_track("t1").await.unwrap().unwrap();
        assert_eq!(loaded.title, "Enter Sandman");

        let pid = repo.create_playlist("Rock").await.unwrap();
        repo.add_track_to_playlist(&pid, "t1", 0).await.unwrap();
        let pl = repo.get_playlist(&pid).await.unwrap().unwrap();
        assert_eq!(pl.tracks.len(), 1);
        assert_eq!(pl.tracks[0].track.title, "Enter Sandman");

        repo.toggle_like("t1", true).await.unwrap();
        assert_eq!(repo.liked_track_ids().await.unwrap(), vec!["t1".to_string()]);
        repo.record_listen("t1", 120.0).await.unwrap();
        assert_eq!(repo.most_played(5).await.unwrap().len(), 1);
    }
}
