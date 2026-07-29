package app.indelible.reader.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.core.platform.platformClientType
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme

/**
 * The foot of the reader: the article title standing on a shelf, with reading
 * position as a hairline just inside the shelf's top edge.
 *
 * The title stands here rather than riding a top bar — a standing label, not a
 * heading. Both survive immersive mode: when the furniture leaves, what you are
 * reading and how far in you are are the two things worth keeping.
 *
 * The shelf is solid from just below its top edge rather than a long feather. A
 * gradient tall enough to mask the article would have to pass through the dock,
 * which leaves the dock floating inside its own fade.
 */
@Composable
fun ReaderBottomChrome(
    title: String,
    progress: Float,
    modifier: Modifier = Modifier,
) {
    val page = IndelibleTheme.colors.readerBg
    val bottomInsetModifier =
        if (platformClientType() == "ios") {
            Modifier.padding(bottom = IndelibleSpacing.step4)
        } else {
            Modifier.navigationBarsPadding()
        }
    Column(
        modifier =
            modifier
                .fillMaxWidth()
                .background(
                    Brush.verticalGradient(
                        0f to Color.Transparent,
                        SHELF_EDGE_STOP to page,
                        1f to page,
                    ),
                ),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        ReaderProgressBarV2(progress = progress)
        Text(
            text = title,
            style = MaterialTheme.typography.bodySmall,
            // --muted, not --faint: the fainter token measures 2.7:1 on paper and fails.
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            textAlign = TextAlign.Center,
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(
                        start = IndelibleSpacing.step20,
                        end = IndelibleSpacing.step20,
                        top = IndelibleSpacing.step12,
                        bottom = IndelibleSpacing.step4,
                    ),
        )
        // Preserve the approved shelf position while keeping the progress hairline
        // away from system chrome at the physical screen edge.
        Spacer(modifier = bottomInsetModifier)
    }
}

// Where the shelf reaches full page colour. Short enough that the edge lands under
// the dock, so what reads is a shelf the dock sits on rather than a smear.
private const val SHELF_EDGE_STOP = 0.12f

@Preview
@Composable
private fun ReaderBottomChromeLightPreview() {
    AppTheme(darkTheme = false) {
        Surface {
            ReaderBottomChrome(
                title = "The quiet work of finishing things",
                progress = 42f,
            )
        }
    }
}

@Preview
@Composable
private fun ReaderBottomChromeDarkPreview() {
    AppTheme(darkTheme = true) {
        Surface {
            ReaderBottomChrome(
                title = "A title long enough that it has to ellipsise at the foot of the screen",
                progress = 78f,
            )
        }
    }
}
