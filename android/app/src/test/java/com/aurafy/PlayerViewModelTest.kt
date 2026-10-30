package com.aurafy

import com.aurafy.viewmodel.PlayerUiState
import com.aurafy.viewmodel.PlayerViewModel
import uniffi.aurafy_ffi.AurafyApp
import uniffi.aurafy_ffi.TrackDto
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class PlayerViewModelTest {

    private fun viewModel() = PlayerViewModel(AurafyApp())

    @Test
    fun playPauseTogglesState() {
        val viewModel = viewModel()
        val initial = viewModel.uiState.value
        assertEquals(false, initial.isPlaying)

        viewModel.playPause()
        assertTrue(viewModel.uiState.value.isPlaying)

        viewModel.playPause()
        assertEquals(false, viewModel.uiState.value.isPlaying)
        viewModel.onCleared()
    }

    @Test
    fun seekUpdatesPosition() {
        val viewModel = viewModel()
        viewModel.seekTo(42L)
        assertEquals(42L, viewModel.uiState.value.positionSecs)
        viewModel.onCleared()
    }

    @Test
    fun selectTrackUpdatesState() {
        val viewModel = viewModel()
        val track = TrackDto(
            id = "local:test",
            title = "Test Song",
            artists = "Test Artist",
            album = null,
            durationSecs = 200.0,
            isOfflineAvailable = true,
            source = "local",
        )
        viewModel.selectTrack(track)
        // selectTrack launches a coroutine; state fields set synchronously after
        // resolveStreamUrl may lag, so poll briefly.
        val deadline = System.currentTimeMillis() + 2000
        while (System.currentTimeMillis() < deadline && viewModel.uiState.value.title != "Test Song") {
            Thread.sleep(50)
        }
        val state = viewModel.uiState.value
        assertEquals("Test Song", state.title)
        assertEquals("Test Artist", state.artists)
        assertEquals(200L, state.durationSecs)
        assertTrue(state.isOffline)
        viewModel.onCleared()
    }

    @Test
    fun initialUiStateIsEmpty() {
        val viewModel = viewModel()
        val state: PlayerUiState = viewModel.uiState.value
        assertTrue(state.searchResults.isEmpty())
        assertFalse(state.isPlaying)
        viewModel.onCleared()
    }
}
