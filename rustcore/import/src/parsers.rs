//! Format parsers: M3U/M3U8, XSPF, CSV, JSON.
//! Each returns a `RawPlaylist` ready for catalog matching.

use crate::source::{RawPlaylist, RawTrackEntry};
use crate::ImportError;

pub fn parse_m3u(content: &str, origin: &str) -> Result<RawPlaylist, ImportError> {
    let mut tracks = Vec::new();
    let mut pending_meta: Option<(String, f64)> = None;
    let mut title = "Imported playlist".to_string();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(ext) = line.strip_prefix("#EXTM3U") {
            let _ = ext;
            continue;
        }
        if let Some(extinf) = line.strip_prefix("#EXTINF:") {
            // #EXTINF:331.0,Artist - Title
            let (dur, rest) = extinf.split_once(',').ok_or_else(|| {
                ImportError::Malformed("EXTINF without comma".into())
            })?;
            let duration = dur.trim().parse::<f64>().ok();
            pending_meta = Some((rest.trim().to_string(), duration.unwrap_or(-1.0)));
        } else if let Some(name) = line.strip_prefix("#PLAYLIST:") {
            title = name.trim().to_string();
        } else if !line.starts_with('#') {
            let (t, a, d) = match pending_meta.take() {
                Some((label, dur)) if dur >= 0.0 => {
                    parse_artist_dash_title(&label, Some(dur))
                }
                Some((label, _)) => parse_artist_dash_title(&label, None),
                None => {
                    // No metadata: derive from filename in the URL.
                    let name = line.rsplit('/').next().unwrap_or(line);
                    let stem = name.rsplit_once('.').map(|(s, _)| s).unwrap_or(name);
                    parse_artist_dash_title(stem, None)
                }
            };
            tracks.push(RawTrackEntry {
                title: t,
                artists: a,
                album: None,
                duration_secs: d,
                isrc: None,
                media_url: Some(line.to_string()),
            });
        }
    }

    Ok(RawPlaylist {
        source_service: "m3u".into(),
        title,
        description: None,
        owner: None,
        origin: origin.into(),
        tracks,
    })
}

fn parse_artist_dash_title(label: &str, duration: Option<f64>) -> (String, Vec<String>, Option<f64>) {
    let label = label.trim();
    let parts: Vec<&str> = label.splitn(2, " - ").collect();
    match parts.as_slice() {
        [artist, title] => (
            title.trim().to_string(),
            vec![artist.trim().to_string()],
            duration,
        ),
        _ => (label.to_string(), Vec::new(), duration),
    }
}

pub fn parse_xspf(content: &str, origin: &str) -> Result<RawPlaylist, ImportError> {
    let doc = roxmltree::Document::parse(content)
        .map_err(|e| ImportError::Malformed(format!("XSPF XML error: {}", e)))?;
    let root = doc.root_element();

    let child_text = |node: roxmltree::Node, name: &str| -> Option<String> {
        node.children()
            .filter(roxmltree::Node::is_element)
            .find(|e| e.tag_name().name() == name)
            .and_then(|e| e.text())
            .map(|s| s.trim().to_string())
    };

    let title = child_text(root, "title").unwrap_or_else(|| "Imported playlist".into());

    let mut tracks = Vec::new();
    let track_list = root
        .children()
        .filter(roxmltree::Node::is_element)
        .find(|e| e.tag_name().name() == "trackList");
    if let Some(tl) = track_list {
        for tr in tl
            .children()
            .filter(roxmltree::Node::is_element)
            .filter(|e| e.tag_name().name() == "track")
        {
            let duration_ms: Option<f64> = child_text(tr, "duration")
                .and_then(|d| d.parse::<f64>().ok().map(|ms| ms / 1000.0));
            tracks.push(RawTrackEntry {
                title: child_text(tr, "title").unwrap_or_default(),
                artists: child_text(tr, "creator").map(|c| vec![c]).unwrap_or_default(),
                album: child_text(tr, "album"),
                duration_secs: duration_ms,
                isrc: None,
                media_url: child_text(tr, "location"),
            });
        }
    }

    Ok(RawPlaylist {
        source_service: "xspf".into(),
        title,
        description: None,
        owner: None,
        origin: origin.into(),
        tracks,
    })
}

pub fn parse_csv(content: &str, origin: &str) -> Result<RawPlaylist, ImportError> {
    // Expected export columns: title, artist, album, duration (seconds or mm:ss).
    let mut lines = content.lines().filter(|l| !l.trim().is_empty());
    let header = lines.next().ok_or_else(|| ImportError::Malformed("empty CSV".into()))?;
    let cols: Vec<String> = split_csv_line(header)
        .into_iter()
        .map(|c| c.trim().to_lowercase())
        .collect();
    let idx = |name: &str| cols.iter().position(|c| c == name);

    let (i_title, i_artist, i_album, i_dur) = (
        idx("title").or_else(|| idx("track")).or_else(|| idx("name")),
        idx("artist").or_else(|| idx("artists")),
        idx("album"),
        idx("duration").or_else(|| idx("duration_secs")),
    );

    let mut tracks = Vec::new();
    for line in lines {
        let fields = split_csv_line(line);
        let field = |i: Option<usize>| i.and_then(|i| fields.get(i)).cloned().unwrap_or_default();
        let title = field(i_title);
        if title.is_empty() {
            continue;
        }
        let artists: Vec<String> = field(i_artist)
            .split(&[',', ';', '|'][..])
            .map(|a| a.trim().to_string())
            .filter(|a| !a.is_empty())
            .collect();
        tracks.push(RawTrackEntry {
            title,
            artists,
            album: {
                let a = field(i_album);
                if a.is_empty() { None } else { Some(a) }
            },
            duration_secs: field(i_dur).parse::<f64>().ok(),
            isrc: None,
            media_url: None,
        });
    }

    Ok(RawPlaylist {
        source_service: "csv".into(),
        title: "Imported playlist".into(),
        description: None,
        owner: None,
        origin: origin.into(),
        tracks,
    })
}

fn split_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    for c in line.chars() {
        match c {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                fields.push(cur.trim().to_string());
                cur = String::new();
            }
            _ => cur.push(c),
        }
    }
    fields.push(cur.trim().to_string());
    fields
}

pub fn parse_export_json(content: &str, origin: &str) -> Result<RawPlaylist, ImportError> {
    #[derive(serde::Deserialize)]
    struct JsonExport {
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        owner: Option<String>,
        tracks: Vec<JsonTrack>,
    }
    #[derive(serde::Deserialize)]
    struct JsonTrack {
        title: String,
        #[serde(default)]
        artist: Option<String>,
        #[serde(default)]
        artists: Option<Vec<String>>,
        #[serde(default)]
        album: Option<String>,
        #[serde(default)]
        duration: Option<f64>,
        #[serde(default)]
        isrc: Option<String>,
        #[serde(default)]
        url: Option<String>,
    }

    let export: JsonExport = serde_json::from_str(content)?;
    let tracks = export
        .tracks
        .into_iter()
        .map(|t| RawTrackEntry {
            title: t.title,
            artists: t
                .artists
                .or_else(|| t.artist.map(|a| vec![a]))
                .unwrap_or_default(),
            album: t.album,
            duration_secs: t.duration,
            isrc: t.isrc,
            media_url: t.url,
        })
        .collect();

    Ok(RawPlaylist {
        source_service: "json".into(),
        title: export.title.unwrap_or_else(|| "Imported playlist".into()),
        description: export.description,
        owner: export.owner,
        origin: origin.into(),
        tracks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_m3u_with_metadata() {
        let m3u = "#EXTM3U\n#PLAYLIST:Rock Classics\n#EXTINF:331, Metallica - Enter Sandman\nhttps://example.com/1.mp3\n#EXTINF:388, Metallica - Nothing Else Matters\nhttps://example.com/2.mp3\n";
        let pl = parse_m3u(m3u, "test.m3u").unwrap();
        assert_eq!(pl.title, "Rock Classics");
        assert_eq!(pl.tracks.len(), 2);
        assert_eq!(pl.tracks[0].title, "Enter Sandman");
        assert_eq!(pl.tracks[0].artists, vec!["Metallica"]);
        assert_eq!(pl.tracks[0].duration_secs, Some(331.0));
        assert_eq!(pl.tracks[0].media_url, Some("https://example.com/1.mp3".into()));
    }

    #[test]
    fn parses_m3u_without_metadata() {
        let m3u = "file:///music/Nirvana - Smells Like Teen Spirit.mp3\n";
        let pl = parse_m3u(m3u, "x.m3u8").unwrap();
        assert_eq!(pl.tracks[0].title, "Smells Like Teen Spirit");
    }

    #[test]
    fn parses_xspf() {
        let xspf = r##"<?xml version="1.0" encoding="UTF-8"?>
<playlist version="1" xmlns="http://xspf.org/ns/0/">
  <title>My Playlist</title>
  <trackList>
    <track>
      <title>Everlong</title>
      <creator>Foo Fighters</creator>
      <album>The Colour and the Shape</album>
      <duration>250000</duration>
      <location>https://example.com/everlong.mp3</location>
    </track>
  </trackList>
</playlist>"##;
        let pl = parse_xspf(xspf, "test.xspf").unwrap();
        assert_eq!(pl.title, "My Playlist");
        assert_eq!(pl.tracks.len(), 1);
        assert_eq!(pl.tracks[0].title, "Everlong");
        assert_eq!(pl.tracks[0].artists, vec!["Foo Fighters".to_string()]);
        assert_eq!(pl.tracks[0].duration_secs, Some(250.0));
    }

    #[test]
    fn parses_csv() {
        let csv = "title,artist,album,duration\nEverlong,Foo Fighters,The Colour,250\n\"Nothing Else Matters\",Metallica,Black Album,388\n";
        let pl = parse_csv(csv, "test.csv").unwrap();
        assert_eq!(pl.tracks.len(), 2);
        assert_eq!(pl.tracks[0].title, "Everlong");
        assert_eq!(pl.tracks[1].artists, vec!["Metallica".to_string()]);
        assert_eq!(pl.tracks[1].duration_secs, Some(388.0));
    }

    #[test]
    fn parses_json_export() {
        let json = r#"{"title":"Mix","tracks":[{"title":"Creep","artist":"Radiohead","duration":238}]}"#;
        let pl = parse_export_json(json, "test.json").unwrap();
        assert_eq!(pl.title, "Mix");
        assert_eq!(pl.tracks[0].artists, vec!["Radiohead".to_string()]);
    }
}
