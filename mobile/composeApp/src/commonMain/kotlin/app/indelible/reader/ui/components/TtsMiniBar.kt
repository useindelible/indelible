package app.indelible.reader.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.KeyboardArrowUp
import androidx.compose.material.icons.filled.Pause
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_action_open_player
import indelible.composeapp.generated.resources.reader_action_pause
import indelible.composeapp.generated.resources.reader_action_play
import indelible.composeapp.generated.resources.reader_action_stop
import indelible.composeapp.generated.resources.reader_now_playing
import org.jetbrains.compose.resources.stringResource

/**
 * Non-modal floating player pill shown while narration plays and the Listen sheet
 * is closed. The article keeps scrolling beneath it. Tapping the body or the
 * chevron expands the full Listen panel; the cross stops playback.
 */
@Composable
fun TtsMiniBar(
    title: String,
    voiceName: String,
    isPlaying: Boolean,
    progressFraction: Float,
    onTogglePlay: () -> Unit,
    onExpand: () -> Unit,
    onClose: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier.fillMaxWidth(),
        shape = IndelibleShape.full,
        color = MaterialTheme.colorScheme.surfaceContainer,
        shadowElevation = IndelibleSpacing.step8,
    ) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .clickable(
                        onClickLabel = stringResource(Res.string.reader_action_open_player),
                        onClick = onExpand,
                    ).padding(horizontal = IndelibleSpacing.step8, vertical = IndelibleSpacing.step6),
            horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                modifier =
                    Modifier
                        .size(IndelibleSpacing.touchTarget)
                        .clip(IndelibleShape.md)
                        .background(IndelibleTheme.colors.tagColors.first()),
                contentAlignment = Alignment.Center,
            ) {
                WaveformBars(
                    progressFraction = progressFraction,
                    playing = isPlaying,
                    modifier = Modifier.width(IndelibleSpacing.step28).height(IndelibleSpacing.step16),
                    barCount = 5,
                )
            }
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = stringResource(Res.string.reader_now_playing, voiceName),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.primary,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleSmall,
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            MiniBarButton(
                icon = if (isPlaying) Icons.Filled.Pause else Icons.Filled.PlayArrow,
                contentDescription =
                    stringResource(
                        if (isPlaying) Res.string.reader_action_pause else Res.string.reader_action_play,
                    ),
                tint = MaterialTheme.colorScheme.primary,
                onClick = onTogglePlay,
            )
            MiniBarButton(
                icon = Icons.Filled.KeyboardArrowUp,
                contentDescription = stringResource(Res.string.reader_action_open_player),
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
                onClick = onExpand,
            )
            MiniBarButton(
                icon = Icons.Filled.Close,
                contentDescription = stringResource(Res.string.reader_action_stop),
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
                onClick = onClose,
            )
        }
    }
}

@Composable
private fun MiniBarButton(
    icon: ImageVector,
    contentDescription: String,
    tint: androidx.compose.ui.graphics.Color,
    onClick: () -> Unit,
) {
    Box(
        modifier =
            Modifier
                .size(IndelibleSpacing.touchTarget)
                .clip(IndelibleShape.full)
                .clickable(onClickLabel = contentDescription, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = tint,
            modifier = Modifier.size(IndelibleSpacing.step24),
        )
    }
}

@Preview
@Composable
private fun TtsMiniBarPreviewLight() {
    AppTheme(darkTheme = false) {
        TtsMiniBar(
            title = "The End of the Beginning",
            voiceName = "Ava",
            isPlaying = true,
            progressFraction = 0.4f,
            onTogglePlay = {},
            onExpand = {},
            onClose = {},
            modifier = Modifier.padding(IndelibleSpacing.step16),
        )
    }
}

@Preview
@Composable
private fun TtsMiniBarPreviewDark() {
    AppTheme(darkTheme = true) {
        TtsMiniBar(
            title = "The End of the Beginning",
            voiceName = "Ren",
            isPlaying = false,
            progressFraction = 0.7f,
            onTogglePlay = {},
            onExpand = {},
            onClose = {},
            modifier = Modifier.padding(IndelibleSpacing.step16),
        )
    }
}
