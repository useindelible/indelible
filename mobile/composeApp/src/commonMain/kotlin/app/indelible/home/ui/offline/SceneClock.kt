package app.indelible.home.ui.offline

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.tween
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.State
import androidx.compose.runtime.remember
import kotlin.math.abs

internal const val SCENE_MOTION_MS = 10_000
private const val TWINKLE_LOW = 0.25f

/** Motion time: runs for [SCENE_MOTION_MS] after each [key] change, then holds so a resting Home draws nothing. */
@Composable
internal fun rememberSceneClock(key: Any): State<Float> {
    val clock = remember { Animatable(0f) }
    LaunchedEffect(key) {
        clock.animateTo(clock.value + SCENE_MOTION_MS, tween(SCENE_MOTION_MS, easing = LinearEasing))
    }
    return clock.asState()
}

internal fun phase(
    timeMs: Float,
    periodMs: Int,
): Float = (timeMs % periodMs) / periodMs

/** A light that brightens and dims over [periodMs] each way, never fully out. */
internal fun twinkle(
    timeMs: Float,
    periodMs: Int,
): Float {
    val p = phase(timeMs, periodMs * 2)
    val rise = 1 - abs(p * 2 - 1)
    return TWINKLE_LOW + (1 - TWINKLE_LOW) * FastOutSlowInEasing.transform(rise)
}
