package app.indelible.ui.components

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import kotlin.math.min

@Composable
fun IndelibleVaultMark(
    modifier: Modifier = Modifier,
    frameColor: Color = MaterialTheme.colorScheme.primary,
    detailColor: Color = MaterialTheme.colorScheme.onPrimary,
) {
    Canvas(modifier = modifier) {
        val side = min(size.width, size.height)
        val scale = side / 200f
        val origin = Offset((size.width - side) / 2f, (size.height - side) / 2f)
        fun point(x: Float, y: Float) = Offset(origin.x + x * scale, origin.y + y * scale)
        fun length(value: Float) = value * scale
        val center = point(100f, 100f)

        drawRoundRect(
            color = frameColor,
            topLeft = point(20f, 20f),
            size = Size(length(160f), length(160f)),
            cornerRadius = CornerRadius(length(42f), length(42f)),
        )

        drawCircle(
            color = detailColor.copy(alpha = 0.32f),
            radius = length(64f),
            center = center,
            style = Stroke(width = length(1.5f)),
        )

        listOf(
            point(100f, 46f),
            point(138f, 62f),
            point(154f, 100f),
            point(138f, 138f),
            point(100f, 154f),
            point(62f, 138f),
            point(46f, 100f),
            point(62f, 62f),
        ).forEach { rivetCenter ->
            drawCircle(
                color = detailColor.copy(alpha = 0.62f),
                radius = length(3.4f),
                center = rivetCenter,
            )
        }

        listOf(
            RectSpec(94f, 48f, 12f, 40f, 6f),
            RectSpec(94f, 112f, 12f, 40f, 6f),
            RectSpec(48f, 94f, 40f, 12f, 6f),
            RectSpec(112f, 94f, 40f, 12f, 6f),
        ).forEach { spoke ->
            drawRoundRect(
                color = detailColor,
                topLeft = point(spoke.x, spoke.y),
                size = Size(length(spoke.width), length(spoke.height)),
                cornerRadius = CornerRadius(length(spoke.radius), length(spoke.radius)),
            )
        }

        drawCircle(
            color = detailColor,
            radius = length(19f),
            center = center,
        )
        drawCircle(
            color = frameColor,
            radius = length(7f),
            center = center,
        )
    }
}

private data class RectSpec(
    val x: Float,
    val y: Float,
    val width: Float,
    val height: Float,
    val radius: Float,
)

@Preview(showBackground = true)
@Composable
private fun IndelibleVaultMarkPreviewLight() {
    AppTheme(darkTheme = false) {
        IndelibleVaultMark(
            modifier = Modifier.size(IndelibleSpacing.step96),
        )
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun IndelibleVaultMarkPreviewDark() {
    AppTheme(darkTheme = true) {
        IndelibleVaultMark(
            modifier = Modifier.size(IndelibleSpacing.step96),
        )
    }
}
