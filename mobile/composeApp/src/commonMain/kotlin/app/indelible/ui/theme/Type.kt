package app.indelible.ui.theme

import androidx.compose.material3.Typography
import androidx.compose.runtime.Composable
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp

/**
 * Indelible typography scale mapped to Material 3 slots.
 *
 * Mapping reference:
 *   displaySmall   → display      (34sp / 700 / -0.04em / lh 1.12)
 *   headlineLarge  → title-1      (28sp / 700 / -0.03em / lh 1.18)
 *   headlineMedium → title-2      (22sp / 700 / -0.03em / lh 1.20)
 *   headlineSmall  → title-3      (20sp / 600 / -0.025em / lh 1.25)
 *   titleLarge     → headline     (17sp / 600 / -0.02em / lh 1.30)
 *   bodyLarge      → body         (15sp / 400 / -0.01em / lh 1.50)
 *   titleSmall     → callout      (14sp / 600 / -0.01em / lh 1.40)
 *   bodyMedium     → subheadline  (13sp / 400 / -0.01em / lh 1.45)
 *   bodySmall      → footnote     (12sp / 400 / -0.005em / lh 1.40)
 *   labelSmall     → caption-1    (11sp / 500 / +0.06em / lh 1.20, uppercase at call site)
 *                  → caption-2    (11sp / 400 / -0.005em / lh 1.20)
 *
 * For caption-1 (section labels — UPPERCASE, wide tracking), use:
 *   MaterialTheme.typography.labelSmall.copy(
 *       fontWeight = FontWeight.Medium,
 *       letterSpacing = 0.06.em,
 *   )
 * and apply textDecoration / textTransform via Modifier or a wrapper composable.
 *
 * Built as a @Composable so the Geist family (loaded from bundled TTFs via
 * compose-resources) is injected into every slot — including the Material
 * default slots — so no text falls back to the platform default font.
 */
@Composable
fun indelibleTypography(): Typography {
    val geist = geistFontFamily()
    val base = Typography()
    return Typography(
        // Material-default slots, re-skinned with Geist so nothing falls back.
        displayLarge = base.displayLarge.copy(fontFamily = geist),
        displayMedium = base.displayMedium.copy(fontFamily = geist),
        // display (34sp / 700 / -0.04em) — empty states, onboarding hero
        displaySmall =
            TextStyle(
                fontFamily = geist,
                fontSize = 34.sp,
                fontWeight = FontWeight.Bold,
                letterSpacing = (-0.04).em,
                lineHeight = 38.sp,
            ),
        // title-1 (28sp / 700 / -0.03em) — page titles: "Library", "Settings"
        headlineLarge =
            TextStyle(
                fontFamily = geist,
                fontSize = 28.sp,
                fontWeight = FontWeight.Bold,
                letterSpacing = (-0.03).em,
                lineHeight = 33.sp,
            ),
        // title-2 (22sp / 700 / -0.03em) — section headers, panel titles
        headlineMedium =
            TextStyle(
                fontFamily = geist,
                fontSize = 22.sp,
                fontWeight = FontWeight.Bold,
                letterSpacing = (-0.03).em,
                lineHeight = 26.sp,
            ),
        // title-3 (20sp / 600 / -0.025em) — detail panel article title
        headlineSmall =
            TextStyle(
                fontFamily = geist,
                fontSize = 20.sp,
                fontWeight = FontWeight.SemiBold,
                letterSpacing = (-0.025).em,
                lineHeight = 25.sp,
            ),
        // headline (17sp / 600 / -0.02em) — list item titles, bold labels
        titleLarge =
            TextStyle(
                fontFamily = geist,
                fontSize = 17.sp,
                fontWeight = FontWeight.SemiBold,
                letterSpacing = (-0.02).em,
                lineHeight = 22.sp,
            ),
        titleMedium = base.titleMedium.copy(fontFamily = geist),
        // body (15sp / 400 / -0.01em) — primary body text
        bodyLarge =
            TextStyle(
                fontFamily = geist,
                fontSize = 15.sp,
                fontWeight = FontWeight.Normal,
                letterSpacing = (-0.01).em,
                lineHeight = 22.5.sp,
            ),
        // callout (14sp / 600 / -0.01em) — article row title in list
        titleSmall =
            TextStyle(
                fontFamily = geist,
                fontSize = 14.sp,
                fontWeight = FontWeight.SemiBold,
                letterSpacing = (-0.01).em,
                lineHeight = 19.6.sp,
            ),
        // subheadline (13sp / 400 / -0.01em) — metadata, secondary descriptions, nav labels
        bodyMedium =
            TextStyle(
                fontFamily = geist,
                fontSize = 13.sp,
                fontWeight = FontWeight.Normal,
                letterSpacing = (-0.01).em,
                lineHeight = 18.85.sp,
            ),
        // footnote (12sp / 400 / -0.005em) — timestamps, source labels
        bodySmall =
            TextStyle(
                fontFamily = geist,
                fontSize = 12.sp,
                fontWeight = FontWeight.Normal,
                letterSpacing = (-0.005).em,
                lineHeight = 16.8.sp,
            ),
        labelLarge = base.labelLarge.copy(fontFamily = geist),
        labelMedium = base.labelMedium.copy(fontFamily = geist),
        // caption-1 base (11sp / 500 / +0.06em) — section labels (uppercase at call site)
        // caption-2 base (11sp / 400 / -0.005em) — tab bar labels, tertiary metadata
        labelSmall =
            TextStyle(
                fontFamily = geist,
                fontSize = 11.sp,
                fontWeight = FontWeight.Normal,
                letterSpacing = (-0.005).em,
                lineHeight = 13.2.sp,
            ),
    )
}
