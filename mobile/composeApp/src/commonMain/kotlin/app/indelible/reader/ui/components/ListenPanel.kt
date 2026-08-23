package app.indelible.reader.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Bedtime
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.KeyboardArrowUp
import androidx.compose.material.icons.filled.Pause
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.RecordVoiceOver
import androidx.compose.material.icons.filled.Replay
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.reader.playback.PlaybackState
import app.indelible.reader.playback.ReaderVoice
import app.indelible.reader.playback.StubPlaybackController
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_action_back_15_seconds
import indelible.composeapp.generated.resources.reader_action_forward_15_seconds
import indelible.composeapp.generated.resources.reader_action_pause
import indelible.composeapp.generated.resources.reader_action_play
import indelible.composeapp.generated.resources.reader_sleep
import indelible.composeapp.generated.resources.reader_sleep_minutes
import indelible.composeapp.generated.resources.reader_speed_075
import indelible.composeapp.generated.resources.reader_speed_100
import indelible.composeapp.generated.resources.reader_speed_125
import indelible.composeapp.generated.resources.reader_speed_150
import indelible.composeapp.generated.resources.reader_speed_175
import indelible.composeapp.generated.resources.reader_speed_200
import indelible.composeapp.generated.resources.reader_voice_fallback
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource

// Playback speed and sleep-timer option menus: the values are the user-facing choices.
@Suppress("MagicNumber")
private val speedSteps = listOf(0.75f, 1.0f, 1.25f, 1.5f, 1.75f, 2.0f)
private val speedLabelResources =
    listOf(
        Res.string.reader_speed_075,
        Res.string.reader_speed_100,
        Res.string.reader_speed_125,
        Res.string.reader_speed_150,
        Res.string.reader_speed_175,
        Res.string.reader_speed_200,
    )

@Suppress("MagicNumber")
private val sleepSteps = listOf<Int?>(null, 15, 30, 60)

/**
 * Listen player sheet body: now-playing card, position-aware waveform, scrubber,
 * transport, and chips for speed / voice / sleep timer with an expandable voice
 * list. Stateless beyond the local voice-list expansion — every control reports
 * back through a callback so the [app.indelible.reader.playback.ReaderPlaybackController]
 * stays the single source of truth.
 */
@Composable
fun ListenPanel(
    title: String,
    source: String,
    state: PlaybackState,
    voices: List<ReaderVoice>,
    onTogglePlay: () -> Unit,
    onSeek: (Long) -> Unit,
    onSkip: (Long) -> Unit,
    onSetSpeed: (Float) -> Unit,
    onSelectVoice: (String) -> Unit,
    onSetSleepTimer: (Int?) -> Unit,
    modifier: Modifier = Modifier,
) {
    var voicesExpanded by remember { mutableStateOf(false) }
    val currentVoiceName =
        voices
            .firstOrNull { it.id == state.voiceId }
            ?.let { stringResource(it.nameRes) }
            ?: stringResource(Res.string.reader_voice_fallback)

    Column(modifier = modifier.fillMaxWidth()) {
        NowPlayingCard(title = title, source = source)

        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

        WaveformBars(
            progressFraction = state.progressFraction,
            playing = state.isPlaying,
            modifier = Modifier.fillMaxWidth().height(IndelibleSpacing.step40),
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

        Slider(
            value = state.progressFraction,
            onValueChange = { onSeek((it * state.durationMs).toLong()) },
            valueRange = 0f..1f,
            modifier = Modifier.fillMaxWidth(),
        )
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Text(
                text = formatPlaybackTime(state.positionMs),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.primary,
            )
            Text(
                text = formatPlaybackTime(state.durationMs),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

        TransportRow(
            isPlaying = state.isPlaying,
            onSkipBack = { onSkip(-SKIP_MS) },
            onTogglePlay = onTogglePlay,
            onSkipForward = { onSkip(SKIP_MS) },
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            PlayerChip(
                label = stringResource(speedLabelResourceFor(state.speed)),
                accent = true,
                onClick = { onSetSpeed(nextSpeed(state.speed)) },
            )
            PlayerChip(
                label = currentVoiceName,
                leadingIcon = Icons.Filled.RecordVoiceOver,
                trailingIcon =
                    if (voicesExpanded) Icons.Filled.KeyboardArrowUp else Icons.Filled.KeyboardArrowDown,
                onClick = { voicesExpanded = !voicesExpanded },
            )
            PlayerChip(
                label =
                    state.sleepTimerMinutes?.let {
                        pluralStringResource(Res.plurals.reader_sleep_minutes, it, it)
                    } ?: stringResource(Res.string.reader_sleep),
                leadingIcon = Icons.Filled.Bedtime,
                accent = state.sleepTimerMinutes != null,
                onClick = { onSetSleepTimer(nextSleep(state.sleepTimerMinutes)) },
            )
        }

        AnimatedVisibility(visible = voicesExpanded) {
            Column {
                Spacer(modifier = Modifier.height(IndelibleSpacing.step12))
                VoiceList(
                    voices = voices,
                    selectedVoiceId = state.voiceId,
                    onVoiceSelected = onSelectVoice,
                )
            }
        }
    }
}

@Composable
private fun NowPlayingCard(
    title: String,
    source: String,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier =
                Modifier
                    .size(IndelibleSpacing.step56)
                    .clip(IndelibleShape.lg)
                    .background(IndelibleTheme.colors.tagColors.first()),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = source.take(1).uppercase(), // i18n-ignore: user-provided source initial
                style = MaterialTheme.typography.titleLarge,
                color = IndelibleTheme.colors.onSuccess,
            )
        }
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = source,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = title,
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.onSurface,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
private fun TransportRow(
    isPlaying: Boolean,
    onSkipBack: () -> Unit,
    onTogglePlay: () -> Unit,
    onSkipForward: () -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step28, Alignment.CenterHorizontally),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        SkipButton(forward = false, onClick = onSkipBack)
        PlayButton(isPlaying = isPlaying, onClick = onTogglePlay)
        SkipButton(forward = true, onClick = onSkipForward)
    }
}

@Composable
private fun SkipButton(
    forward: Boolean,
    onClick: () -> Unit,
) {
    Box(
        modifier =
            Modifier
                .size(IndelibleSpacing.touchTarget)
                .clip(CircleShape)
                .clickable(
                    onClickLabel =
                        stringResource(
                            if (forward) {
                                Res.string.reader_action_forward_15_seconds
                            } else {
                                Res.string.reader_action_back_15_seconds
                            },
                        ),
                    onClick = onClick,
                ),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            imageVector = Icons.Filled.Replay,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurface,
            modifier =
                Modifier
                    .size(IndelibleSpacing.step32)
                    .then(if (forward) Modifier.scale(scaleX = -1f, scaleY = 1f) else Modifier),
        )
        Text(
            text = "15",
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurface,
        )
    }
}

@Composable
private fun PlayButton(
    isPlaying: Boolean,
    onClick: () -> Unit,
) {
    val gradient =
        Brush.linearGradient(
            listOf(MaterialTheme.colorScheme.primary, IndelibleTheme.colors.accentStrong),
        )
    Box(
        modifier =
            Modifier
                .size(IndelibleSpacing.step64)
                .clip(CircleShape)
                .background(gradient)
                .clickable(
                    onClickLabel =
                        stringResource(
                            if (isPlaying) Res.string.reader_action_pause else Res.string.reader_action_play,
                        ),
                    onClick = onClick,
                ),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            imageVector = if (isPlaying) Icons.Filled.Pause else Icons.Filled.PlayArrow,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onPrimary,
            modifier = Modifier.size(IndelibleSpacing.step32),
        )
    }
}

@Composable
private fun PlayerChip(
    label: String,
    onClick: () -> Unit,
    accent: Boolean = false,
    leadingIcon: androidx.compose.ui.graphics.vector.ImageVector? = null,
    trailingIcon: androidx.compose.ui.graphics.vector.ImageVector? = null,
) {
    val background =
        if (accent) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceVariant
    val foreground =
        if (accent) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
    Row(
        modifier =
            Modifier
                .clip(IndelibleShape.full)
                .background(background)
                .clickable(onClick = onClick)
                .padding(horizontal = IndelibleSpacing.step12, vertical = IndelibleSpacing.step8),
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step4),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (leadingIcon != null) {
            Icon(
                imageVector = leadingIcon,
                contentDescription = null,
                tint = foreground,
                modifier = Modifier.size(IndelibleSpacing.step16),
            )
        }
        Text(text = label, style = MaterialTheme.typography.labelMedium, color = foreground)
        if (trailingIcon != null) {
            Icon(
                imageVector = trailingIcon,
                contentDescription = null,
                tint = foreground,
                modifier = Modifier.size(IndelibleSpacing.step16),
            )
        }
    }
}

private fun speedLabelResourceFor(speed: Float): StringResource {
    val index = speedSteps.indexOfFirst { kotlin.math.abs(it - speed) < SPEED_EPSILON }
    return speedLabelResources[index.coerceIn(0, speedLabelResources.lastIndex)]
}

private fun nextSpeed(current: Float): Float {
    val index = speedSteps.indexOfFirst { kotlin.math.abs(it - current) < SPEED_EPSILON }
    val safe = if (index < 0) speedSteps.indexOf(1.0f) else index
    return speedSteps[(safe + 1) % speedSteps.size]
}

private fun nextSleep(current: Int?): Int? {
    val index = sleepSteps.indexOf(current).coerceAtLeast(0)
    return sleepSteps[(index + 1) % sleepSteps.size]
}

private const val MS_PER_SECOND = 1000L
private const val SECONDS_PER_MINUTE = 60L

private fun formatPlaybackTime(ms: Long): String {
    val totalSeconds = (ms / MS_PER_SECOND).coerceAtLeast(0L)
    val minutes = totalSeconds / SECONDS_PER_MINUTE
    val seconds = totalSeconds % SECONDS_PER_MINUTE
    return "$minutes:${seconds.toString().padStart(2, '0')}"
}

private const val SKIP_MS = 15_000L
private const val SPEED_EPSILON = 0.01f

@Preview
@Composable
private fun ListenPanelPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            ListenPanel(
                title = "The End of the Beginning",
                source = "Stratechery",
                state = PlaybackState(isPlaying = true, positionMs = 368_000, speed = 1.0f),
                voices = StubPlaybackController.VOICES,
                onTogglePlay = {},
                onSeek = {},
                onSkip = {},
                onSetSpeed = {},
                onSelectVoice = {},
                onSetSleepTimer = {},
                modifier = Modifier.padding(IndelibleSpacing.screenPaddingH),
            )
        }
    }
}

@Preview
@Composable
private fun ListenPanelPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            ListenPanel(
                title = "The End of the Beginning",
                source = "Stratechery",
                state = PlaybackState(isPlaying = false, positionMs = 90_000, speed = 1.5f, sleepTimerMinutes = 30),
                voices = StubPlaybackController.VOICES,
                onTogglePlay = {},
                onSeek = {},
                onSkip = {},
                onSetSpeed = {},
                onSelectVoice = {},
                onSetSleepTimer = {},
                modifier = Modifier.padding(IndelibleSpacing.screenPaddingH),
            )
        }
    }
}
