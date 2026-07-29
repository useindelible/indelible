package app.indelible.ui.platform

import androidx.compose.runtime.Composable

/**
 * Hides or shows the OS system bars (status and navigation) to deepen the reader's
 * immersive mode, restoring them when the caller leaves composition. A swipe still
 * reveals them transiently, so the user is never trapped. No-op on platforms where
 * the bars are owned by the host (iOS view controller, desktop), so shared code can
 * call it unconditionally.
 */
@Composable
expect fun ImmersiveSystemBars(hidden: Boolean)

/**
 * Keeps the OS status-bar icons (clock, battery, signal) legible against the app's
 * surfaces. [lightStatusBars] true means the bar sits on a LIGHT surface, so the
 * system paints DARK icons — the readable clock on the white sidebar; false paints
 * light icons for dark surfaces. Nesting is supported: a deeper override (e.g. the
 * reader's own canvas contrast) restores the surrounding screen's appearance when it
 * leaves composition. No-op where the status bar is host-owned (iOS view controller,
 * desktop), so shared code can call it unconditionally.
 */
@Composable
expect fun StatusBarAppearance(lightStatusBars: Boolean)
