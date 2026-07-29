package app.indelible.reader.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import app.indelible.ui.theme.SerifFontFamily
import app.indelible.ui.theme.geistMonoFontFamily
import coil3.compose.AsyncImage

/**
 * Tappable video thumbnail that stands in for the provider embed (which the reader hides).
 * Identity below it — title, then channel with views and duration — is rendered by the
 * document itself, so this composable deliberately stops at the poster frame.
 */
@Composable
fun YouTubeVideoHeader(
    thumbnailUrl: String?,
    durationSeconds: Int?,
    onPlayTapped: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier) {
        Box(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .aspectRatio(VIDEO_ASPECT_RATIO)
                    .background(MaterialTheme.colorScheme.surfaceVariant)
                    .clickable(onClick = onPlayTapped),
            contentAlignment = Alignment.Center,
        ) {
            if (!thumbnailUrl.isNullOrBlank()) {
                AsyncImage(
                    model = thumbnailUrl,
                    contentDescription = null,
                    contentScale = ContentScale.Crop,
                    modifier = Modifier.fillMaxSize(),
                )
            }
            Box(
                modifier =
                    Modifier
                        .fillMaxSize()
                        .background(Color.Black.copy(alpha = SCRIM_ALPHA)),
            )
            Box(
                modifier =
                    Modifier
                        .size(IndelibleSpacing.step64)
                        .background(
                            color = Color.White.copy(alpha = PLAY_BUTTON_ALPHA),
                            shape = IndelibleShape.full,
                        ),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    imageVector = Icons.Filled.PlayArrow,
                    contentDescription = "Play on YouTube",
                    tint = Color.White,
                    modifier = Modifier.size(IndelibleSpacing.step40),
                )
            }
            if (durationSeconds != null) {
                Text(
                    text = formatDuration(durationSeconds),
                    style = MaterialTheme.typography.bodySmall,
                    color = Color.White,
                    modifier =
                        Modifier
                            .align(Alignment.BottomEnd)
                            .padding(IndelibleSpacing.step8)
                            .background(
                                color = Color.Black.copy(alpha = DURATION_BG_ALPHA),
                                shape = IndelibleShape.xs,
                            ).padding(
                                horizontal = IndelibleSpacing.step4,
                                vertical = IndelibleSpacing.step2,
                            ),
                )
            }
        }

    }
}

private const val VIDEO_ASPECT_RATIO = 16f / 9f
private const val SCRIM_ALPHA = 0.28f
private const val PLAY_BUTTON_ALPHA = 0.22f
private const val DURATION_BG_ALPHA = 0.7f
private const val SECONDS_PER_HOUR = 3600
private const val SECONDS_PER_MINUTE = 60

private fun formatDuration(seconds: Int): String {
    val h = seconds / SECONDS_PER_HOUR
    val m = (seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE
    val s = seconds % SECONDS_PER_MINUTE

    fun Int.pad2() = toString().padStart(2, '0')
    return if (h > 0) {
        "$h:${m.pad2()}:${s.pad2()}"
    } else {
        "$m:${s.pad2()}"
    }
}

@Preview(showBackground = true)
@Composable
private fun YouTubeVideoHeaderPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            YouTubeVideoHeader(
                thumbnailUrl = null,
                durationSeconds = 1203,
                onPlayTapped = {},
            )
        }
    }
}

@Preview(showBackground = true)
@Composable
private fun YouTubeVideoHeaderPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            YouTubeVideoHeader(
                thumbnailUrl = null,
                durationSeconds = 273,
                onPlayTapped = {},
            )
        }
    }
}
