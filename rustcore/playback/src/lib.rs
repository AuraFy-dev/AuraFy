//! Playback support: encrypted offline cache. Actual audio output is
//! delegated to Android Media3/ExoPlayer; Rust decoders (symphonia)
//! plug in via FFI for formats ExoPlayer lacks.

pub mod crypto;
pub mod offline_cache;

pub use crypto::CacheCrypto;
pub use offline_cache::OfflineCache;
