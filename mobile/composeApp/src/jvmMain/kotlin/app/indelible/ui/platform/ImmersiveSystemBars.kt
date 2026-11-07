package app.indelible.ui.platform

import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.statusBars
import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.Dp

@Composable
actual fun ImmersiveSystemBars(hidden: Boolean) {
}

@Composable
actual fun StatusBarAppearance(lightStatusBars: Boolean) {
}

@Composable
actual fun statusBarHeightIgnoringVisibility(): Dp = WindowInsets.statusBars.asPaddingValues().calculateTopPadding()
