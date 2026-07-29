package app.indelible.reader.ui.components

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import kotlin.math.PI
import kotlin.math.abs
import kotlin.math.sin

/**
 * Decorative audio waveform: rounded bars where the played region is tinted with
 * the accent and the remainder is muted. Bars gently breathe while [playing].
 * Purely visual — it reflects [progressFraction], it does not control playback.
 */
private const val DEFAULT_BAR_COUNT = 44
private const val WAVEFORM_PHASE_MS = 1100
private const val BAR_WIDTH_FRACTION = 0.5f
private const val BASELINE_HEIGHT = 0.30f
private const val BASELINE_AMPLITUDE = 0.55f
private const val BAR_FREQUENCY = 0.7f
private const val PULSE_BASE = 0.70f
private const val PULSE_AMPLITUDE = 0.30f
private const val PULSE_PHASE_STEP = 0.5f
private const val MIN_BAR_HEIGHT_FRACTION = 0.12f

@Composable
fun WaveformBars(
    progressFraction: Float,
    playing: Boolean,
    modifier: Modifier = Modifier,
    barCount: Int = DEFAULT_BAR_COUNT,
) {
    val accent = MaterialTheme.colorScheme.primary
    val muted = MaterialTheme.colorScheme.outlineVariant
    val transition = rememberInfiniteTransition(label = "waveform")
    val phase by transition.animateFloat(
        initialValue = 0f,
        targetValue = (2.0 * PI).toFloat(),
        animationSpec =
            infiniteRepeatable(
                animation = tween(durationMillis = WAVEFORM_PHASE_MS, easing = LinearEasing),
                repeatMode = RepeatMode.Restart,
            ),
        label = "phase",
    )

    Canvas(modifier) {
        val slot = size.width / barCount
        val barWidth = slot * BAR_WIDTH_FRACTION
        val playedBars = progressFraction * barCount
        for (i in 0 until barCount) {
            val baseline = BASELINE_HEIGHT + BASELINE_AMPLITUDE * abs(sin(i * BAR_FREQUENCY))
            val pulse = if (playing) PULSE_BASE + PULSE_AMPLITUDE * sin(phase + i * PULSE_PHASE_STEP) else 1f
            val barHeight = (size.height * baseline * pulse).coerceIn(
                    size.height * MIN_BAR_HEIGHT_FRACTION,
                    size.height,
                )
            val x = i * slot + (slot - barWidth) / 2f
            val top = (size.height - barHeight) / 2f
            drawRoundRect(
                color = if (i <= playedBars) accent else muted,
                topLeft = Offset(x, top),
                size = Size(barWidth, barHeight),
                cornerRadius = CornerRadius(barWidth / 2f, barWidth / 2f),
            )
        }
    }
}

@Preview
@Composable
private fun WaveformBarsPreviewLight() {
    AppTheme(darkTheme = false) {
        WaveformBars(
            progressFraction = 0.4f,
            playing = true,
            modifier = Modifier.fillMaxWidth().height(IndelibleSpacing.step40),
        )
    }
}

@Preview
@Composable
private fun WaveformBarsPreviewDark() {
    AppTheme(darkTheme = true) {
        WaveformBars(
            progressFraction = 0.65f,
            playing = false,
            modifier = Modifier.fillMaxWidth().height(IndelibleSpacing.step40),
        )
    }
}
