//! Data layer: sqlx-backed repository for tracks, playlists,
//! likes and listen history.

pub mod repository;

pub use repository::Repository;
