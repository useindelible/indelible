package app.indelible.reader.viewmodel

import app.indelible.reader.model.ArticleTocStatus
import app.indelible.reader.repository.ReaderRepository
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch

/** Readable HTML and the Contents outline for [ReaderViewModel], polled while the server still prepares them. */
internal class ReaderHtmlLoader(
    private val documentId: String,
    private val repository: ReaderRepository,
    private val scope: CoroutineScope,
    private val uiState: MutableStateFlow<ReaderUiState>,
) {
    /** Polls while the render is in flight; after [MAX_CONTENT_POLLS] tries, a retryable UNAVAILABLE. */
    fun loadHtmlContent() {
        scope.launch {
            repeat(MAX_CONTENT_POLLS) { attempt ->
                if (attempt > 0) {
                    delay(CONTENT_POLL_INTERVAL_MS)
                    repository
                        .getItem(documentId)
                        .onSuccess { refreshed -> updateSuccessState { it.copy(item = refreshed) } }
                }
                val current = uiState.value as? ReaderUiState.Success ?: return@launch
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

    /** Its own poll: the outline can stay `pending` after the HTML lands; `none` is terminal. */
    fun loadArticleToc() {
        scope.launch {
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

    // A video body is a transcript beside a native poster: the embed and aura are redundant, and
    // its short paragraphs get tighter spacing, still relative to the reader's own preference.
    private fun maybeHideVideoEmbed(html: String): String {
        val state = uiState.value as? ReaderUiState.Success
        if (state?.item?.itemType != "video") return html
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

    private fun updateSuccessState(transform: (ReaderUiState.Success) -> ReaderUiState.Success) {
        val current = uiState.value as? ReaderUiState.Success ?: return
        uiState.value = transform(current)
    }

    private companion object {
        const val MAX_CONTENT_POLLS = 15
        const val CONTENT_POLL_INTERVAL_MS = 2000L
    }
}
