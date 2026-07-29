package app.indelible.reader.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.reader.playback.ReaderVoice
import app.indelible.reader.playback.StubPlaybackController
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme

/** Selectable list of narration voices, each with a colored initial avatar. */
@Composable
fun VoiceList(
    voices: List<ReaderVoice>,
    selectedVoiceId: String,
    onVoiceSelected: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step4),
    ) {
        voices.forEachIndexed { index, voice ->
            VoiceRow(
                voice = voice,
                color = IndelibleTheme.colors.tagColors[index % IndelibleTheme.colors.tagColors.size],
                selected = voice.id == selectedVoiceId,
                onClick = { onVoiceSelected(voice.id) },
            )
        }
    }
}

@Composable
private fun VoiceRow(
    voice: ReaderVoice,
    color: androidx.compose.ui.graphics.Color,
    selected: Boolean,
    onClick: () -> Unit,
) {
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .clip(IndelibleShape.lg)
                .background(
                    if (selected) MaterialTheme.colorScheme.primaryContainer else Color.Transparent,
                ).clickable(onClick = onClick)
                .padding(horizontal = IndelibleSpacing.step12, vertical = IndelibleSpacing.step10),
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier = Modifier.size(IndelibleSpacing.step40).clip(CircleShape).background(color),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = voice.name.take(1),
                style = MaterialTheme.typography.titleSmall,
                color = IndelibleTheme.colors.onSuccess,
            )
        }
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = voice.name,
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = voice.tagline,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        if (selected) {
            Icon(
                imageVector = Icons.Filled.Check,
                contentDescription = "Selected",
                tint = MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(IndelibleSpacing.step20),
            )
        }
    }
}

@Preview
@Composable
private fun VoiceListPreviewLight() {
    AppTheme(darkTheme = false) {
        VoiceList(
            voices = StubPlaybackController.VOICES,
            selectedVoiceId = "ava",
            onVoiceSelected = {},
        )
    }
}

@Preview
@Composable
private fun VoiceListPreviewDark() {
    AppTheme(darkTheme = true) {
        VoiceList(
            voices = StubPlaybackController.VOICES,
            selectedVoiceId = "ren",
            onVoiceSelected = {},
        )
    }
}
