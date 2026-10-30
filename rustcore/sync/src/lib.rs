//! Cross-device sync: a minimal axum server exposing profile, playlists
//! and playback state. The Android app syncs via the same structs.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::Json;
use domain::{PlaybackState, Playlist};
use tokio::sync::RwLock;

#[derive(Default)]
pub struct SyncStore {
    playlists: HashMap<String, Playlist>,
    playback: HashMap<String, PlaybackState>,
}

#[derive(Clone)]
pub struct AppState(Arc<RwLock<SyncStore>>);

pub fn router(state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/playlists", get(list_playlists))
        .route("/playlists/:id", get(get_playlist).put(put_playlist))
        .route("/playback/:device_id", get(get_playback).put(put_playback))
        .with_state(state)
}

async fn list_playlists(State(state): State<AppState>) -> Json<Vec<Playlist>> {
    let store = state.0.read().await;
    Json(store.playlists.values().cloned().collect())
}

async fn get_playlist(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Playlist>, StatusCode> {
    let store = state.0.read().await;
    store
        .playlists
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn put_playlist(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(playlist): Json<Playlist>,
) -> Json<Playlist> {
    let mut store = state.0.write().await;
    store.playlists.insert(id, playlist.clone());
    Json(playlist)
}

async fn get_playback(
    State(state): State<AppState>,
    axum::extract::Path(device_id): axum::extract::Path<String>,
) -> Result<Json<PlaybackState>, StatusCode> {
    let store = state.0.read().await;
    store
        .playback
        .get(&device_id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn put_playback(
    State(state): State<AppState>,
    axum::extract::Path(device_id): axum::extract::Path<String>,
    Json(playback): Json<PlaybackState>,
) -> Json<PlaybackState> {
    let mut store = state.0.write().await;
    store.playback.insert(device_id, playback.clone());
    Json(playback)
}



pub async fn serve(addr: &str) -> anyhow::Result<()> {
    let app = router(AppState(Arc::new(RwLock::new(SyncStore::default()))));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "sync server listening");
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::PlaylistTrack;

    #[tokio::test]
    async fn playlist_store_roundtrip() {
        let mut store = SyncStore::default();
        let playlist = Playlist {
            id: "p1".into(),
            title: "Mix".into(),
            description: None,
            owner: Some("me".into()),
            tracks: vec![PlaylistTrack {
                position: 0,
                track: domain::Track {
                    id: "t1".into(),
                    title: "Song".into(),
                    artists: vec!["A".into()],
                    album: None,
                    duration_secs: None,
                    artwork_url: None,
                    isrc: None,
                    external_ids: Vec::new(),
                    source: "local".into(),
                },
            }],
            source: "sync".into(),
        };
        store.playlists.insert("p1".into(), playlist.clone());
        assert_eq!(store.playlists.get("p1").unwrap().title, "Mix");
    }
}
