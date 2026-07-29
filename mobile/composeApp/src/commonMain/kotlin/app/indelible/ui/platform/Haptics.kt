package app.indelible.ui.platform

import androidx.compose.runtime.Composable

/**
 * Returns a light, system-respecting haptic "tick" callback that is safe to
 * invoke from any gesture handler (dock toggle, scope selection, highlight
 * created, triage swipe). The callback honours the OS haptic settings and is a
 * no-op on platforms without a haptic engine (desktop). Captured once per
 * composition so the platform generator can be pre-warmed.
 */
@Composable
expect fun rememberHapticTick(): () -> Unit
