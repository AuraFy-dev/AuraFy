use async_trait::async_trait;

use domain::Track;

use crate::ImportError;

/// A parsed-but-unmatched playlist, as imported from an external service.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RawPlaylist {
    pub source_service: String,
    pub title: String,
    pub description: Option<String>,
    pub owner: Option<String>,
    /// Original playlist URL or file name, for the report.
    pub origin: String,
    pub tracks: Vec<RawTrackEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RawTrackEntry {
    pub title: String,
    pub artists: Vec<String>,
    pub album: Option<String>,
    pub duration_secs: Option<f64>,
    pub isrc: Option<String>,
    /// Direct media URL if the source provides one (M3U/XSPF).
    pub media_url: Option<String>,
}

impl From<RawTrackEntry> for Track {
    fn from(entry: RawTrackEntry) -> Self {
        Track {
            id: domain::new_id(),
            title: entry.title,
            artists: entry.artists,
            album: entry.album,
            duration_secs: entry.duration_secs,
            artwork_url: None,
            isrc: entry.isrc,
            external_ids: Vec::new(),
            source: "import".into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaylistSourceType {
    M3U,
    M3U8,
    Xspf,
    Csv,
    Json,
    Url,
}

impl PlaylistSourceType {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "m3u" => Some(Self::M3U),
            "m3u8" => Some(Self::M3U8),
            "xspf" => Some(Self::Xspf),
            "csv" => Some(Self::Csv),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

/// One importable playlist source. Implement this trait to support a
/// new service or format; register it in `PlaylistImporter`.
#[async_trait]
pub trait PlaylistSource: Send + Sync {
    /// Which service/format this source handles, e.g. "spotify-csv".
    fn name(&self) -> &str;

    /// Can this source handle the given input (file path or URL)?
    fn accepts(&self, input: &str) -> bool;

    async fn load(&self, input: &str) -> Result<RawPlaylist, ImportError>;
}
