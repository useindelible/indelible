package app.indelible.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color
import app.indelible.ui.platform.StatusBarAppearance

// ============================================================
// LIGHT COLOR SCHEME
// ============================================================
private val LightColorScheme =
    lightColorScheme(
        // Accent / Primary
        primary = AccentLight,
        onPrimary = White,
        primaryContainer = FillSelectedLight,
        onPrimaryContainer = AccentLight,
        // Secondary (unused by Indelible, default Material)
        secondary = AccentLight,
        onSecondary = White,
        secondaryContainer = FillSelectedLight,
        onSecondaryContainer = AccentLight,
        // Backgrounds
        background = BgPrimaryLight,
        onBackground = TextPrimaryLight,
        // Surfaces
        surface = BgPrimaryLight,
        onSurface = TextPrimaryLight,
        surfaceVariant = BgSecondaryLight,
        onSurfaceVariant = TextSecondaryLight,
        surfaceContainer = BgElevatedLight,
        surfaceContainerLow = BgSecondaryLight,
        surfaceContainerHigh = BgTertiaryLight,
        surfaceTint = AccentLight,
        // Borders / Outlines
        outline = BorderSecondaryLight,
        outlineVariant = BorderPrimaryLight,
        // Errors / Destructive
        error = DestructiveLight,
        onError = White,
        errorContainer = FillSelectedLight,
        onErrorContainer = DestructiveLight,
        // Inverse
        inverseSurface = TextPrimaryLight,
        inverseOnSurface = BgPrimaryLight,
        inversePrimary = AccentDark,
    )

// ============================================================
// DARK COLOR SCHEME
// ============================================================
private val DarkColorScheme =
    darkColorScheme(
        // Accent / Primary
        primary = AccentDark,
        onPrimary = White,
        primaryContainer = FillSelectedDark,
        onPrimaryContainer = AccentDark,
        // Secondary
        secondary = AccentDark,
        onSecondary = White,
        secondaryContainer = FillSelectedDark,
        onSecondaryContainer = AccentDark,
        // Backgrounds
        background = BgPrimaryDark,
        onBackground = TextPrimaryDark,
        // Surfaces
        surface = BgPrimaryDark,
        onSurface = TextPrimaryDark,
        surfaceVariant = BgSecondaryDark,
        onSurfaceVariant = TextSecondaryDark,
        surfaceContainer = BgElevatedDark,
        surfaceContainerLow = BgSecondaryDark,
        surfaceContainerHigh = BgTertiaryDark,
        surfaceTint = AccentDark,
        // Borders / Outlines
        outline = BorderSecondaryDark,
        outlineVariant = BorderPrimaryDark,
        // Errors / Destructive
        error = DestructiveDark,
        onError = White,
        errorContainer = FillSelectedDark,
        onErrorContainer = DestructiveDark,
        // Inverse
        inverseSurface = TextPrimaryDark,
        inverseOnSurface = BgPrimaryDark,
        inversePrimary = AccentLight,
    )

// ============================================================
// EXTENDED COLORS — semantic slots beyond MaterialTheme
// ============================================================
@Immutable
data class IndelibleExtendedColors(
    val warning: Color,
    val onWarning: Color,
    val success: Color,
    val onSuccess: Color,
    /**
     * Five-slot banner palette for collection cards.
     * Index: 0=blue, 1=green, 2=purple, 3=pink, 4=yellow.
     * Always 5 elements; access with a modulo-5 index.
     * Uses pastel highlight colors — suitable for large banner areas.
     */
    val collectionBanners: List<Color>,
    /**
     * Five-slot vivid accent palette for tag color dots and similar small indicators.
     * Same index mapping as [collectionBanners]: 0=blue, 1=green, 2=purple, 3=red, 4=orange.
     * Uses saturated system colors — suitable for small filled circles.
     */
    val tagColors: List<Color>,
    /**
     * Tertiary text — the prototype's --text-tertiary. No Material slot maps to
     * it (onSurfaceVariant is the secondary tier), so it lives here. Used for
     * mono eyebrow/section labels, grid keys, bylines, and note meta.
     */
    val textTertiary: Color,
    /** Reader canvas + text inks for the native reader surface (behind the WebView). */
    val readerBg: Color,
    val readerInk: Color,
    val readerBody: Color,
    /** Accent hairline used by reader chrome (progress fill, dividers). */
    val accentLine: Color,
    /** Stronger accent stop, paired with the primary accent for gradient fills. */
    val accentStrong: Color,
    /** Three-stop aura blob palettes, indexed by [app.indelible.ui.components.AuraStyle]. */
    /** Reader background swatches (paper, sepia, slate, black) for display settings. */
    val readerBackgroundSwatches: List<Color>,
)

private val LightExtendedColors =
    IndelibleExtendedColors(
        warning = WarningLight,
        onWarning = White,
        success = SuccessLight,
        onSuccess = White,
        collectionBanners =
            listOf(
                HighlightBlueLight,
                HighlightGreenLight,
                HighlightPurpleLight,
                HighlightPinkLight,
                HighlightYellowLight,
            ),
        tagColors =
            listOf(
                AccentLight,
                SuccessLight,
                PurpleLight,
                DestructiveLight,
                WarningLight,
            ),
        textTertiary = TextTertiaryLight,
        readerBg = ReaderBgPaper,
        readerInk = ReaderInkLight,
        readerBody = ReaderBodyLight,
        accentLine = AccentLineLight,
        accentStrong = AccentHoverLight,
        readerBackgroundSwatches = listOf(ReaderBgPaper, ReaderSwatchSepia, ReaderSwatchSlate, ReaderBgDark),
    )

private val DarkExtendedColors =
    IndelibleExtendedColors(
        warning = WarningDark,
        onWarning = White,
        success = SuccessDark,
        onSuccess = White,
        collectionBanners =
            listOf(
                HighlightBlueDark,
                HighlightGreenDark,
                HighlightPurpleDark,
                HighlightPinkDark,
                HighlightYellowDark,
            ),
        tagColors =
            listOf(
                AccentDark,
                SuccessDark,
                PurpleDark,
                DestructiveDark,
                WarningDark,
            ),
        textTertiary = TextTertiaryDark,
        readerBg = ReaderBgDark,
        readerInk = ReaderInkDark,
        readerBody = ReaderBodyDark,
        accentLine = AccentLineDark,
        accentStrong = AccentHoverDark,
        readerBackgroundSwatches = listOf(ReaderBgPaper, ReaderSwatchSepia, ReaderSwatchSlate, ReaderBgDark),
    )

val LocalIndelibleColors = staticCompositionLocalOf { LightExtendedColors }

// ============================================================
// APP THEME — use this as the root composable everywhere
// ============================================================

/**
 * The single source-of-truth theme for the Indelible mobile app.
 *
 * Wraps MaterialTheme with:
 *   - Indelible ColorScheme (light / dark)
 *   - Indelible Typography (11-role type scale)
 *   - Indelible Shapes (radius scale)
 *   - Extended colors (warning, success) via [LocalIndelibleColors]
 *
 * Usage: wrap the root App() composable and any Compose Previews.
 *
 * Access extended colors via `IndelibleTheme.colors.warning`, etc.
 */
@Composable
fun AppTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit,
) {
    val colorScheme = if (darkTheme) DarkColorScheme else LightColorScheme
    val extendedColors = if (darkTheme) DarkExtendedColors else LightExtendedColors

    CompositionLocalProvider(LocalIndelibleColors provides extendedColors) {
        MaterialTheme(
            colorScheme = colorScheme,
            typography = indelibleTypography(),
            shapes = IndelibleShapes,
        ) {
            // Keep the OS status-bar icons legible against the active theme: a light theme
            // (e.g. the white sidebar) gets dark icons. The reader re-enters AppTheme with
            // its own canvas darkness, so this tracks that contrast too.
            StatusBarAppearance(lightStatusBars = !darkTheme)
            content()
        }
    }
}

object IndelibleTheme {
    val colors: IndelibleExtendedColors
        @Composable get() = LocalIndelibleColors.current
}
