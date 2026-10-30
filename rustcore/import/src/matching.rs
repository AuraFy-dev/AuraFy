use domain::{best_match, Track};

use crate::report::{ImportReport, TrackMatch};

/// Matches imported tracks against the local catalog using fuzzy
/// matching (title/artists/duration + ISRC override).
pub struct TrackMatcher {
    threshold: f64,
}

impl Default for TrackMatcher {
    fn default() -> Self {
        Self { threshold: 0.8 }
    }
}

impl TrackMatcher {
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    pub fn match_all(&self, raw: &[Track], catalog: &[Track]) -> ImportReport {
        let mut matches = Vec::with_capacity(raw.len());
        for query in raw {
            matches.push(match_entry(query, catalog, self.threshold));
        }
        ImportReport { matches }
    }
}

fn match_entry(query: &Track, catalog: &[Track], threshold: f64) -> TrackMatch {
    match best_match(query, catalog, threshold) {
        Some((track, score)) => TrackMatch {
            query_title: query.title.clone(),
            query_artists: query.artists.clone(),
            matched_track: Some(track.clone()),
            score,
        },
        None => TrackMatch {
            query_title: query.title.clone(),
            query_artists: query.artists.clone(),
            matched_track: None,
            score: 0.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(title: &str, artists: &[&str], duration: Option<f64>) -> Track {
        Track {
            id: domain::new_id(),
            title: title.into(),
            artists: artists.iter().map(|s| s.to_string()).collect(),
            album: None,
            duration_secs: duration,
            artwork_url: None,
            isrc: None,
            external_ids: Vec::new(),
            source: "catalog".into(),
        }
    }

    #[test]
    fn matches_and_reports_unmatched() {
        let raw = vec![
            track("Nothing Else Matters", &["Metallica"], Some(388.0)),
            track("Nonexistent Song", &["Ghost"], Some(123.0)),
        ];
        let catalog = vec![track("Nothing Else Matters", &["Metallica"], Some(388.0))];
        let report = TrackMatcher::default().match_all(&raw, &catalog);
        assert_eq!(report.found(), 1);
        assert_eq!(report.not_found(), 1);
    }
}
