package com.aurafy.ui

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.lifecycle.viewmodel.compose.viewModel
import com.aurafy.ui.player.PlayerScreen
import com.aurafy.viewmodel.PlayerViewModel

@Composable
fun AuraFyApp() {
    val playerViewModel: PlayerViewModel = viewModel()
    Scaffold { padding ->
        PlayerScreen(
            viewModel = playerViewModel,
            modifier = Modifier.padding(padding),
        )
    }
}
