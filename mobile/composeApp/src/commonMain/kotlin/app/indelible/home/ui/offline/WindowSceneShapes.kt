package app.indelible.home.ui.offline

import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.PathOperation
import androidx.compose.ui.graphics.vector.PathParser
import app.indelible.ui.theme.WindowScene
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.sin

/** A repeating scalloped band, one [period] wide and [height] tall, with bumps as (x, y, radius); all in dp. */
internal class CloudLayer(
    val period: Float,
    val height: Float,
    val cycleMs: Int,
    val alpha: Float,
    val baseTop: Float,
    val bumps: List<Triple<Float, Float, Float>>,
)

/** Far to near; all three sit behind the wing. */
internal val CLOUD_LAYERS =
    listOf(
        CloudLayer(
            period = 350f,
            height = 84f,
            cycleMs = 110_000,
            alpha = 0.9f,
            baseTop = 46f,
            bumps =
                listOf(
                    Triple(22f, 48f, 18f),
                    Triple(58f, 42f, 24f),
                    Triple(98f, 50f, 16f),
                    Triple(134f, 40f, 26f),
                    Triple(176f, 48f, 18f),
                    Triple(214f, 44f, 22f),
                    Triple(254f, 50f, 16f),
                    Triple(290f, 42f, 24f),
                    Triple(330f, 48f, 18f),
                ),
        ),
        CloudLayer(
            period = 300f,
            height = 66f,
            cycleMs = 70_000,
            alpha = 1f,
            baseTop = 36f,
            bumps =
                listOf(
                    Triple(28f, 38f, 24f),
                    Triple(74f, 32f, 28f),
                    Triple(122f, 42f, 18f),
                    Triple(162f, 30f, 30f),
                    Triple(214f, 40f, 20f),
                    Triple(258f, 34f, 26f),
                ),
        ),
        CloudLayer(
            period = 420f,
            height = 46f,
            cycleMs = 44_000,
            alpha = 1f,
            baseTop = 28f,
            bumps =
                listOf(
                    Triple(32f, 30f, 22f),
                    Triple(90f, 24f, 28f),
                    Triple(150f, 32f, 18f),
                    Triple(200f, 22f, 30f),
                    Triple(262f, 32f, 20f),
                    Triple(316f, 26f, 26f),
                    Triple(376f, 30f, 22f),
                ),
        ),
    )

/** One period of [layer] with its top at y = 0, in px. */
internal fun cloudBand(
    layer: CloudLayer,
    density: Float,
): Path =
    Path().apply {
        addRect(Rect(0f, layer.baseTop * density, layer.period * density, layer.height * density))
        layer.bumps.forEach { (x, y, r) -> addOval(Rect(Offset(x * density, y * density), r * density)) }
    }

internal fun cloudShade(
    layer: CloudLayer,
    sceneHeight: Float,
    density: Float,
): Brush =
    Brush.verticalGradient(
        listOf(Color.White.copy(alpha = 0.16f), Color.Black.copy(alpha = 0.28f)),
        0f,
        minOf(layer.height * density, sceneHeight),
    )

/** (x%, y%, diameter in dp) inside the upper part of the window. */
internal val STARS_A =
    listOf(
        Triple(8f, 22f, 1.4f),
        Triple(19f, 58f, 1f),
        Triple(31f, 12f, 1.2f),
        Triple(44f, 40f, 1f),
        Triple(57f, 16f, 1.6f),
        Triple(66f, 52f, 1f),
        Triple(88f, 64f, 1.2f),
        Triple(94f, 12f, 1f),
    )
internal val STARS_B =
    listOf(
        Triple(13f, 40f, 1f),
        Triple(26f, 76f, 1.3f),
        Triple(38f, 28f, 1f),
        Triple(51f, 66f, 1.2f),
        Triple(62f, 30f, 1f),
        Triple(74f, 8f, 1.4f),
    )

internal class MoonShape(
    val center: Offset,
    val lit: Path,
    val glow: Brush,
    val glowRadius: Float,
)

/** A crescent: the disc less the same disc shifted by the shadow offset, turned by the tilt. */
internal fun moonPath(
    width: Float,
    density: Float,
): MoonShape {
    val r = MOON_RADIUS * density
    val center = Offset(width - MOON_RIGHT * density - r, MOON_TOP * density + r)
    val rad = MOON_TILT * PI.toFloat() / 180f
    val dx = MOON_SHADOW_X * density
    val dy = MOON_SHADOW_Y * density
    val shadow = Offset(dx * cos(rad) - dy * sin(rad), dx * sin(rad) + dy * cos(rad))
    val lit =
        Path().apply {
            val disc = Path().apply { addOval(Rect(center, r)) }
            op(disc, Path().apply { addOval(Rect(center + shadow, r)) }, PathOperation.Difference)
        }
    val glowRadius = r * MOON_GLOW
    val glow = Brush.radialGradient(listOf(Color.White.copy(alpha = 0.2f), Color.Transparent), center, glowRadius)
    return MoonShape(center, lit, glow, glowRadius)
}

internal class GlassBrushes(
    val streak: Brush,
    val faintStreak: Brush,
    val innerShadow: Brush,
)

/** The two streaks at 112 degrees and the shadow the frame casts into the top of the pane. */
internal fun glassBrushes(
    width: Float,
    density: Float,
): GlassBrushes {
    val end = Offset(width, width * GLASS_SLOPE)
    return GlassBrushes(
        streak =
            Brush.linearGradient(
                0.34f to Color.Transparent,
                0.40f to WindowScene.glint,
                0.47f to Color.Transparent,
                start = Offset.Zero,
                end = end,
            ),
        faintStreak =
            Brush.linearGradient(
                0.49f to Color.Transparent,
                0.52f to WindowScene.glintFaint,
                0.55f to Color.Transparent,
                start = Offset.Zero,
                end = end,
            ),
        innerShadow =
            Brush.verticalGradient(
                listOf(Color.Black.copy(alpha = 0.3f), Color.Transparent),
                0f,
                INNER_SHADOW * density,
            ),
    )
}

internal val WING_BODY = PathParser().parsePathString("M28 206 L302 148 Q312 146 318 147 L326 150 L230 206 Z").toPath()
internal val WING_SHADE = PathParser().parsePathString("M168 206 L320 149 L326 150 L230 206 Z").toPath()
internal val WING_FIN = PathParser().parsePathString("M310 148 L318 110 Q321 104 326 106 L327 150 Z").toPath()
internal val WING_EDGE = PathParser().parsePathString("M28 206 L302 148 Q312 146 318 147").toPath()
internal val FIN_EDGE = PathParser().parsePathString("M311 147 L318 110 Q321 104 326 106").toPath()
internal val NAV_LIGHT = Offset(324.5f, 107f)
internal val STROBE_LIGHT = Offset(324.5f, 112f)
internal const val WING_W = 344f
internal const val WING_H = 206f
internal const val WING_EDGE_WIDTH = 1.6f
internal const val NAV_RADIUS = 2.2f
internal const val NAV_GLOW = 5f
internal const val STROBE_RADIUS = 3f

private const val MOON_RADIUS = 13f
private const val MOON_RIGHT = 40f
private const val MOON_TOP = 26f
private const val MOON_SHADOW_X = -8f
private const val MOON_SHADOW_Y = 4f
private const val MOON_TILT = -20f
private const val MOON_GLOW = 2.2f
private const val GLASS_SLOPE = 0.4f
private const val INNER_SHADOW = 22f
