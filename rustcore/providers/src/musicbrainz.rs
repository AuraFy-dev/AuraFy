use std::time::Duration;

use async_trait::async_trait;
use domain::{
    MusicProvider, Playlist, ProviderCapabilities, ProviderError, ProviderInfo,
    SearchResults, StreamSource, Track,
};
use serde::Deserialize;

const BASE_URL: &str = "https://musicbrainz.org/ws/2";

/// MusicBrainz metadata provider (open data, CC0). Metadata only:
/// it never returns audio streams.
#[derive(Clone)]
pub struct MusicBrainzProvider {
    client: reqwest::Client,
    app_name: String,
    contact: String,
}

#[derive(Debug, Deserialize)]
struct MbRecording {
    id: String,
    title: String,
    length: Option<u64>,
    #[serde(default)]
    artist_credit: Vec<MbArtistCredit>,
    #[serde(default)]
    releases: Vec<MbRelease>,
}

#[derive(Debug, Deserialize)]
struct MbArtistCredit {
    name: String,
}

#[derive(Debug, Deserialize)]
struct MbRelease {
    id: String,
    title: String,
    #[serde(default)]
    media: Vec<MbMedium>,
}

#[derive(Debug, Deserialize)]
struct MbMedium {
    #[serde(default)]
    track_count: u32,
}

#[derive(Debug, Deserialize)]
struct MbRecordingSearch {
    #[serde(default)]
    recordings: Vec<MbRecording>,
}

#[derive(Debug, Deserialize)]
struct MbArtist {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MbArtistList {
    #[serde(default)]
    artists: Vec<MbArtist>,
}

#[derive(Debug, Deserialize)]
struct MbReleaseList {
    #[serde(default)]
    releases: Vec<MbRelease>,
}

impl MusicBrainzProvider {
    /// MusicBrainz asks for a meaningful User-Agent (app name + contact).
    pub fn new(app_name: String, contact: String) -> Result<Self, ProviderError> {
        let ua = format!("{}/0.1 ({})", app_name, contact);
        let client = reqwest::Client::builder()
            .user_agent(ua)
            .timeout(Duration::from_secs(15))
            .build()?;
        Ok(Self {
            client,
            app_name,
            contact,
        })
    }

    /// Placeholder used when the HTTP client cannot be built (e.g. no TLS
    /// backend): every search then returns an error the registry swallows.
    pub fn offline() -> Self {
        Self {
            client: reqwest::Client::new(),
            app_name: String::new(),
            contact: String::new(),
        }
    }

    fn recordings_url(query: &str, limit: u32) -> String {
        format!(
            "{}/recording?query=urlencoded:{}&fmt=json&limit={}",
            BASE_URL,
            urlencoding::encode(query),
            limit.min(100)
        )
    }
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut out = String::with_capacity(s.len() * 2);
        for b in s.bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    out.push(b as char)
                }
                _ => out.push_str(&format!("%{:02X}", b)),
            }
        }
        out
    }
}

#[async_trait]
impl MusicProvider for MusicBrainzProvider {
    fn id(&self) -> &str {
        "musicbrainz"
    }

    fn name(&self) -> &str {
        "MusicBrainz"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            search: true,
            stream: false,
            playlists: false,
            offline_download: false,
        }
    }

    async fn search(&self, query: &str, limit: u32) -> Result<SearchResults, ProviderError> {
        if query.trim().is_empty() {
            return Ok(SearchResults::empty(self.id(), query));
        }
        let url = Self::recordings_url(query, limit);
        let resp: MbRecordingSearch = self
            .client
            .get(&url)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| {
                if e.status().map(|s| s.as_u16()) == Some(503) {
                    ProviderError::RateLimited(self.id().into())
                } else {
                    ProviderError::Network(e.to_string())
                }
            })?
            .json()
            .await?;

        let tracks = resp
            .recordings
            .into_iter()
            .take(limit as usize)
            .map(|r| Track {
                id: format!("mb:{}", r.id),
                title: r.title,
                artists: r.artist_credit.into_iter().map(|a| a.name).collect(),
                album: r.releases.first().map(|rel| rel.title.clone()),
                duration_secs: r.length.map(|ms| ms as f64 / 1000.0),
                artwork_url: None,
                isrc: None,
                external_ids: vec![domain::ExternalId {
                    provider: "musicbrainz".into(),
                    id: r.id.clone(),
                    url: Some(format!("https://musicbrainz.org/recording/{}", r.id)),
                }],
                source: "musicbrainz".into(),
            })
            .collect();

        Ok(SearchResults {
            provider: self.id().into(),
            query: query.into(),
            tracks,
            albums: Vec::new(),
            artists: Vec::new(),
            playlists: Vec::new(),
        })
    }

    async fn resolve_stream(&self, _track: &Track) -> Result<StreamSource, ProviderError> {
        Err(ProviderError::NotAvailable(
            "MusicBrainz is a metadata-only source; pair it with an audio provider".into(),
        ))
    }

    async fn get_playlist(&self, _reference: &str) -> Result<Playlist, ProviderError> {
        Err(ProviderError::NotAvailable(
            "MusicBrainz does not expose playlists".into(),
        ))
    }

    async fn provider_info(&self) -> Result<ProviderInfo, ProviderError> {
        Ok(ProviderInfo {
            id: self.id().into(),
            name: self.name().into(),
            capabilities: self.capabilities(),
            is_authenticated: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_search_url() {
        let url = MusicBrainzProvider::recordings_url("Enter Sandman", 25);
        assert!(url.contains("query=urlencoded:Enter%20Sandman"));
        assert!(url.contains("limit=25"));
    }

    #[test]
    fn urlencodes_special_chars() {
        assert_eq!(urlencoding::encode("AC/DC"), "AC%2FDC");
    }
}
