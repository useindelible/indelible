package app.indelible.home.ui.offline

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import app.indelible.ui.theme.CoverArt
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import app.indelible.ui.theme.SerifFontFamily
import app.indelible.ui.theme.geistMonoFontFamily
import coil3.compose.AsyncImage

/**
 * An item as a book on the shelf: its own [imageUrl] when it has one and it loads, else printed cover art,
 * under a shaded spine and a gloss, over a page block and a drop shadow. [tilt] is degrees.
 */
@Composable
fun ShelfCover(
    documentId: String,
    kind: String,
    title: String,
    kept: Boolean,
    width: Dp,
    height: Dp,
    imageUrl: String?,
    modifier: Modifier = Modifier,
    tilt: Float = 0f,
) {
    var artworkShown by remember(imageUrl) { mutableStateOf(false) }
    val style = CoverStyle.entries[documentId.hashCode().mod(CoverStyle.entries.size)]
    val shape = RoundedCornerShape(topStart = 3.dp, bottomStart = 3.dp, topEnd = 8.dp, bottomEnd = 8.dp)
    val ink = if (style == CoverStyle.BONE) CoverArt.paperInk else CoverArt.ink
    Box(
        modifier =
            modifier
                .size(width + PAGE_DEPTH, height + PAGE_DEPTH)
                .graphicsLayer {
                    rotationZ = tilt
                    rotationY = TURN
                    cameraDistance = CAMERA * density
                },
    ) {
        Box(
            Modifier
                .offset(PAGE_DEPTH, PAGE_DEPTH / 2)
                .size(width, height)
                .clip(shape)
                .background(CoverArt.pages)
                .drawWithContent { drawPageEdges() },
        )
        Box(
            Modifier
                .size(width, height)
                .shadow(COVER_ELEVATION, shape, ambientColor = Color.Black, spotColor = Color.Black)
                .clip(shape)
                .drawWithContent {
                    drawArt(style)
                    drawSpineAndGloss()
                }.border(1.dp, Color.White.copy(alpha = 0.08f), shape),
        ) {
            if (imageUrl != null) {
                AsyncImage(
                    model = imageUrl,
                    contentDescription = null,
                    contentScale = ContentScale.Crop,
                    onSuccess = { artworkShown = true },
                    modifier = Modifier.fillMaxSize(),
                )
            }
            if (!artworkShown) CoverText(kind, title, style, ink)
            if (kept) KeptRibbon(Modifier.align(Alignment.TopEnd).offset(x = (-10).dp))
        }
    }
}

@Composable
private fun CoverText(
    kind: String,
    title: String,
    style: CoverStyle,
    ink: Color,
) {
    Column(Modifier.fillMaxSize().padding(start = IndelibleSpacing.step12, end = 9.dp, top = 9.dp, bottom = 10.dp)) {
        Text(
            text = kind,
            style = MaterialTheme.typography.labelSmall.copy(fontFamily = geistMonoFontFamily()),
            color = ink.copy(alpha = 0.8f),
            maxLines = 1,
        )
        Spacer(Modifier.weight(1f))
        Text(
            text = title,
            style =
                MaterialTheme.typography.titleSmall.copy(
                    fontFamily = SerifFontFamily,
                    fontWeight = FontWeight.Normal,
                    fontStyle = if (style == CoverStyle.OXBLOOD) FontStyle.Italic else FontStyle.Normal,
                ),
            color = ink,
            maxLines = 3,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
private fun KeptRibbon(modifier: Modifier) {
    val mono = IndelibleTheme.monochrome
    Canvas(modifier.size(13.dp, 19.dp).clip(RIBBON)) {
        drawRect(Brush.verticalGradient(listOf(mono.boneTop, mono.boneMid, mono.boneBottom)))
    }
}

private val COVER_ELEVATION = 10.dp
private const val TURN = -8f
private const val CAMERA = 14f
