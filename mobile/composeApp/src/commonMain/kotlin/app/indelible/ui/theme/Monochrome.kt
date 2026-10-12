package app.indelible.ui.theme

import androidx.compose.runtime.Immutable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color

/**
 * The monochrome palette the app is moving to: warm near-black, bone ink, one neutral accent.
 * [boneTop], [boneMid] and [boneBottom] are the stops of the polished primary fill, [glossTop] and
 * [glossMid] the sheen at the top of a surface, and [specular] its one-pixel top edge.
 */
@Immutable
data class MonochromeColors(
    val canvas: Color,
    val surface: Color,
    val raised: Color,
    val raised2: Color,
    val line: Color,
    val line2: Color,
    val ink: Color,
    val ink2: Color,
    val ink3: Color,
    val accentWash: Color,
    val accentEdge: Color,
    val onAccent: Color,
    val success: Color,
    val successWash: Color,
    val warning: Color,
    val warningWash: Color,
    val track: Color,
    val boneTop: Color,
    val boneMid: Color,
    val boneBottom: Color,
    val glossTop: Color,
    val glossMid: Color,
    /** Where the sheen has faded out, as a fraction of the surface height. */
    val glossEnd: Float,
    val specular: Color,
    val dropShadow: Color,
)

val DarkMonochrome =
    MonochromeColors(
        canvas = Color(0xFF0A0A0A),
        surface = Color(0xFF121212),
        raised = Color(0xFF181818),
        raised2 = Color(0xFF1F1F1F),
        line = Color.White.copy(alpha = 0.08f),
        line2 = Color.White.copy(alpha = 0.15f),
        ink = Color(0xFFF2F2F2),
        ink2 = Color(0xFFA3A3A3),
        ink3 = Color(0xFF6B6B6B),
        accentWash = Color.White.copy(alpha = 0.07f),
        accentEdge = Color.White.copy(alpha = 0.2f),
        onAccent = Color(0xFF0A0A0A),
        success = Color(0xFF8FBF6B),
        successWash = Color(0xFF8FBF6B).copy(alpha = 0.11f),
        warning = Color(0xFFD4D4D4),
        warningWash = Color.White.copy(alpha = 0.06f),
        track = Color.White.copy(alpha = 0.08f),
        boneTop = Color(0xFFFFFFFF),
        boneMid = Color(0xFFE4E4E4),
        boneBottom = Color(0xFFC8C8C8),
        glossTop = Color.White.copy(alpha = 0.055f),
        glossMid = Color.White.copy(alpha = 0.014f),
        glossEnd = 0.7f,
        specular = Color.White.copy(alpha = 0.085f),
        dropShadow = Color.Black.copy(alpha = 0.95f),
    )

val LightMonochrome =
    MonochromeColors(
        canvas = Color(0xFFF4F4F4),
        surface = Color(0xFFFFFFFF),
        raised = Color(0xFFFFFFFF),
        raised2 = Color(0xFFF0F0F0),
        line = Color.Black.copy(alpha = 0.075f),
        line2 = Color.Black.copy(alpha = 0.16f),
        ink = Color(0xFF0A0A0A),
        ink2 = Color(0xFF555555),
        ink3 = Color(0xFF8C8C8C),
        accentWash = Color.Black.copy(alpha = 0.05f),
        accentEdge = Color.Black.copy(alpha = 0.14f),
        onAccent = Color(0xFFFFFFFF),
        success = Color(0xFF3E7A2B),
        successWash = Color(0xFF3E7A2B).copy(alpha = 0.09f),
        warning = Color(0xFF3D3D3D),
        warningWash = Color.Black.copy(alpha = 0.04f),
        track = Color.Black.copy(alpha = 0.08f),
        boneTop = Color(0xFF3A3A3A),
        boneMid = Color(0xFF151515),
        boneBottom = Color(0xFF050505),
        glossTop = Color.White,
        glossMid = Color.White.copy(alpha = 0.3f),
        glossEnd = 0.6f,
        specular = Color.White,
        dropShadow = Color.Black.copy(alpha = 0.28f),
    )

val LocalMonochrome = staticCompositionLocalOf { LightMonochrome }

/** One state of the window seat: a top-to-bottom sky, three cloud layers and the wing. */
@Immutable
data class WindowSky(
    val sky: Brush,
    val cloudFar: Color,
    val cloudMid: Color,
    val cloudNear: Color,
    val wing: Color,
    val wingEdge: Color,
    val wingShade: Color,
)

/** The window is outside the app's theme: the same sky in light and dark. */
object WindowScene {
    val glass = Color(0xFF050606)
    val chipFill = Color(0xFF080909).copy(alpha = 0.42f)
    val chipEdge = Color.White.copy(alpha = 0.12f)
    val onScene = Color(0xFFF5F5F5)
    val star = Color.White
    val moon = Color(0xFFF4F4F4)
    val sunCore = Color.White
    val sunHalo = Color(0xFFEDEDED)
    val navLight = Color(0xFF9BE07A)
    val strobe = Color.White
    val glint = Color.White.copy(alpha = 0.07f)
    val glintFaint = Color.White.copy(alpha = 0.035f)
    val glassEdge = Color.White.copy(alpha = 0.10f)

    val dusk =
        WindowSky(
            sky =
                Brush.verticalGradient(
                    0f to hex(0x050505),
                    0.4f to hex(0x0E0E0E),
                    0.7f to hex(0x262626),
                    0.92f to hex(0x5A5A5A),
                    1f to hex(0x7A7A7A),
                ),
            cloudFar = hex(0x8E8E8E),
            cloudMid = hex(0x484848),
            cloudNear = hex(0x222222),
            wing = hex(0x0A0A0A),
            wingEdge = hex(0xD9D9D9),
            wingShade = Color.Black.copy(alpha = 0.4f),
        )
    val dawn =
        WindowSky(
            sky =
                Brush.verticalGradient(
                    0f to hex(0x1A1A1A),
                    0.36f to hex(0x4A4A4A),
                    0.68f to hex(0x9C9C9C),
                    1f to hex(0xE6E6E6),
                ),
            cloudFar = hex(0xE2E2E2),
            cloudMid = hex(0xA8A8A8),
            cloudNear = hex(0x5E5E5E),
            wing = hex(0x242424),
            wingEdge = hex(0xF5F5F5),
            wingShade = Color.Black.copy(alpha = 0.4f),
        )
    val day =
        WindowSky(
            sky =
                Brush.verticalGradient(
                    0f to hex(0x8C8C8C),
                    0.38f to hex(0xB4B4B4),
                    0.72f to hex(0xD8D8D8),
                    1f to hex(0xF4F4F4),
                ),
            cloudFar = hex(0xFFFFFF),
            cloudMid = hex(0xF0F0F0),
            cloudNear = hex(0xD2D2D2),
            wing = hex(0xB8B8B8),
            wingEdge = hex(0xFFFFFF),
            wingShade = Color.Black.copy(alpha = 0.2f),
        )
    val night =
        WindowSky(
            sky =
                Brush.verticalGradient(
                    0f to hex(0x030303),
                    0.55f to hex(0x0A0A0A),
                    1f to hex(0x151515),
                ),
            cloudFar = hex(0x272727),
            cloudMid = hex(0x1B1B1B),
            cloudNear = hex(0x121212),
            wing = hex(0x070707),
            wingEdge = hex(0x404040),
            wingShade = Color.Black.copy(alpha = 0.4f),
        )
}

/** Cover art for items on the shelf; each style is a background ramp plus one motif colour. */
object CoverArt {
    val ink = Color(0xFFF5F5F5)
    val paperInk = Color(0xFF141414)
    val ember = listOf(hex(0x222222), hex(0x090909)) to hex(0xE7A45E)
    val oxblood = listOf(hex(0x5A241C), hex(0x1E0C09)) to Color.White.copy(alpha = 0.12f)
    val coastSky = listOf(hex(0xF0C58F), hex(0xD98A5C))
    val coastSea = listOf(hex(0x2D4A44), hex(0x14231F))
    val coastSun = hex(0xFFF1CF)
    val bone = hex(0xECECEC) to hex(0xCFCFCF)
    val tide = listOf(hex(0x2A2A2A), hex(0x0D0D0D)) to Color.White.copy(alpha = 0.2f)
    val pages = hex(0xE4E4E4)
}

private fun hex(rgb: Int): Color = Color(0xFF000000 or rgb.toLong())
