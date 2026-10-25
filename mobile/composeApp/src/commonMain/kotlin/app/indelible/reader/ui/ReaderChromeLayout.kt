package app.indelible.reader.ui

import androidx.compose.ui.unit.Dp
import app.indelible.reader.model.DataPanel
import app.indelible.reader.playback.PlaybackState
import app.indelible.ui.theme.IndelibleSpacing

internal fun isMiniBarShown(
    playback: PlaybackState,
    activePanel: DataPanel,
): Boolean = (playback.isPlaying || playback.positionMs > 0L) && activePanel != DataPanel.LISTEN

/** Where the chapter pill rests above the dock; [miniBarClearance] lifts it over the Listen mini-bar. */
internal fun chapterBarBottom(
    immersive: Boolean,
    miniBarClearance: Dp?,
): Dp {
    val resting = if (immersive) IndelibleSpacing.step56 else IndelibleSpacing.step48 + IndelibleSpacing.step64
    return if (miniBarClearance == null) resting else maxOf(resting, miniBarClearance + IndelibleSpacing.step8)
}
