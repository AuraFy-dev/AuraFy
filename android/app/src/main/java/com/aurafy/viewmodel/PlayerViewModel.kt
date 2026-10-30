package com.aurafy.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.aurafy_ffi.AurafyApp
import uniffi.aurafy_ffi.TrackDto

/**
 * UI state for the player screen.
 */
data class PlayerUiState(
    val trackId: String? = null,
    val title: String = "",
    val artists: String = "",
    val album: String? = null,
    val durationSecs: Long = 0L,
    val positionSecs: Long = 0L,
    val isPlaying: Boolean = false,
    val streamUrl: String? = null,
    val isOffline: Boolean = false,
    val searchResults: List<TrackDto> = emptyList(),
)

/**
 * Calls the Rust core through UniFFI-generated bindings.
 * UniFFI calls are blocking (bridged onto a tokio runtime inside Rust),
 * so they run on Dispatchers.IO.
 */
class PlayerViewModel(
    private val app: AurafyApp = AurafyApp(),
) : ViewModel() {

    private val _uiState = MutableStateFlow(PlayerUiState())
    val uiState: StateFlow<PlayerUiState> = _uiState.asStateFlow()

    fun search(query: String) {
        if (query.isBlank()) return
        viewModelScope.launch {
            val results = withContext(Dispatchers.IO) {
                runCatching { app.search(query, 20u) }.getOrDefault(emptyList())
            }
            _uiState.update { it.copy(searchResults = results) }
        }
    }

    fun selectTrack(track: TrackDto) {
        viewModelScope.launch {
            val url = withContext(Dispatchers.IO) {
                runCatching { app.resolveStreamUrl(track.id) }.getOrNull()
            }
            _uiState.update {
                it.copy(
                    trackId = track.id,
                    title = track.title,
                    artists = track.artists,
                    album = track.album,
                    durationSecs = (track.durationSecs ?: 0.0).toLong(),
                    streamUrl = url,
                    isOffline = track.isOfflineAvailable,
                    positionSecs = 0L,
                )
            }
        }
    }

    fun playPause() {
        _uiState.update { it.copy(isPlaying = !it.isPlaying) }
    }

    fun seekTo(positionSecs: Long) {
        _uiState.update { it.copy(positionSecs = positionSecs) }
    }

    override fun onCleared() {
        app.close()
        super.onCleared()
    }
}
