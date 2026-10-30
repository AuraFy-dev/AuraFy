//! UniFFI facade exposed to Kotlin. Async core operations are bridged
//! onto a background tokio runtime.

mod app;

use uniffi::{self, export};

pub use app::{AurafyApp, TrackDto};

/// Global background runtime for async FFI calls.
pub fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("failed to build tokio runtime")
    })
}

#[export]
fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

uniffi::setup_scaffolding!();
