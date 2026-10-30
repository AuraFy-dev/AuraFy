use unicode_normalization::UnicodeNormalization;

use crate::entities::Track;

/// Normalize a string for fuzzy matching: case-fold, strip accents,
/// drop punctuation and collapse whitespace.
pub fn normalize(input: &str) -> String {
    let folded: String = input
        .nfkd()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .flat_map(|c| c.to_lowercase())
        .collect();
    let mut out = String::with_capacity(folded.len());
    let mut last_space = true;
    for c in folded.chars() {
        if c.is_whitespace() {
            if !last_space {
                out.push(' ');
                last_space = true;
            }
        } else {
            out.push(c);
            last_space = false;
        }
    }
    out.trim().to_string()
}

/// Score how well `candidate` matches the query track, in 0.0..=1.0.
/// Weights: title 0.5, artists 0.35, duration 0.15.
pub fn match_score(query: &Track, candidate: &Track) -> f64 {
    if let (Some(q), Some(c)) = (&query.isrc, &candidate.isrc) {
        if q.eq_ignore_ascii_case(c) {
            return 1.0;
        }
    }

    let q_title = normalize(&query.title);
    let c_title = normalize(&candidate.title);
    let title_sim = strsim::jaro_winkler(&q_title, &c_title).clamp(0.0, 1.0);

    let artist_sim = best_artist_similarity(&query.artists, &candidate.artists);

    let duration_sim = match (query.duration_secs, candidate.duration_secs) {
        (Some(q), Some(c)) => {
            let diff = (q - c).abs();
            (1.0 - (diff - 3.0).max(0.0) / 27.0).clamp(0.0, 1.0)
        }
        _ => 0.5,
    };

    0.5 * title_sim + 0.35 * artist_sim + 0.15 * duration_sim
}

fn best_artist_similarity(query: &[String], candidates: &[String]) -> f64 {
    if query.is_empty() || candidates.is_empty() {
        return 0.0;
    }
    let mut best = 0.0;
    for q in query {
        let qn = normalize(q);
        for c in candidates {
            let cn = normalize(c);
            let sim = strsim::jaro_winkler(&qn, &cn).clamp(0.0, 1.0);
            if sim > best {
                best = sim;
            }
        }
    }
    best
}

/// Pick the best candidate for a query track, if any exceeds `threshold`.
pub fn best_match<'a>(
    query: &Track,
    candidates: &'a [Track],
    threshold: f64,
) -> Option<(&'a Track, f64)> {
    candidates
        .iter()
        .map(|c| (c, match_score(query, c)))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .filter(|(_, score)| *score >= threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(title: &str, artists: &[&str], duration: Option<f64>) -> Track {
        Track {
            id: "t".into(),
            title: title.into(),
            artists: artists.iter().map(|s| s.to_string()).collect(),
            album: None,
            duration_secs: duration,
            artwork_url: None,
            isrc: None,
            external_ids: Vec::new(),
            source: "test".into(),
        }
    }

    #[test]
    fn normalizes_unicode_and_case() {
        assert_eq!(normalize("Metallica — Enter Sándman"), "metallica enter sandman");
        assert_eq!(normalize("  Bon  Jovi "), "bon jovi");
    }

    #[test]
    fn same_track_scores_high() {
        let a = track("Enter Sandman", &["Metallica"], Some(331.0));
        let b = track("Enter Sandman (Remastered)", &["METALLICA"], Some(330.0));
        assert!(match_score(&a, &b) > 0.85);
    }

    #[test]
    fn isrc_overrides_everything() {
        let mut a = track("Totally Different Title", &["Someone"], Some(100.0));
        let mut b = track("Another Title", &["Else"], Some(999.0));
        a.isrc = Some("USWB19900912".into());
        b.isrc = Some("uswb19900912".into());
        assert_eq!(match_score(&a, &b), 1.0);
    }

    #[test]
    fn best_match_respects_threshold() {
        let query = track("Nothing Else Matters", &["Metallica"], Some(388.0));
        let pool = vec![
            track("Smells Like Teen Spirit", &["Nirvana"], Some(301.0)),
            track("Nothing Else Matters", &["Metallica"], Some(388.0)),
        ];
        let (best, score) = best_match(&query, &pool, 0.8).unwrap();
        assert_eq!(best.title, "Nothing Else Matters");
        assert!(score > 0.95);
        // An identical candidate legitimately scores 1.0, so even a very high
        // threshold must match it.
        assert!(best_match(&query, &pool, 0.995).is_some());
        // A dissimilar-only pool must fall below a moderate threshold.
        let dissimilar = vec![track("Smells Like Teen Spirit", &["Nirvana"], Some(301.0))];
        assert!(best_match(&query, &dissimilar, 0.5).is_none());
    }
}
