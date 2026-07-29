package app.indelible.reader.playback

import kotlinx.coroutines.flow.StateFlow

/** One synthetic narration voice the Listen player can read with. */
data class ReaderVoice(
    val id: String,
    val name: String,
    val tagline: String,
)

/**
 * Immutable snapshot of the Listen player. Position and duration are milliseconds.
 * [currentSentenceIndex] tracks which `.say` span the WebView should highlight.
 */
data class PlaybackState(
    val isPlaying: Boolean = false,
    val positionMs: Long = 0,
    val durationMs: Long = DEFAULT_DURATION_MS,
    val speed: Float = 1.0f,
    val voiceId: String = DEFAULT_VOICE_ID,
    val sleepTimerMinutes: Int? = null,
    val currentSentenceIndex: Int = 0,
) {
    /** Fraction in 0..1 for scrubber/waveform rendering. */
    val progressFraction: Float
        get() = if (durationMs <= 0L) 0f else (positionMs.toFloat() / durationMs).coerceIn(0f, 1f)

    companion object {
        const val DEFAULT_DURATION_MS = 8L * 60L * 1000L
        const val DEFAULT_VOICE_ID = "ava"
    }
}

/**
 * Drives the reader Listen player. The contract is intentionally narrow so a real
 * audio backend can replace the shipped [StubPlaybackController] without touching
 * the UI. There is no real audio yet — see that class.
 */
interface ReaderPlaybackController {
    val state: StateFlow<PlaybackState>
    val voices: List<ReaderVoice>

    fun play()

    fun pause()

    fun togglePlayPause()

    fun seekTo(ms: Long)

    fun skip(deltaMs: Long)

    fun setSpeed(speed: Float)

    fun selectVoice(voiceId: String)

    fun setSleepTimer(minutes: Int?)

    fun release()
}
