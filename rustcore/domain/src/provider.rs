use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::entities::{Album, Artist, Playlist, Track};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResults {
    pub provider: String,
    pub query: String,
    pub tracks: Vec<Track>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
    pub playlists: Vec<Playlist>,
}

impl SearchResults {
    pub fn empty(provider: &str, query: &str) -> Self {
        Self {
            provider: provider.to_string(),
            query: query.to_string(),
            tracks: Vec::new(),
            albums: Vec::new(),
            artists: Vec::new(),
            playlists: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamSource {
    pub url: String,
    pub mime: Option<String>,
    pub is_hls: bool,
    pub is_dash: bool,
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProviderCapabilities {
    pub search: bool,
    pub stream: bool,
    pub playlists: bool,
    pub offline_download: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub capabilities: ProviderCapabilities,
    pub is_authenticated: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("network error: {0}")]
    Network(String),
    #[error("authentication required for provider {0}")]
    AuthRequired(String),
    #[error("rate limited by provider {0}")]
    RateLimited(String),
    #[error("content not available: {0}")]
    NotAvailable(String),
    #[error("invalid response from {0}")]
    InvalidResponse(String),
    #[error("configuration error: {0}")]
    Config(String),
}

impl From<reqwest::Error> for ProviderError {
    fn from(err: reqwest::Error) -> Self {
        ProviderError::Network(err.to_string())
    }
}

/// A pluggable music source. Users connect their own legal sources;
/// the app ships no built-in access to copyrighted catalogs.
#[async_trait]
pub trait MusicProvider: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn capabilities(&self) -> ProviderCapabilities;

    async fn search(&self, query: &str, limit: u32) -> Result<SearchResults, ProviderError>;

    async fn resolve_stream(&self, track: &Track) -> Result<StreamSource, ProviderError>;

    async fn get_playlist(&self, reference: &str) -> Result<Playlist, ProviderError> {
        let _ = reference;
        Err(ProviderError::NotAvailable(format!(
            "provider {} does not expose playlists",
            self.id()
        )))
    }

    async fn provider_info(&self) -> Result<ProviderInfo, ProviderError>;
}
