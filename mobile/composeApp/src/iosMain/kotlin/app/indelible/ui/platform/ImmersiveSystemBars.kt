package app.indelible.ui.platform

import androidx.compose.runtime.Composable

// The iOS status bar is owned by the hosting UIViewController
// (prefersStatusBarHidden); toggling it from shared Compose needs a Swift-side
// bridge that does not exist yet. The reader's full-bleed aura already paints
// behind the iOS status bar, so this stays a no-op until that bridge lands.
@Composable
actual fun ImmersiveSystemBars(hidden: Boolean) {
}

// The iOS status-bar style (light/dark icons) is owned by the hosting
// UIViewController's preferredStatusBarStyle; driving it from shared Compose needs
// the same Swift-side bridge ImmersiveSystemBars awaits. No-op until that lands.
@Composable
actual fun StatusBarAppearance(lightStatusBars: Boolean) {
}
