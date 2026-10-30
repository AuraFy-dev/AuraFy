//! `PlaylistImporter`: dispatches input to registered `PlaylistSource`
//! implementations, then fuzzy-matches against the catalog.

use std::path::Path;
use std::sync::Arc;

use domain::Track;

use crate::parsers;
use crate::report::ImportReport;
use crate::{ImportError, PlaylistSource, RawPlaylist, TrackMatcher};

pub struct PlaylistImporter {
    sources: Vec<Arc<dyn PlaylistSource>>,
    matcher: TrackMatcher,
}

impl Default for PlaylistImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaylistImporter {
    /// Importer with built-in file-format sources.
    pub fn new() -> Self {
        let sources: Vec<Arc<dyn PlaylistSource>> =
            vec![Arc::new(FileFormatSource)];
        Self { sources, matcher: TrackMatcher::default() }
    }

    /// Register an additional source (e.g. a Spotify link fetcher).
    pub fn register_source(&mut self, source: Arc<dyn PlaylistSource>) {
        self.sources.push(source);
    }

    pub fn source_names(&self) -> Vec<String> {
        self.sources.iter().map(|s| s.name().to_string()).collect()
    }

    /// Parse a file path or URL into a raw playlist.
    pub async fn import_raw(&self, input: &str) -> Result<RawPlaylist, ImportError> {
        for source in &self.sources {
            if source.accepts(input) {
                return source.load(input).await;
            }
        }
        Err(ImportError::UnsupportedFormat(input.to_string()))
    }

    /// Import + match against the catalog in one step.
    pub async fn import(
        &self,
        input: &str,
        catalog: &[Track],
    ) -> Result<(RawPlaylist, ImportReport), ImportError> {
        let raw = self.import_raw(input).await?;
        let queries: Vec<Track> = raw.tracks.clone().into_iter().map(Into::into).collect();
        let report = self.matcher.match_all(&queries, catalog);
        Ok((raw, report))
    }
}

/// Built-in source for local files in all supported formats.
struct FileFormatSource;

#[async_trait::async_trait]
impl PlaylistSource for FileFormatSource {
    fn name(&self) -> &str {
        "file"
    }

    fn accepts(&self, input: &str) -> bool {
        let lower = input.to_ascii_lowercase();
        [".m3u", ".m3u8", ".xspf", ".csv", ".json"]
            .iter()
            .any(|ext| lower.ends_with(ext))
            && !input.starts_with("http")
    }

    async fn load(&self, input: &str) -> Result<RawPlaylist, ImportError> {
        let path = Path::new(input);
        let content = std::fs::read_to_string(path)?;
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        match ext.to_ascii_lowercase().as_str() {
            "m3u" | "m3u8" => parsers::parse_m3u(&content, input),
            "xspf" => parsers::parse_xspf(&content, input),
            "csv" => parsers::parse_csv(&content, input),
            "json" => parsers::parse_export_json(&content, input),
            _ => Err(ImportError::UnsupportedFormat(ext.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> Vec<Track> {
        vec![Track {
            id: "1".into(),
            title: "Enter Sandman".into(),
            artists: vec!["Metallica".into()],
            album: None,
            duration_secs: Some(331.0),
            artwork_url: None,
            isrc: None,
            external_ids: Vec::new(),
            source: "local".into(),
        }]
    }

    #[tokio::test]
    async fn imports_and_matches_m3u_file() {
        let dir = std::env::temp_dir().join("aurafy_import_test");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("rock.m3u");
        std::fs::write(
            &file,
            "#EXTM3U\n#EXTINF:331, Metallica - Enter Sandman\nhttp://x/1.mp3\n#EXTINF:200, Unknown - Missing Song\nhttp://x/2.mp3\n",
        )
        .unwrap();
        let importer = PlaylistImporter::new();
        let (raw, report) = importer.import(file.to_str().unwrap(), &catalog()).await.unwrap();
        assert_eq!(raw.tracks.len(), 2);
        assert_eq!(report.found(), 1);
        assert_eq!(report.not_found(), 1);
    }

    #[tokio::test]
    async fn rejects_unsupported_input() {
        let importer = PlaylistImporter::new();
        assert!(matches!(
            importer.import_raw("https://open.spotify.com/playlist/xyz").await,
            Err(ImportError::UnsupportedFormat(_))
        ));
    }
}
