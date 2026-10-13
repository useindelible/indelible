package app.indelible.reader.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.core.i18n.UiMessage
import app.indelible.core.offline.ChapterPosition
import app.indelible.core.util.uuidV7
import app.indelible.reader.model.ArticleTocEntry
import app.indelible.reader.model.DataPanel
import app.indelible.reader.model.HighlightColor
import app.indelible.reader.model.ReaderContentMode
import app.indelible.reader.model.ReaderDocument
import app.indelible.reader.model.ReaderPreferences
import app.indelible.reader.model.TagData
import app.indelible.reader.playback.PlaybackState
import app.indelible.reader.playback.ReaderPlaybackController
import app.indelible.reader.playback.ReaderVoice
import app.indelible.reader.playback.StubPlaybackController
import app.indelible.reader.repository.ReaderRepository
import app.indelible.reader.repository.ReadingEventWriter
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_error_load
import indelible.composeapp.generated.resources.reader_error_move
import indelible.composeapp.generated.resources.reader_error_retry_content
import indelible.composeapp.generated.resources.reader_error_save_library
import indelible.composeapp.generated.resources.reader_error_save_note
import indelible.composeapp.generated.resources.reader_error_save_tags
import indelible.composeapp.generated.resources.reader_saved_library
import kotlinx.coroutines.CancellationException
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
    private val initialPreferences: ReaderPreferences = ReaderPreferences(),
    private val readingEvents: ReadingEventWriter,
) : ViewModel() {
    /** One sitting: every event this reader records is grouped under it. */
    private val sessionId = uuidV7()
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

    /** The open book's reading session; null for anything else. */
    var epubSession: EpubReaderSession? = null
        private set

    private var leaving = false

    private val bookChapterMissing: Boolean
        get() = epubSession?.let { it.currentChapterKey() == null } == true

    private val highlightActions =
        ReaderHighlightActions(documentId, repository, viewModelScope, _uiState, _effects) {
            epubSession?.currentChapterKey()
        }

    private val htmlLoader = ReaderHtmlLoader(documentId, repository, viewModelScope, _uiState)

    private val progressSync =
        ReaderProgressSync(viewModelScope) { percent, position ->
            readingEvents.recordProgress(documentId, percent, sessionId, position)
        }
    private var initialScrollRequested = false
    private var progressTrackingEnabled = false

    init {
        _uiState.value = ReaderUiState.Loading
        load(initialPreferences)
    }

    fun retryOpen() {
        if (_uiState.value !is ReaderUiState.Error) return
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
                            "book" -> ReaderContentMode.EPUB
                            else -> ReaderContentMode.HTML
                        }
                    val finished = item.isFinished()
                    val savedProgress = if (finished) 0f else item.progressPercent?.toFloat() ?: 0f

                    _uiState.value =
                        ReaderUiState.Success(
                            item = item,
                            htmlContent = null,
                            contentMode = contentMode,
                            highlights = emptyList(),
                            progress = savedProgress,
                            preferences = preferences,
                        )

                    recordOpened()

                    when (contentMode) {
                        ReaderContentMode.HTML -> {
                            htmlLoader.loadHtmlContent()
                            htmlLoader.loadArticleToc()
                        }
                        ReaderContentMode.EPUB -> openBook(item, finished)
                        ReaderContentMode.PDF_COMING_SOON -> Unit
                    }
                    loadHighlights()
                    loadItemNoteAndTags()
                    loadEntities()
                }.onFailure {
                    _uiState.value = ReaderUiState.Error(UiMessage(Res.string.reader_error_load))
                }
        }
    }

    private fun openBook(
        item: ReaderDocument,
        finished: Boolean,
    ) {
        val locator = item.chapterLocator?.takeUnless { finished }
        val resumeAt = locator?.let { ChapterPosition(it, item.chapterOffset ?: 0) }
        val session = EpubReaderSession(documentId, repository, viewModelScope, progressSync, resumeAt)
        epubSession = session
        viewModelScope.launch {
            session.state.collect { epub ->
                updateSuccessState {
                    it.copy(epub = epub, progress = epub.percent ?: it.progress, toc = epub.contentsPanel())
                }
            }
        }
        session.open()
    }

    fun onTocEntryTapped(entry: ArticleTocEntry) {
        closePanel()
        epubSession?.let { book ->
            book.onTocEntry(entry.id)
            return
        }
        viewModelScope.launch {
            _effects.emit(ReaderEffect.ScrollToAnchor(entry.id, entry.sourceHeadingIndex))
        }
    }

    fun reprocessBook() {
        val book = epubSession ?: return
        viewModelScope.launch {
            repository
                .reprocessDocument(documentId)
                .onSuccess { book.onReprocessQueued() }
                .onFailure {
                    _effects.emit(ReaderEffect.ShowSnackbar(UiMessage(Res.string.reader_error_retry_content)))
                }
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
                    htmlLoader.loadHtmlContent()
                }.onFailure {
                    updateSuccessState {
                        it.copy(
                            contentStatus = ReaderContentStatus.UNAVAILABLE,
                            retryStatus = ReaderRetryStatus.IDLE,
                            retryAfterSeconds = null,
                        )
                    }
                    _effects.emit(
                        ReaderEffect.ShowSnackbar(UiMessage(Res.string.reader_error_retry_content)),
                    )
                }
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
                .onSuccess { note -> updateSuccessState { it.copy(itemNote = note, itemNoteLoaded = true) } }
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
                    _effects.emit(ReaderEffect.ShowSnackbar(UiMessage(Res.string.reader_saved_library)))
                }.onFailure {
                    _effects.emit(
                        ReaderEffect.ShowSnackbar(UiMessage(Res.string.reader_error_save_library)),
                    )
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
        val state =
            (_uiState.value as? ReaderUiState.Success)?.takeIf { it.contentMode == ReaderContentMode.HTML } ?: return
        initialScrollRequested = true
        viewModelScope.launch {
            _effects.emit(ReaderEffect.ScrollToPercent(state.progress))
        }
    }

    fun onScrollRestored() {
        progressTrackingEnabled = true
    }

    /** The app leaves the screen: a book writes where its reader is. */
    fun onStop() {
        val book = epubSession ?: return
        viewModelScope.launch { book.captureAndFlush() }
    }

    /** Leaves once the newest position is written; a book first asks its chapter where the reader is. */
    fun navigateBack() {
        if (leaving) return
        leaving = true
        val book = epubSession
        if (book == null) progressSync.flush()
        viewModelScope.launch {
            book?.captureAndFlush()
            _effects.emit(ReaderEffect.NavigateBack)
            leaving = false
        }
    }

    fun createHighlight(
        color: HighlightColor,
        textContent: String,
        startOffset: Long,
        endOffset: Long,
    ) {
        if (!bookChapterMissing) highlightActions.createHighlight(color, textContent, startOffset, endOffset)
    }

    fun createHighlightForTag(
        color: HighlightColor,
        textContent: String,
        startOffset: Long,
        endOffset: Long,
        onCreated: (String) -> Unit,
    ) {
        if (!bookChapterMissing) highlightActions.createHighlight(color, textContent, startOffset, endOffset, onCreated)
    }

    fun deleteHighlight(highlightId: String) = highlightActions.deleteHighlight(highlightId)

    fun updateHighlightColor(
        highlightId: String,
        color: HighlightColor,
    ) = highlightActions.updateHighlightColor(highlightId, color)

    fun upsertHighlightNote(
        highlightId: String,
        body: String,
    ) = highlightActions.upsertHighlightNote(highlightId, body)

    fun deleteHighlightNote(highlightId: String) = highlightActions.deleteHighlightNote(highlightId)

    fun setHighlightTags(
        highlightId: String,
        tags: List<String>,
    ) = highlightActions.setHighlightTags(highlightId, tags)

    fun loadTagsForPicker(onResult: (List<TagData>) -> Unit) = highlightActions.loadTagsForPicker(onResult)

    fun saveItemNote(body: String) {
        if ((_uiState.value as? ReaderUiState.Success)?.itemNoteLoaded != true) return
        updateSuccessState { it.copy(itemNote = body) }
        viewModelScope.launch {
            repository
                .upsertItemNote(documentId, body)
                .onSuccess { saved -> updateSuccessState { it.copy(itemNote = saved) } }
                .onFailure {
                    _effects.emit(ReaderEffect.ShowSnackbar(UiMessage(Res.string.reader_error_save_note)))
                }
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
                    _effects.emit(ReaderEffect.ShowSnackbar(UiMessage(Res.string.reader_error_move)))
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
                .onFailure {
                    _effects.emit(ReaderEffect.ShowSnackbar(UiMessage(Res.string.reader_error_save_tags)))
                }
        }
    }

    /** The open is worth recording, never worth losing the document over. */
    private suspend fun recordOpened() {
        val recorded = runCatching { readingEvents.recordOpened(documentId, sessionId) }
        recorded.exceptionOrNull()?.let { if (it is CancellationException) throw it }
    }

    private fun updateSuccessState(transform: (ReaderUiState.Success) -> ReaderUiState.Success) {
        val current = _uiState.value as? ReaderUiState.Success ?: return
        _uiState.value = transform(current)
    }

    override fun onCleared() {
        playbackController.release()
        super.onCleared()
    }
}

private const val PERCENT_MAX = 100f

/** A finished item opens at its start, so a reread does not begin on the last page. */
private fun ReaderDocument.isFinished(): Boolean =
    finishedAt != null ||
        (progressPercent?.toFloat() ?: 0f) >= PERCENT_MAX ||
        (maxProgressPercent?.toFloat() ?: 0f) >= PERCENT_MAX
