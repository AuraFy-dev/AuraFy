use serde::{Deserialize, Serialize};

use domain::Track;

/// One import result entry: a raw query track and its catalog match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackMatch {
    pub query_title: String,
    pub query_artists: Vec<String>,
    pub matched_track: Option<Track>,
    pub score: f64,
}

/// Full import report: how many matched, how many need manual mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportReport {
    pub matches: Vec<TrackMatch>,
}

impl ImportReport {
    pub fn found(&self) -> usize {
        self.matches.iter().filter(|m| m.matched_track.is_some()).count()
    }

    pub fn not_found(&self) -> usize {
        self.matches.len() - self.found()
    }

    /// Tracks above this score are safe to auto-confirm in UI.
    pub fn high_confidence(&self) -> Vec<&TrackMatch> {
        self.matches.iter().filter(|m| m.score >= 0.95).collect()
    }
}
