package app.indelible.reader.playback

import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_voice_ava
import indelible.composeapp.generated.resources.reader_voice_ava_tagline
import indelible.composeapp.generated.resources.reader_voice_iris
import indelible.composeapp.generated.resources.reader_voice_iris_tagline
import indelible.composeapp.generated.resources.reader_voice_mara
import indelible.composeapp.generated.resources.reader_voice_mara_tagline
import indelible.composeapp.generated.resources.reader_voice_ren
import indelible.composeapp.generated.resources.reader_voice_ren_tagline
import indelible.composeapp.generated.resources.reader_voice_theo
import indelible.composeapp.generated.resources.reader_voice_theo_tagline
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

/**
 * STUB — no real audio. Advances a synthetic playback clock so the Listen UI can
 * be built and demonstrated end to end (transport, scrubber, waveform, sentence
 * highlighting). Replace with a platform `expect/actual` audio player
 * (AVAudioPlayer on iOS, ExoPlayer on Android) wired to backend TTS endpoints
 * when those exist; the [ReaderPlaybackController] interface is the drop-in seam.
 */
class StubPlaybackController(
    private val scope: CoroutineScope,
    private val sentenceCount: Int = DEFAULT_SENTENCE_COUNT,
) : ReaderPlaybackController {
    private val _state = MutableStateFlow(PlaybackState())
    override val state: StateFlow<PlaybackState> = _state.asStateFlow()

    override val voices: List<ReaderVoice> = VOICES

    private var tickJob: Job? = null
    private var sleepJob: Job? = null

    override fun play() {
        if (_state.value.isPlaying) return
        // Restart from the top if we were parked at the end.
        if (_state.value.positionMs >= _state.value.durationMs) {
            _state.value = _state.value.copy(positionMs = 0, currentSentenceIndex = 0)
        }
        _state.value = _state.value.copy(isPlaying = true)
        startTicking()
    }

    override fun pause() {
        tickJob?.cancel()
        tickJob = null
        _state.value = _state.value.copy(isPlaying = false)
    }

    override fun togglePlayPause() {
        if (_state.value.isPlaying) pause() else play()
    }

    override fun seekTo(ms: Long) {
        val clamped = ms.coerceIn(0L, _state.value.durationMs)
        _state.value =
            _state.value.copy(
                positionMs = clamped,
                currentSentenceIndex = sentenceIndexFor(clamped),
            )
    }

    override fun skip(deltaMs: Long) = seekTo(_state.value.positionMs + deltaMs)

    override fun setSpeed(speed: Float) {
        _state.value = _state.value.copy(speed = speed.coerceIn(MIN_SPEED, MAX_SPEED))
    }

    override fun selectVoice(voiceId: String) {
        if (voices.none { it.id == voiceId }) return
        _state.value = _state.value.copy(voiceId = voiceId)
    }

    override fun setSleepTimer(minutes: Int?) {
        sleepJob?.cancel()
        sleepJob = null
        _state.value = _state.value.copy(sleepTimerMinutes = minutes)
        if (minutes != null) {
            sleepJob =
                scope.launch {
                    delay(minutes * MILLIS_PER_MINUTE)
                    pause()
                    _state.value = _state.value.copy(sleepTimerMinutes = null)
                }
        }
    }

    override fun release() {
        tickJob?.cancel()
        tickJob = null
        sleepJob?.cancel()
        sleepJob = null
        _state.value = _state.value.copy(isPlaying = false)
    }

    private fun startTicking() {
        tickJob?.cancel()
        tickJob =
            scope.launch {
                while (isActive && _state.value.isPlaying) {
                    delay(TICK_MS)
                    val current = _state.value
                    if (!current.isPlaying) break
                    val advance = (TICK_MS * current.speed).toLong()
                    val next = current.positionMs + advance
                    if (next >= current.durationMs) {
                        _state.value =
                            current.copy(
                                positionMs = current.durationMs,
                                currentSentenceIndex = (sentenceCount - 1).coerceAtLeast(0),
                                isPlaying = false,
                            )
                        break
                    }
                    _state.value =
                        current.copy(
                            positionMs = next,
                            currentSentenceIndex = sentenceIndexFor(next),
                        )
                }
            }
    }

    private fun sentenceIndexFor(positionMs: Long): Int {
        val duration = _state.value.durationMs
        if (duration <= 0L || sentenceCount <= 0) return 0
        val index = ((positionMs.toFloat() / duration) * sentenceCount).toInt()
        return index.coerceIn(0, sentenceCount - 1)
    }

    companion object {
        const val TICK_MS = 250L
        const val MILLIS_PER_MINUTE = 60_000L
        const val MIN_SPEED = 0.75f
        const val MAX_SPEED = 2.0f
        const val DEFAULT_SENTENCE_COUNT = 120

        val VOICES =
            listOf(
                ReaderVoice("ava", Res.string.reader_voice_ava, Res.string.reader_voice_ava_tagline),
                ReaderVoice("ren", Res.string.reader_voice_ren, Res.string.reader_voice_ren_tagline),
                ReaderVoice("mara", Res.string.reader_voice_mara, Res.string.reader_voice_mara_tagline),
                ReaderVoice("theo", Res.string.reader_voice_theo, Res.string.reader_voice_theo_tagline),
                ReaderVoice("iris", Res.string.reader_voice_iris, Res.string.reader_voice_iris_tagline),
            )
    }
}
