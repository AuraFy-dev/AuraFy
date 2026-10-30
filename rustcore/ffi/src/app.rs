use std::sync::{Arc, Mutex};

use domain::MusicProvider;
use providers::{LocalLibraryProvider, MusicBrainzProvider, ProviderRegistry};

use crate::runtime;

/// Flat track record exposed to Kotlin (UniFFI-friendly).
#[derive(uniffi::Record)]
pub struct TrackDto {
    pub id: String,
    pub title: String,
    pub artists: String,
    pub album: Option<String>,
    pub duration_secs: Option<f64>,
    pub is_offline_available: bool,
    pub source: String,
}

impl From<&domain::Track> for TrackDto {
    fn from(t: &domain::Track) -> Self {
        Self {
            id: t.id.clone(),
            title: t.title.clone(),
            artists: t.artist_names(),
            album: t.album.clone(),
            duration_secs: t.duration_secs,
            is_offline_available: t.source == "local",
            source: t.source.clone(),
        }
    }
}

/// High-level facade owned by Kotlin as a single `AurafyApp` instance.
/// Shared behind `Arc<Mutex<..>>` so it can be used from any thread.
#[derive(uniffi::Object)]
pub struct AurafyApp {
    registry: Mutex<ProviderRegistry>,
    metadata_search: MusicBrainzProvider,
}

#[uniffi::export]
impl AurafyApp {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            registry: Mutex::new(ProviderRegistry::new()),
            metadata_search: MusicBrainzProvider::new("AuraFy".into(), "contact@example.com".into())
                .unwrap_or_else(|_| MusicBrainzProvider::offline()),
        }
    }

    /// Register the user's own local music folder as a provider (legal content).
    pub fn add_local_library(&self, path: String) {
        self.registry
            .lock()
            .expect("registry poisoned")
            .register(Arc::new(LocalLibraryProvider::new(path)));
    }

    /// Search local providers (user's libraries). Blocking bridge over tokio;
    /// call from a background thread on the Kotlin side.
    pub fn search(&self, query: String, limit: u32) -> Vec<TrackDto> {
        let results = {
            let registry = self.registry.lock().expect("registry poisoned");
            runtime().block_on(registry.search_all(&query, limit))
        };
        results
            .into_iter()
            .flat_map(|r| r.tracks)
            .map(|t| TrackDto::from(&t))
            .collect()
    }

    /// Search the MusicBrainz open metadata catalogue (no audio).
    pub fn search_metadata(&self, query: String, limit: u32) -> Vec<TrackDto> {
        let results = runtime().block_on(self.metadata_search.search(&query, limit));
        match results {
            Ok(r) => r.tracks.iter().map(TrackDto::from).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Resolve a playable URL for a track. Returns `None` when the provider
    /// cannot serve the track (not configured, offline, expired).
    pub fn resolve_stream_url(&self, track_id: String) -> Option<String> {
        let registry = self.registry.lock().expect("registry poisoned");
        runtime().block_on(async {
            let track = domain::Track {
                id: track_id.clone(),
                title: String::new(),
                artists: Vec::new(),
                album: None,
                duration_secs: None,
                artwork_url: None,
                isrc: None,
                external_ids: Vec::new(),
                source: track_id
                    .split(':')
                    .next()
                    .unwrap_or_default()
                    .to_string(),
            };
            registry
                .resolve_stream(&track)
                .await
                .ok()
                .map(|s| s.url)
        })
    }

    pub fn provider_count(&self) -> u32 {
        let registry = self.registry.lock().expect("registry poisoned");
        registry.len() as u32
    }
}

impl Default for AurafyApp {
    fn default() -> Self {
        Self::new()
    }
}
