use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artists: Vec<String>,
    pub album: Option<String>,
    pub duration_secs: Option<f64>,
    pub artwork_url: Option<String>,
    pub isrc: Option<String>,
    pub external_ids: Vec<ExternalId>,
    pub source: String,
}

impl Track {
    pub fn artist_names(&self) -> String {
        self.artists.join(", ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExternalId {
    pub provider: String,
    pub id: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub year: Option<i32>,
    pub artwork_url: Option<String>,
    pub track_count: u32,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub artwork_url: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Playlist {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub owner: Option<String>,
    pub tracks: Vec<PlaylistTrack>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlaylistTrack {
    pub position: u32,
    pub track: Track,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayQueueItem {
    pub track: Track,
    pub stream_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackState {
    pub track_id: Option<String>,
    pub position_secs: f64,
    pub duration_secs: Option<f64>,
    pub is_playing: bool,
    pub queue_index: Option<u32>,
    pub queue_length: u32,
    pub updated_at: DateTime<Utc>,
}

pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}
