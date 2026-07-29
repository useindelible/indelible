package app.indelible.reader.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.ReaderIcons

// The pill reads over live prose rather than a blanked strip, so it carries its own
// legibility. A true backdrop blur cannot reach the article — that is a platform view
// beneath the Compose layer — so the veil does the work the blur would have.
private const val PILL_VEIL_ALPHA = 0.84f

/**
 * The reader's only persistent chrome. Replaces the top bar: navigation floats over
 * the article instead of riding a bar, so the page runs to the top edge.
 *
 * Back sits alone on the left; the right group is a tight pair. Save appears only for
 * an unsaved feed item, and everything the old top bar carried beyond that moved into
 * the item record behind [onMore].
 */
@Composable
fun ReaderFloatingControls(
    onBack: () -> Unit,
    onContents: () -> Unit,
    onMore: () -> Unit,
    modifier: Modifier = Modifier,
    showContents: Boolean = true,
    canSave: Boolean = false,
    onSave: () -> Unit = {},
) {
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .statusBarsPadding()
                .padding(
                    start = IndelibleSpacing.step12,
                    end = IndelibleSpacing.step12,
                    top = IndelibleSpacing.step16,
                    bottom = IndelibleSpacing.step8,
                ),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.Top,
    ) {
        FloatingPill(
            icon = ReaderIcons.Back,
            contentDescription = "Back",
            onClick = onBack,
        )
        Row(horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step10)) {
            if (canSave) {
                FloatingPill(
                    icon = ReaderIcons.Save,
                    contentDescription = "Save to library",
                    onClick = onSave,
                )
            }
            if (showContents) {
                FloatingPill(
                    icon = ReaderIcons.Contents,
                    contentDescription = "Contents",
                    onClick = onContents,
                )
            }
            FloatingPill(
                icon = ReaderIcons.More,
                contentDescription = "More",
                onClick = onMore,
            )
        }
    }
}

@Composable
private fun FloatingPill(
    icon: ImageVector,
    contentDescription: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier =
            modifier
                .size(IndelibleSpacing.step40)
                .defaultMinSize(minWidth = IndelibleSpacing.step40)
                .background(
                    color = MaterialTheme.colorScheme.surfaceContainer.copy(alpha = PILL_VEIL_ALPHA),
                    shape = CircleShape,
                )
                .border(
                    width = IndelibleSpacing.hairline,
                    color = MaterialTheme.colorScheme.outlineVariant,
                    shape = CircleShape,
                )
                .clickable(onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            imageVector = icon,
            contentDescription = contentDescription,
            tint = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.size(IndelibleSpacing.step20),
        )
    }
}

@Preview
@Composable
private fun ReaderFloatingControlsLightPreview() {
    AppTheme(darkTheme = false) {
        Surface(color = Color(0xFFFBFAF6)) {
            ReaderFloatingControls(
                onBack = {},
                onContents = {},
                onMore = {},
                canSave = true,
            )
        }
    }
}

@Preview
@Composable
private fun ReaderFloatingControlsDarkPreview() {
    AppTheme(darkTheme = true) {
        Surface {
            ReaderFloatingControls(
                onBack = {},
                onContents = {},
                onMore = {},
                canSave = false,
            )
        }
    }
}
