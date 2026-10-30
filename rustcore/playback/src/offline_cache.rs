use std::path::{Path, PathBuf};

use crate::crypto::CacheCrypto;

/// Encrypted on-disk cache for downloaded tracks.
/// Layout: `<cache_dir>/<track_id>.bin` where .bin = nonce || AES-GCM blob,
/// with track_id bound as AAD (tampering across files fails).
pub struct OfflineCache {
    dir: PathBuf,
    crypto: CacheCrypto,
}

impl OfflineCache {
    pub fn new(dir: impl Into<PathBuf>, crypto: CacheCrypto) -> std::io::Result<Self> {
        let dir = dir.into();
        std::fs::create_dir_all(&dir)?;
        Ok(Self { dir, crypto })
    }

    fn path_for(&self, track_id: &str) -> PathBuf {
        // Sanitize: only [A-Za-z0-9_-] survive.
        let safe: String = track_id
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        self.dir.join(format!("{}.bin", safe))
    }

    pub fn store(&self, track_id: &str, audio: &[u8]) -> std::io::Result<()> {
        let blob = self.crypto.encrypt(audio, &format!("track:{}", track_id));
        let tmp = self.path_for(track_id).with_extension("tmp");
        std::fs::write(&tmp, &blob)?;
        std::fs::rename(&tmp, self.path_for(track_id))
    }

    pub fn load(&self, track_id: &str) -> Result<Option<Vec<u8>>, String> {
        let path = self.path_for(track_id);
        if !path.exists() {
            return Ok(None);
        }
        let blob = std::fs::read(&path).map_err(|e| e.to_string())?;
        self.crypto
            .decrypt(&blob, &format!("track:{}", track_id))
            .map(Some)
    }

    pub fn remove(&self, track_id: &str) {
        let _ = std::fs::remove_file(self.path_for(track_id));
    }

    pub fn contains(&self, track_id: &str) -> bool {
        self.path_for(track_id).exists()
    }

    pub fn total_size(&self) -> u64 {
        walk_size(&self.dir)
    }
}

fn walk_size(dir: &Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += walk_size(&path);
            } else if let Ok(md) = entry.metadata() {
                total += md.len();
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_load_remove_roundtrip() {
        let dir = std::env::temp_dir().join("aurafy_cache_test");
        let cache = OfflineCache::new(
            dir.join(format!("{:?}", std::process::id())),
            CacheCrypto::new(&CacheCrypto::generate_key()),
        )
        .unwrap();
        cache.store("track/42", b"fake audio").unwrap();
        assert!(cache.contains("track/42"));
        let audio = cache.load("track/42").unwrap().unwrap();
        assert_eq!(audio, b"fake audio");
        cache.remove("track/42");
        assert!(!cache.contains("track/42"));
    }
}
