package app.indelible.reader.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.reader.model.ArticleTocEntry
import app.indelible.reader.model.ArticleTocStatus
import app.indelible.reader.model.DataPanel
import app.indelible.reader.model.HighlightColor
import app.indelible.reader.model.ReaderContentMode
import app.indelible.reader.model.ReaderPreferences
import app.indelible.reader.model.TagData
import app.indelible.reader.playback.PlaybackState
import app.indelible.reader.playback.ReaderPlaybackController
import app.indelible.reader.playback.ReaderVoice
import app.indelible.reader.playback.StubPlaybackController
import app.indelible.reader.repository.ReaderRepository
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class ReaderViewModel(
    private val documentId: String,
    private val repository: ReaderRepository,
    initialPreferences: ReaderPreferences = ReaderPreferences(),
) : ViewModel() {
    private val _uiState = MutableStateFlow<ReaderUiState>(ReaderUiState.Loading)
    val uiState: StateFlow<ReaderUiState> = _uiState.asStateFlow()

    private val _effects = MutableSharedFlow<ReaderEffect>()
    val effects: SharedFlow<ReaderEffect> = _effects.asSharedFlow()

    private val _activePanel = MutableStateFlow(DataPanel.NONE)
    val activePanel: StateFlow<DataPanel> = _activePanel.asStateFlow()

    private val _defaultHighlightColor = MutableStateFlow(HighlightColor.YELLOW)
    val defaultHighlightColor: StateFlow<HighlightColor> = _defaultHighlightColor.asStateFlow()

    private val playbackController: ReaderPlaybackController = StubPlaybackController(viewModelScope)
    val playbackState: StateFlow<PlaybackState> = playbackController.state
    val voices: List<ReaderVoice> get() = playbackController.voices

    private val progressSync =
        ReaderProgressSync(viewModelScope) { percent ->
            repository.updateProgress(documentId, percent)
        }
    private var initialScrollRequested = false
    private var progressTrackingEnabled = false

    init {
        _uiState.value = ReaderUiState.Loading
        load(initialPreferences)
    }

    private fun load(preferences: ReaderPreferences) {
        viewModelScope.launch {
            repository
                .getItem(documentId)
                .onSuccess { item ->
                    val contentMode =
                        when (item.itemType) {
                            "pdf" -> ReaderContentMode.PDF_COMING_SOON
                            "book" -> ReaderContentMode.EPUB_COMING_SOON
                            else -> ReaderContentMode.HTML
                        }
                    val currentProgress = item.progressPercent?.toFloat() ?: 0f
                    val savedProgress =
                        if (item.finishedAt != null ||
                            currentProgress >= PERCENT_MAX ||
                            (item.maxProgressPercent?.toFloat() ?: 0f) >= PERCENT_MAX
                        ) {
                            0f
                        } else {
                            currentProgress
                        }

                    _uiState.value =
                        ReaderUiState.Success(
                            item = item,
                            htmlContent = null,
                            contentMode = contentMode,
                            highlights = emptyList(),
                            progress = savedProgress,
                            preferences = preferences,
                        )

                    if (contentMode == ReaderContentMode.HTML) {
                        loadHtmlContent()
                        loadArticleToc()
                    }
                    loadHighlights()
                    loadItemNoteAndTags()
                    loadEntities()
                }.onFailure { error ->
                    _uiState.value =
                        ReaderUiState.Error(error.message ?: "Failed to load item")
                }
        }
    }

    /**
     * Resolves the readable HTML, polling the document while its render is still in flight (the
     * common case for a just-prepared feed document). Gives up after [MAX_CONTENT_POLLS] attempts
     * and surfaces a retryable [ReaderContentStatus.UNAVAILABLE] state rather than spinning forever.
     */
    private fun loadHtmlContent() {
        viewModelScope.launch {
            repeat(MAX_CONTENT_POLLS) { attempt ->
                if (attempt > 0) {
                    delay(CONTENT_POLL_INTERVAL_MS)
                    repository
                        .getItem(documentId)
                        .onSuccess { refreshed -> updateSuccessState { it.copy(item = refreshed) } }
                }
                val current = _uiState.value as? ReaderUiState.Success ?: return@launch
                if (current.item.readableReady) {
                    val fetched = repository.fetchReadableHtml(documentId)
                    if (fetched.isSuccess) {
                        val html = fetched.getOrThrow()
                        updateSuccessState {
                            it.copy(
                                htmlContent = maybeHideVideoEmbed(html),
                                contentStatus = ReaderContentStatus.READY,
                                retryStatus = ReaderRetryStatus.IDLE,
                                retryAfterSeconds = null,
                            )
                        }
                        return@launch
                    }
                }
            }
            updateSuccessState {
                it.copy(
                    contentStatus = ReaderContentStatus.UNAVAILABLE,
                    retryStatus = ReaderRetryStatus.IDLE,
                    retryAfterSeconds = null,
                )
            }
        }
    }

    /**
     * Resolves the Contents outline with its own poll: the backend answers `pending`
     * while a backfill derives the outline, and the readable-content poll above stops
     * as soon as HTML lands, so this budget is independent. `none` is terminal.
     */
    private fun loadArticleToc() {
        viewModelScope.launch {
            repeat(MAX_CONTENT_POLLS) { attempt ->
                if (attempt > 0) delay(CONTENT_POLL_INTERVAL_MS)
                val fetched = repository.getArticleToc(documentId)
                val toc = fetched.getOrNull()
                when (toc?.status) {
                    ArticleTocStatus.READY -> {
                        updateSuccessState {
                            it.copy(
                                toc =
                                    TocPanelState(
                                        status = TocStatus.READY,
                                        entries = toc.entries,
                                        truncated = toc.truncated,
                                        activeIndex =
                                            TocSections.activeSectionIndex(
                                                it.progress,
                                                toc.entries,
                                            ),
                                    ),
                            )
                        }
                        return@launch
                    }
                    ArticleTocStatus.NONE -> {
                        updateSuccessState { it.copy(toc = TocPanelState(status = TocStatus.NONE)) }
                        return@launch
                    }
                    ArticleTocStatus.PENDING -> {
                        updateSuccessState { it.copy(toc = it.toc.copy(status = TocStatus.PENDING)) }
                    }
                    null -> Unit
                }
            }
            updateSuccessState { it.copy(toc = it.toc.copy(status = TocStatus.UNAVAILABLE)) }
        }
    }

    fun onTocEntryTapped(entry: ArticleTocEntry) {
        closePanel()
        viewModelScope.launch {
            _effects.emit(ReaderEffect.ScrollToAnchor(entry.id, entry.sourceHeadingIndex))
        }
    }

    fun retryLoadContent() {
        val current = _uiState.value as? ReaderUiState.Success ?: return
        if (current.contentMode != ReaderContentMode.HTML) return
        if (current.retryStatus != ReaderRetryStatus.IDLE) return
        updateSuccessState { it.copy(retryStatus = ReaderRetryStatus.QUEUING) }
        viewModelScope.launch {
            repository
                .reprocessDocument(documentId)
                .onSuccess { result ->
                    updateSuccessState {
                        it.copy(
                            retryStatus =
                                when {
                                    result.queued -> ReaderRetryStatus.QUEUED
                                    result.retryAfterSeconds != null -> ReaderRetryStatus.COOLDOWN
                                    else -> ReaderRetryStatus.QUEUED
                                },
                            retryAfterSeconds = result.retryAfterSeconds,
                        )
                    }
                    loadHtmlContent()
                }.onFailure { error ->
                    updateSuccessState {
                        it.copy(
                            contentStatus = ReaderContentStatus.UNAVAILABLE,
                            retryStatus = ReaderRetryStatus.IDLE,
                            retryAfterSeconds = null,
                        )
                    }
                    _effects.emit(ReaderEffect.ShowSnackbar(error.message ?: "Failed to retry content"))
                }
        }
    }

    /**
     * Video bodies are transcripts. The provider's own embed is redundant beside the native
     * poster; the masthead aura is too, because the poster is real cover art sitting where the
     * aura would paint; and transcript paragraphs are short and frequent, so the article's
     * paragraph rhythm reads as disjointed. All scoped to video, and the spacing stays relative
     * to the reader's own preference rather than overriding it.
     */
    private fun maybeHideVideoEmbed(html: String): String {
        val state = _uiState.value as? ReaderUiState.Success ?: return html
        if (state.item.itemType != "video") return html
        val videoStyle =
            "<style>.yt-embed{display:none!important}" +
                ".r-aura{display:none!important}" +
                ".article-body p{margin-bottom:calc(var(--paragraph-spacing) * 0.65)}</style>"
        return if (html.contains("</head>", ignoreCase = true)) {
            html.replace("</head>", "$videoStyle</head>", ignoreCase = true)
        } else {
            "$videoStyle$html"
        }
    }

    private fun loadHighlights() {
        viewModelScope.launch {
            repository
                .listHighlights(documentId)
                .onSuccess { highlights ->
                    updateSuccessState { it.copy(highlights = highlights) }
                }
        }
    }

    /** Entities are supplementary: a failed fetch leaves the list empty and the section hidden. */
    private fun loadEntities() {
        viewModelScope.launch {
            repository
                .listDocumentEntities(documentId)
                .onSuccess { entities -> updateSuccessState { it.copy(entities = entities) } }
        }
    }

    private fun loadItemNoteAndTags() {
        viewModelScope.launch {
            repository
                .getItemNote(documentId)
                .onSuccess { note -> updateSuccessState { it.copy(itemNote = note) } }
        }
        val libraryEntryId = (_uiState.value as? ReaderUiState.Success)?.item?.libraryEntryId ?: return
        viewModelScope.launch {
            repository
                .getItemTags(libraryEntryId)
                .onSuccess { tags -> updateSuccessState { it.copy(itemTags = tags) } }
        }
    }

    /**
     * Saves an unsaved (feed-opened) document to the library, then refreshes the read-model so
     * the library entry id lands and library-gated affordances (triage, bookmark, item tags)
     * unlock. Highlights/notes/Mila already work pre-save since they are document-scoped.
     */
    fun saveToLibrary() {
        val current = _uiState.value as? ReaderUiState.Success ?: return
        val url = current.item.url
        if (current.item.saved || url == null) return
        viewModelScope.launch {
            repository
                .saveToLibrary(url, current.item.title, current.item.itemType)
                .onSuccess {
                    repository
                        .getItem(documentId)
                        .onSuccess { refreshed -> updateSuccessState { it.copy(item = refreshed) } }
                    loadItemNoteAndTags()
                    _effects.emit(ReaderEffect.ShowSnackbar("Saved to Library"))
                }.onFailure { error ->
                    _effects.emit(ReaderEffect.ShowSnackbar(error.message ?: "Couldn't save to Library"))
                }
        }
    }

    fun updatePreferences(preferences: ReaderPreferences) {
        updateSuccessState { it.copy(preferences = preferences) }
    }

    fun openPanel(panel: DataPanel) {
        _activePanel.value = panel
    }

    fun closePanel() {
        _activePanel.value = DataPanel.NONE
    }

    fun setDefaultHighlightColor(color: HighlightColor) {
        _defaultHighlightColor.value = color
    }

    fun togglePlayback() = playbackController.togglePlayPause()

    fun seekPlayback(ms: Long) = playbackController.seekTo(ms)

    fun skipPlayback(deltaMs: Long) = playbackController.skip(deltaMs)

    fun setPlaybackSpeed(speed: Float) = playbackController.setSpeed(speed)

    fun selectVoice(voiceId: String) = playbackController.selectVoice(voiceId)

    fun setSleepTimer(minutes: Int?) = playbackController.setSleepTimer(minutes)

    /** Ends the listen session and resets the player so the mini-bar dismisses. */
    fun stopPlayback() {
        playbackController.pause()
        playbackController.seekTo(0)
    }

    fun onScrollProgress(percent: Float) {
        if (!progressTrackingEnabled) return
        val normalized = percent.coerceIn(0f, PERCENT_MAX)
        updateSuccessState { state ->
            val toc =
                if (state.toc.status == TocStatus.READY) {
                    state.toc.copy(
                        activeIndex = TocSections.activeSectionIndex(normalized, state.toc.entries),
                    )
                } else {
                    state.toc
                }
            state.copy(progress = normalized, toc = toc)
        }
        progressSync.schedule(normalized)
    }

    fun onContentLoaded() {
        if (initialScrollRequested) return
        val state = (_uiState.value as? ReaderUiState.Success) ?: return
        initialScrollRequested = true
        viewModelScope.launch {
            _effects.emit(ReaderEffect.ScrollToPercent(state.progress))
        }
    }

    fun onScrollRestored() {
        progressTrackingEnabled = true
    }

    fun navigateBack() {
        progressSync.flush()
        viewModelScope.launch { _effects.emit(ReaderEffect.NavigateBack) }
    }

    fun createHighlight(
        color: HighlightColor,
        textContent: String,
        startOffset: Long,
        endOffset: Long,
    ) {
        viewModelScope.launch {
            repository
                .createHighlight(
                    itemId = documentId,
                    color = color.apiValue,
                    textContent = textContent,
                    startOffset = startOffset,
                    endOffset = endOffset,
                ).onSuccess { highlight ->
                    updateSuccessState { state ->
                        state.copy(highlights = state.highlights + highlight)
                    }
                }.onFailure {
                    _effects.emit(ReaderEffect.ShowSnackbar("Failed to create highlight"))
                }
        }
    }

    fun createHighlightForTag(
        color: HighlightColor,
        textContent: String,
        startOffset: Long,
        endOffset: Long,
        onCreated: (String) -> Unit,
    ) {
        viewModelScope.launch {
            repository
                .createHighlight(
                    itemId = documentId,
                    color = color.apiValue,
                    textContent = textContent,
                    startOffset = startOffset,
                    endOffset = endOffset,
                ).onSuccess { highlight ->
                    updateSuccessState { state ->
                        state.copy(highlights = state.highlights + highlight)
                    }
                    onCreated(highlight.id)
                }.onFailure {
                    _effects.emit(ReaderEffect.ShowSnackbar("Failed to create highlight"))
                }
        }
    }

    fun deleteHighlight(highlightId: String) {
        val state = (_uiState.value as? ReaderUiState.Success) ?: return
        val removedHighlight = state.highlights.find { it.id == highlightId }
        updateSuccessState { s ->
            s.copy(highlights = s.highlights.filter { it.id != highlightId })
        }
        viewModelScope.launch {
            repository
                .deleteHighlight(highlightId)
                .onFailure {
                    if (removedHighlight != null) {
                        updateSuccessState { s ->
                            s.copy(highlights = s.highlights + removedHighlight)
                        }
                    }
                    _effects.emit(ReaderEffect.ShowSnackbar("Failed to delete highlight"))
                }
        }
    }

    fun updateHighlightColor(
        highlightId: String,
        color: HighlightColor,
    ) {
        viewModelScope.launch {
            repository
                .updateHighlightColor(highlightId, color.apiValue)
                .onSuccess { updated ->
                    updateSuccessState { state ->
                        state.copy(
                            highlights =
                                state.highlights.map { h ->
                                    // The PATCH response omits tags/note; keep the ones already in state.
                                    if (h.id == highlightId) updated.copy(tags = h.tags, note = h.note) else h
                                },
                        )
                    }
                }.onFailure {
                    _effects.emit(ReaderEffect.ShowSnackbar("Failed to update color"))
                }
        }
    }

    fun upsertHighlightNote(
        highlightId: String,
        body: String,
    ) {
        viewModelScope.launch {
            repository
                .upsertHighlightNote(highlightId, body)
                .onSuccess { note ->
                    updateSuccessState { state ->
                        state.copy(
                            highlights =
                                state.highlights.map { h ->
                                    if (h.id == highlightId) h.copy(note = note) else h
                                },
                        )
                    }
                }.onFailure {
                    _effects.emit(ReaderEffect.ShowSnackbar("Failed to save note"))
                }
        }
    }

    fun deleteHighlightNote(highlightId: String) {
        viewModelScope.launch {
            repository
                .deleteHighlightNote(highlightId)
                .onSuccess {
                    updateSuccessState { state ->
                        state.copy(
                            highlights =
                                state.highlights.map { h ->
                                    if (h.id == highlightId) h.copy(note = null) else h
                                },
                        )
                    }
                }.onFailure {
                    _effects.emit(ReaderEffect.ShowSnackbar("Failed to delete note"))
                }
        }
    }

    fun setHighlightTags(
        highlightId: String,
        tags: List<String>,
    ) {
        updateSuccessState { state ->
            state.copy(
                highlights =
                    state.highlights.map { h ->
                        if (h.id == highlightId) h.copy(tags = tags) else h
                    },
            )
        }
        viewModelScope.launch {
            repository
                .setHighlightTags(highlightId, tags)
                .onFailure {
                    _effects.emit(ReaderEffect.ShowSnackbar("Failed to save tags"))
                }
        }
    }

    fun loadTagsForPicker(onResult: (List<TagData>) -> Unit) {
        viewModelScope.launch {
            repository
                .listTags()
                .onSuccess { tags -> onResult(tags) }
                .onFailure { onResult(emptyList()) }
        }
    }

    fun saveItemNote(body: String) {
        updateSuccessState { it.copy(itemNote = body) }
        viewModelScope.launch {
            repository
                .upsertItemNote(documentId, body)
                .onSuccess { saved -> updateSuccessState { it.copy(itemNote = saved) } }
                .onFailure { _effects.emit(ReaderEffect.ShowSnackbar("Failed to save note")) }
        }
    }

    fun moveToTriage(state: String) {
        val current = (_uiState.value as? ReaderUiState.Success)?.item ?: return
        val libraryEntryId = current.libraryEntryId ?: return
        closePanel()
        viewModelScope.launch {
            repository
                .triageItem(libraryEntryId, state)
                .onFailure {
                    _effects.emit(ReaderEffect.ShowSnackbar("Failed to move item"))
                }
        }
    }

    fun setItemTags(tags: List<String>) {
        val libraryEntryId = (_uiState.value as? ReaderUiState.Success)?.item?.libraryEntryId ?: return
        updateSuccessState { it.copy(itemTags = tags) }
        viewModelScope.launch {
            repository
                .setItemTags(libraryEntryId, tags)
                .onSuccess { saved -> updateSuccessState { it.copy(itemTags = saved) } }
                .onFailure { _effects.emit(ReaderEffect.ShowSnackbar("Failed to save tags")) }
        }
    }

    private fun updateSuccessState(transform: (ReaderUiState.Success) -> ReaderUiState.Success) {
        val current = _uiState.value as? ReaderUiState.Success ?: return
        _uiState.value = transform(current)
    }

    override fun onCleared() {
        playbackController.release()
        super.onCleared()
    }

    companion object {
        private const val MAX_CONTENT_POLLS = 15
        private const val CONTENT_POLL_INTERVAL_MS = 2000L
        private const val PERCENT_MAX = 100f
    }
}
