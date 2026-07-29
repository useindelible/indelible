package app.indelible.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.semantics.clearAndSetSemantics
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing

private const val SECONDARY_GHOST_ALPHA = 0.55f
private const val FIRST_LINE_WIDTH = 0.38f
private const val SECOND_LINE_WIDTH = 0.88f
private const val THIRD_LINE_WIDTH = 0.66f

fun Modifier.dashedZeroBorder(color: Color): Modifier =
    drawBehind {
        val strokeWidth = IndelibleSpacing.hairline.toPx()
        drawRoundRect(
            color = color,
            cornerRadius =
                CornerRadius(
                    x = IndelibleSpacing.step10.toPx(),
                    y = IndelibleSpacing.step10.toPx(),
                ),
            style =
                Stroke(
                    width = strokeWidth,
                    pathEffect =
                        PathEffect.dashPathEffect(
                            floatArrayOf(
                                IndelibleSpacing.step6.toPx(),
                                IndelibleSpacing.step6.toPx(),
                            ),
                        ),
                ),
        )
    }

@Composable
fun ZeroedGhostRows(
    borderColor: Color,
    lineColor: Color,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier.clearAndSetSemantics { }) {
        ZeroedGhostRow(borderColor = borderColor, lineColor = lineColor)
        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
        ZeroedGhostRow(
            borderColor = borderColor,
            lineColor = lineColor,
            modifier = Modifier.alpha(SECONDARY_GHOST_ALPHA),
        )
    }
}

@Composable
private fun ZeroedGhostRow(
    borderColor: Color,
    lineColor: Color,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .height(IndelibleSpacing.step80)
                .dashedZeroBorder(borderColor)
                .padding(IndelibleSpacing.step12),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier =
                Modifier
                    .size(IndelibleSpacing.step56)
                    .clip(IndelibleShape.lg)
                    .background(lineColor.copy(alpha = SECONDARY_GHOST_ALPHA)),
        )
        Spacer(modifier = Modifier.width(IndelibleSpacing.step14))
        Column(modifier = Modifier.weight(1f).fillMaxHeight()) {
            GhostLine(FIRST_LINE_WIDTH, lineColor)
            Spacer(modifier = Modifier.height(IndelibleSpacing.step10))
            GhostLine(SECOND_LINE_WIDTH, lineColor)
            Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
            GhostLine(THIRD_LINE_WIDTH, lineColor)
        }
    }
}

@Composable
private fun GhostLine(
    widthFraction: Float,
    color: Color,
) {
    Box(
        modifier =
            Modifier
                .fillMaxWidth(widthFraction)
                .height(IndelibleSpacing.step6)
                .clip(IndelibleShape.full)
                .background(color),
    )
}
