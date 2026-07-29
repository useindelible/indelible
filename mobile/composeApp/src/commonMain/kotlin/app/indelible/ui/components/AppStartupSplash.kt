package app.indelible.ui.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.core.platform.platformClientType
import app.indelible.ui.theme.AccentLight
import app.indelible.ui.theme.AppTheme
import kotlinx.coroutines.delay

private const val MIN_SPLASH_MS = 700L
private const val SPLASH_LOGO_WIDTH_FRACTION = 0.52f

@Composable
fun StartupSplashGate(
    isReady: Boolean,
    onNativeSplashReady: () -> Unit,
    content: @Composable () -> Unit,
) {
    var minimumSplashElapsed by remember { mutableStateOf(platformClientType() != "android") }

    LaunchedEffect(Unit) {
        onNativeSplashReady()
        if (!minimumSplashElapsed) {
            delay(MIN_SPLASH_MS)
            minimumSplashElapsed = true
        }
    }

    if (isReady && minimumSplashElapsed) {
        content()
    } else {
        AppStartupSplash()
    }
}

@Composable
fun AppStartupSplash(
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.background,
    ) {
        Box(
            modifier = Modifier.fillMaxSize(),
            contentAlignment = Alignment.Center,
        ) {
            IndelibleVaultMark(
                modifier =
                    Modifier
                        .fillMaxWidth(SPLASH_LOGO_WIDTH_FRACTION)
                        .aspectRatio(1f),
                frameColor = AccentLight,
            )
        }
    }
}

@Preview(showBackground = true)
@Composable
private fun AppStartupSplashPreviewLight() {
    AppTheme(darkTheme = false) {
        AppStartupSplash()
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun AppStartupSplashPreviewDark() {
    AppTheme(darkTheme = true) {
        AppStartupSplash()
    }
}
