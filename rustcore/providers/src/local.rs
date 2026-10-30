use std::path::{Path, PathBuf};

use async_trait::async_trait;
use domain::{
    MusicProvider, ProviderCapabilities, ProviderError, ProviderInfo, SearchResults,
    StreamSource, Track,
};

/// Scans user's own local music files (user-provided content — fully legal).
#[derive(Clone)]
pub struct LocalLibraryProvider {
    library_dir: PathBuf,
}

impl LocalLibraryProvider {
    pub fn new(library_dir: impl Into<PathBuf>) -> Self {
        Self {
            library_dir: library_dir.into(),
        }
    }

    fn supported_extensions() -> [&'static str; 6] {
        ["mp3", "m4a", "ogg", "flac", "opus", "wav"]
    }

    fn scan_files(&self) -> Result<Vec<(PathBuf, String)>, ProviderError> {
        let mut out = Vec::new();
        let dir = self
            .library_dir
            .to_str()
            .ok_or_else(|| ProviderError::Config("non-UTF8 library path".into()))?;
        walk(dir, &mut out)?;
        Ok(out)
    }

    fn parse_metadata(filename: &str) -> (String, Vec<String>) {
        // "01 - Artist - Title.mp3" or "Artist - Title.mp3"
        let stem = filename.rsplit_once('.').map(|(s, _)| s).unwrap_or(filename);
        let parts: Vec<&str> = stem.splitn(3, " - ").collect();
        match parts.as_slice() {
            [track_no, artist, title] if track_no.chars().all(|c| c.is_ascii_digit()) && !track_no.is_empty() => {
                (title.trim().to_string(), vec![artist.trim().to_string()])
            }
            [artist, title] if !artist.trim().is_empty() => {
                (title.trim().to_string(), vec![artist.trim().to_string()])
            }
            _ => (stem.trim().to_string(), Vec::new()),
        }
    }

    fn file_to_track(path: &Path) -> Option<Track> {
        let filename = path.file_name()?.to_str()?;
        let (title, artists) = Self::parse_metadata(filename);
        Some(Track {
            id: format!("local:{}", path.to_string_lossy()),
            title,
            artists,
            album: None,
            duration_secs: None,
            artwork_url: None,
            isrc: None,
            external_ids: Vec::new(),
            source: "local".into(),
        })
    }
}

fn walk(dir: &str, out: &mut Vec<(PathBuf, String)>) -> Result<(), ProviderError> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| ProviderError::Config(format!("cannot read {}: {}", dir, e)))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(sub) = path.to_str() {
                walk(sub, out)?;
            }
        } else {
            let ext_ok = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| {
                    LocalLibraryProvider::supported_extensions()
                        .iter()
                        .any(|&ok| ok.eq_ignore_ascii_case(e))
                })
                .unwrap_or(false);
            if ext_ok {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                out.push((path, name));
            }
        }
    }
    Ok(())
}

#[async_trait]
impl MusicProvider for LocalLibraryProvider {
    fn id(&self) -> &str {
        "local"
    }

    fn name(&self) -> &str {
        "Локальная библиотека"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            search: true,
            stream: true,
            playlists: false,
            offline_download: false,
        }
    }

    async fn search(&self, query: &str, limit: u32) -> Result<SearchResults, ProviderError> {
        let norm = domain::normalize(query);
        let files = self.scan_files()?;
        let tracks: Vec<Track> = files
            .iter()
            .filter_map(|(path, _)| Self::file_to_track(path))
            .filter(|t| {
                let hay = format!("{} {}", t.title, t.artist_names());
                domain::normalize(&hay).contains(&norm) && !norm.is_empty()
            })
            .take(limit as usize)
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

    async fn resolve_stream(&self, track: &Track) -> Result<StreamSource, ProviderError> {
        let path = track
            .id
            .strip_prefix("local:")
            .ok_or_else(|| ProviderError::NotAvailable("not a local track".into()))?;
        if !Path::new(path).exists() {
            return Err(ProviderError::NotAvailable("file no longer exists".into()));
        }
        Ok(StreamSource {
            url: format!("file://{}", path),
            mime: None,
            is_hls: false,
            is_dash: false,
            expires_at: None,
        })
    }

    async fn provider_info(&self) -> Result<ProviderInfo, ProviderError> {
        let count = self.scan_files().map(|v| v.len()).unwrap_or(0);
        Ok(ProviderInfo {
            id: self.id().into(),
            name: self.name().into(),
            capabilities: self.capabilities(),
            is_authenticated: count > 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_artist_title_filenames() {
        let (t, a) = LocalLibraryProvider::parse_metadata("01 - Metallica - Enter Sandman.mp3");
        assert_eq!(t, "Enter Sandman");
        assert_eq!(a, vec!["Metallica".to_string()]);
        let (t, a) = LocalLibraryProvider::parse_metadata("Aerosmith - Dream On.mp3");
        assert_eq!(t, "Dream On");
        assert_eq!(a, vec!["Aerosmith".to_string()]);
        let (t, a) = LocalLibraryProvider::parse_metadata("Song.flac");
        assert_eq!(t, "Song");
        assert!(a.is_empty());
    }

    #[tokio::test]
    async fn resolves_local_file_stream() {
        let dir = std::env::temp_dir().join("aurafy_test_lib");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("Test - Song.mp3");
        std::fs::write(&file, b"id3").unwrap();
        let provider = LocalLibraryProvider::new(&dir);
        let track = LocalLibraryProvider::file_to_track(&file).unwrap();
        let stream = provider.resolve_stream(&track).await.unwrap();
        assert!(stream.url.starts_with("file://"));
        std::fs::remove_file(&file).ok();
    }
}
