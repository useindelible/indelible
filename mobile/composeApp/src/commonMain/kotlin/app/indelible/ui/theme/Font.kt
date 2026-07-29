package app.indelible.ui.theme

import androidx.compose.runtime.Composable
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.geist_bold
import indelible.composeapp.generated.resources.geist_medium
import indelible.composeapp.generated.resources.geist_mono_medium
import indelible.composeapp.generated.resources.geist_mono_regular
import indelible.composeapp.generated.resources.geist_regular
import indelible.composeapp.generated.resources.geist_semibold
import org.jetbrains.compose.resources.Font

/**
 * Geist — the reimagined UI sans. Loaded from bundled OFL TTFs via
 * compose-resources, so the builder must run in composition.
 */
@Composable
fun geistFontFamily(): FontFamily =
    FontFamily(
        Font(Res.font.geist_regular, FontWeight.Normal),
        Font(Res.font.geist_medium, FontWeight.Medium),
        Font(Res.font.geist_semibold, FontWeight.SemiBold),
        Font(Res.font.geist_bold, FontWeight.Bold),
    )

/** Geist Mono — used for code, durations, and monospace metadata. */
@Composable
fun geistMonoFontFamily(): FontFamily =
    FontFamily(
        Font(Res.font.geist_mono_regular, FontWeight.Normal),
        Font(Res.font.geist_mono_medium, FontWeight.Medium),
    )

/**
 * Serif family for the reader's literary surfaces — item-record hero title,
 * summary, note body, and highlight quotes. The reimagined design specifies
 * Newsreader, which is loaded inside the reader WebView via the Google Fonts
 * CDN but is not bundled as a native TTF. Native Compose therefore falls back
 * to the platform serif, matching the prototype CSS fallback declared after
 * Newsreader. Bundling Newsreader natively is a separate operator task.
 */
val SerifFontFamily: FontFamily = FontFamily.Serif
