#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("unsupported playlist format: {0}")]
    UnsupportedFormat(String),
    #[error("malformed playlist: {0}")]
    Malformed(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
