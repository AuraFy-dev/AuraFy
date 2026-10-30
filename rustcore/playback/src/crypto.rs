use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand::RngCore;

/// AES-256-GCM encryption for the offline cache. The key is stored in
/// Android Keystore and passed over FFI; it never touches disk.
#[derive(Clone)]
pub struct CacheCrypto {
    cipher: Aes256Gcm,
}

impl CacheCrypto {
    pub fn new(key_bytes: &[u8; 32]) -> Self {
        Self {
            cipher: Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key_bytes)),
        }
    }

    /// Derive a random 32-byte key (for tests / server-side use).
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }

    /// Encrypt with a fresh random 12-byte nonce; returns nonce || ciphertext.
    pub fn encrypt(&self, plaintext: &[u8], aad: &str) -> Vec<u8> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(
                nonce,
                Payload {
                    msg: plaintext,
                    aad: aad.as_bytes(),
                },
            )
            .expect("AES-GCM encryption cannot fail with valid key");
        let mut out = Vec::with_capacity(12 + ciphertext.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        out
    }

    /// Decrypt a nonce || ciphertext blob produced by `encrypt`.
    pub fn decrypt(&self, blob: &[u8], aad: &str) -> Result<Vec<u8>, String> {
        if blob.len() < 12 {
            return Err("blob too short".into());
        }
        let (nonce_bytes, ciphertext) = blob.split_at(12);
        self.cipher
            .decrypt(
                Nonce::from_slice(nonce_bytes),
                Payload {
                    msg: ciphertext,
                    aad: aad.as_bytes(),
                },
            )
            .map_err(|_| "decryption failed (wrong key or corrupted blob)".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let crypto = CacheCrypto::new(&CacheCrypto::generate_key());
        let blob = crypto.encrypt(b"audio bytes", "track:42");
        let plain = crypto.decrypt(&blob, "track:42").unwrap();
        assert_eq!(plain, b"audio bytes");
    }

    #[test]
    fn aad_mismatch_fails() {
        let crypto = CacheCrypto::new(&CacheCrypto::generate_key());
        let blob = crypto.encrypt(b"audio bytes", "track:42");
        assert!(crypto.decrypt(&blob, "track:43").is_err());
    }

    #[test]
    fn wrong_key_fails() {
        let a = CacheCrypto::new(&CacheCrypto::generate_key());
        let b = CacheCrypto::new(&CacheCrypto::generate_key());
        let blob = a.encrypt(b"secret", "track:42");
        assert!(b.decrypt(&blob, "track:42").is_err());
    }
}
