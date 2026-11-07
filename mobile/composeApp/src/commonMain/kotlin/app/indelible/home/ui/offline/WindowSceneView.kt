package app.indelible.home.ui.offline

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.layout.Spacer
import androidx.compose.runtime.Composable
import androidx.compose.runtime.State
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithCache
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.scale
import androidx.compose.ui.graphics.drawscope.translate
import app.indelible.ui.theme.WindowScene
import app.indelible.ui.theme.WindowSky
import kotlin.math.max

/** The sky outside the window: moonlit dusk offline, dawn while sending, day once synced, night when empty. */
enum class SkyState { DUSK, DAWN, DAY, NIGHT }

/**
 * The view from a window seat; geometry is in dp. Paths and brushes are built
 * once per size; each frame only reads the animated values and translates the cloud bands.
 */
@Composable
fun WindowSceneView(
    state: SkyState,
    clock: State<Float>,
    modifier: Modifier = Modifier,
) {
    val skies = SkyState.entries.map { sky -> fade(state == sky) }
    val target = state.palette()
    val clouds =
        listOf(
            animateColorAsState(target.cloudFar, tween(SKY_MS)),
            animateColorAsState(target.cloudMid, tween(SKY_MS)),
            animateColorAsState(target.cloudNear, tween(SKY_MS)),
        )
    val wing = animateColorAsState(target.wing, tween(SKY_MS))
    val wingEdge = animateColorAsState(target.wingEdge, tween(SKY_MS))
    val wingShade = animateColorAsState(target.wingShade, tween(SKY_MS))
    val starLevel =
        animateFloatAsState(
            when (state) {
                SkyState.DUSK -> DUSK_STARS
                SkyState.NIGHT -> 1f
                else -> 0f
            },
            tween(SKY_MS),
        )
    val moon = fade(state == SkyState.DUSK)
    val sunUp = fade(state == SkyState.DAWN || state == SkyState.DAY)
    val sunHigh =
        animateFloatAsState(if (state == SkyState.DAY) 1f else 0f, tween(SUN_MS, easing = FastOutSlowInEasing))

    Spacer(
        modifier.drawWithCache {
            val bands = CLOUD_LAYERS.map { cloudBand(it, density) }
            val shades = CLOUD_LAYERS.map { cloudShade(it, size.height, density) }
            val moonShape = moonPath(size.width, density)
            val glass = glassBrushes(size.width, density)
            onDrawBehind {
                drawRect(WindowScene.glass)
                skies.forEachIndexed { i, alpha ->
                    if (alpha.value > 0f) drawRect(SkyState.entries[i].palette().sky, alpha = alpha.value)
                }
                val t = clock.value
                val stars = starLevel.value
                if (stars > 0f) {
                    drawStars(STARS_A, stars * twinkle(t, TWINKLE_A_MS))
                    drawStars(STARS_B, stars * twinkle(t, TWINKLE_B_MS))
                }
                if (moon.value > 0f) drawMoon(moonShape, moon.value)
                if (sunUp.value > 0f) drawSun(sunUp.value, sunHigh.value)
                CLOUD_LAYERS.forEachIndexed { i, layer ->
                    drawBand(layer, bands[i], shades[i], clouds[i].value, phase(t, layer.cycleMs))
                }
                drawWing(
                    WingColors(wing.value, wingEdge.value, wingShade.value),
                    navOn = phase(t, BLINK_MS) < NAV_ON_FRACTION,
                    strobe = strobeAlpha(phase(t, STROBE_MS)),
                )
                drawGlass(glass)
            }
        },
    )
}

@Composable
private fun fade(visible: Boolean): State<Float> = animateFloatAsState(if (visible) 1f else 0f, tween(SKY_MS))

private fun SkyState.palette(): WindowSky =
    when (this) {
        SkyState.DUSK -> WindowScene.dusk
        SkyState.DAWN -> WindowScene.dawn
        SkyState.DAY -> WindowScene.day
        SkyState.NIGHT -> WindowScene.night
    }

private fun DrawScope.drawStars(
    stars: List<Triple<Float, Float, Float>>,
    alpha: Float,
) {
    val field = size.height * STAR_FIELD
    stars.forEach { (x, y, d) ->
        val center = Offset(size.width * x / PERCENT, field * y / PERCENT)
        drawCircle(WindowScene.star, d * STAR_SOLID * density, center, alpha = alpha)
    }
}

private fun DrawScope.drawMoon(
    shape: MoonShape,
    alpha: Float,
) {
    drawCircle(shape.glow, shape.glowRadius, shape.center, alpha = alpha)
    drawPath(shape.lit, WindowScene.moon, alpha = alpha)
}

private fun DrawScope.drawSun(
    alpha: Float,
    high: Float,
) {
    val rise = (1f - alpha) * SUN_RISE * density
    val center = Offset((SUN_X + high * SUN_TRAVEL_X) * density, (SUN_Y - high * SUN_TRAVEL_Y) * density + rise)
    val far = SUN_FAR_GLOW * density
    val near = SUN_NEAR_GLOW * density
    drawCircle(glow(FAR_GLOW_LOW + high * FAR_GLOW_GAIN, center, far), far, center, alpha = alpha)
    drawCircle(glow(NEAR_GLOW_LOW + high * NEAR_GLOW_GAIN, center, near), near, center, alpha = alpha)
    val r = SUN_RADIUS * SQRT2 * density
    drawCircle(
        Brush.radialGradient(
            0f to WindowScene.sunCore,
            SUN_CORE_STOP to WindowScene.sunCore,
            SUN_HALO_STOP to WindowScene.sunHalo,
            SUN_EDGE_STOP to Color.Transparent,
            center = center,
            radius = r,
        ),
        r,
        center,
        alpha = alpha,
    )
}

private fun DrawScope.drawBand(
    layer: CloudLayer,
    band: androidx.compose.ui.graphics.Path,
    shade: Brush,
    color: Color,
    drift: Float,
) {
    val period = layer.period * density
    val top = size.height - layer.height * density
    var start = drift * period - period
    while (start < size.width) {
        translate(start, top) {
            drawPath(band, color, alpha = layer.alpha)
            drawPath(band, shade, alpha = layer.alpha)
        }
        start += period
    }
}

/** The wing's 344x206 drawing, scaled to cover the window and pinned to its bottom-right corner. */
private fun DrawScope.drawWing(
    colors: WingColors,
    navOn: Boolean,
    strobe: Float,
) {
    val k = max(size.width / WING_W, size.height / WING_H)
    translate(size.width - WING_W * k, size.height - WING_H * k) {
        scale(k, k, Offset.Zero) {
            drawPath(WING_BODY, colors.body)
            drawPath(WING_SHADE, colors.shade)
            drawPath(WING_FIN, colors.body)
            val stroke = Stroke(width = WING_EDGE_WIDTH, cap = StrokeCap.Round)
            drawPath(WING_EDGE, colors.edge, alpha = 0.85f, style = stroke)
            drawPath(FIN_EDGE, colors.edge, alpha = 0.85f * 0.55f, style = stroke)
            val lit = if (navOn) 1f else NAV_OFF_ALPHA
            drawCircle(WindowScene.navLight.copy(alpha = 0.35f), NAV_GLOW, NAV_LIGHT, alpha = lit)
            drawCircle(WindowScene.navLight, NAV_RADIUS, NAV_LIGHT, alpha = lit)
            if (strobe > 0f) drawCircle(WindowScene.strobe, STROBE_RADIUS, STROBE_LIGHT, alpha = strobe)
        }
    }
}

private class WingColors(
    val body: Color,
    val edge: Color,
    val shade: Color,
)

private fun glow(
    alpha: Float,
    center: Offset,
    radius: Float,
): Brush = Brush.radialGradient(listOf(Color.White.copy(alpha = alpha), Color.Transparent), center, radius)

private fun strobeAlpha(t: Float): Float =
    when {
        t < STROBE_START -> 0f
        t < STROBE_PEAK -> (t - STROBE_START) / (STROBE_PEAK - STROBE_START)
        t < STROBE_END -> 1f - (t - STROBE_PEAK) / (STROBE_END - STROBE_PEAK)
        else -> 0f
    }

private fun DrawScope.drawGlass(glass: GlassBrushes) {
    drawRect(glass.streak)
    drawRect(glass.faintStreak)
    drawRect(glass.innerShadow)
    drawRect(WindowScene.glassEdge, style = Stroke(density))
}

private const val SKY_MS = 1_400
private const val DUSK_STARS = 0.8f
private const val PERCENT = 100
private const val FAR_GLOW_LOW = 0.18f
private const val FAR_GLOW_GAIN = 0.1f
private const val NEAR_GLOW_LOW = 0.4f
private const val NEAR_GLOW_GAIN = 0.2f
private const val SUN_CORE_STOP = 0.46f
private const val SUN_HALO_STOP = 0.64f
private const val SUN_EDGE_STOP = 0.72f
private const val NAV_OFF_ALPHA = 0.25f
private const val STROBE_START = 0.88f
private const val STROBE_PEAK = 0.90f
private const val STROBE_END = 0.94f
private const val SUN_MS = 1_800
private const val TWINKLE_A_MS = 3_800
private const val TWINKLE_B_MS = 5_200
private const val BLINK_MS = 1_600
private const val STROBE_MS = 2_400
private const val NAV_ON_FRACTION = 0.56f
private const val STAR_FIELD = 0.62f
private const val STAR_SOLID = 0.6f
private const val SQRT2 = 1.4142f
private const val SUN_X = 75f
private const val SUN_Y = 135f
private const val SUN_TRAVEL_X = 170f
private const val SUN_TRAVEL_Y = 84f
private const val SUN_RISE = 30f
private const val SUN_RADIUS = 17f
private const val SUN_NEAR_GLOW = 52f
private const val SUN_FAR_GLOW = 110f
