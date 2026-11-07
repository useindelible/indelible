package app.indelible.home.ui.offline

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.MonochromeColors
import app.indelible.ui.theme.SerifFontFamily
import app.indelible.ui.theme.geistMonoFontFamily

/** Polished black (or porcelain): a sheen that fades by mid-card, a specular top edge and a grounded shadow. */
@Composable
internal fun Modifier.polished(
    mono: MonochromeColors,
    shape: Shape,
    fill: Color = mono.surface,
): Modifier =
    this
        .shadow(CARD_ELEVATION, shape, ambientColor = mono.dropShadow, spotColor = mono.dropShadow)
        .clip(shape)
        .background(fill)
        .background(gloss(mono))
        .border(1.dp, mono.line, shape)
        .drawWithContent {
            drawContent()
            drawLine(mono.specular, Offset(0f, SPECULAR_Y), Offset(size.width, SPECULAR_Y), 1.dp.toPx())
        }

private fun gloss(mono: MonochromeColors): Brush =
    Brush.verticalGradient(0f to mono.glossTop, GLOSS_MID to mono.glossMid, mono.glossEnd to Color.Transparent)

/** The polished primary fill: bone in dark, lacquer black in light. */
internal fun bone(mono: MonochromeColors): Brush =
    Brush.verticalGradient(0f to mono.boneTop, BONE_MID to mono.boneMid, 1f to mono.boneBottom)

@Composable
internal fun serif(
    base: TextStyle,
    weight: FontWeight = FontWeight.Light,
): TextStyle = base.copy(fontFamily = SerifFontFamily, fontWeight = weight)

@Composable
internal fun mono(base: TextStyle = MaterialTheme.typography.labelSmall): TextStyle {
    val family = geistMonoFontFamily()
    return base.copy(fontFamily = family)
}

internal enum class PillTone { OK, WAIT, LIVE }

@Composable
internal fun Pill(
    text: String,
    icon: ImageVector,
    tone: PillTone,
    mono: MonochromeColors,
) {
    val (ink, wash) =
        when (tone) {
            PillTone.OK -> mono.success to mono.successWash
            PillTone.WAIT -> mono.warning to mono.warningWash
            PillTone.LIVE -> mono.ink to mono.accentWash
        }
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step6),
        modifier =
            Modifier
                .height(PILL_HEIGHT)
                .clip(IndelibleShape.full)
                .background(wash)
                .border(1.dp, ink.copy(alpha = 0.25f), IndelibleShape.full)
                .padding(start = 9.dp, end = 11.dp),
    ) {
        Icon(icon, contentDescription = null, tint = ink, modifier = Modifier.size(13.dp))
        Text(text, style = mono(), color = ink)
    }
}

private val CARD_ELEVATION = 14.dp
private val PILL_HEIGHT = 28.dp

private const val GLOSS_MID = 0.42f
private const val BONE_MID = 0.55f
private const val SPECULAR_Y = 0.5f
