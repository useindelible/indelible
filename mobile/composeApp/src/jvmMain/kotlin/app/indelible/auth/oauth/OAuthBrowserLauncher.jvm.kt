package app.indelible.auth.oauth

import androidx.compose.runtime.Composable

@Composable
actual fun rememberOAuthBrowserLauncher(): OAuthBrowserLauncher = NoopOAuthBrowserLauncher
