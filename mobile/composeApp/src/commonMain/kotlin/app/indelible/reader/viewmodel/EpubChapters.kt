package app.indelible.reader.viewmodel

import app.indelible.core.offline.ChapterPosition
import app.indelible.reader.model.ArticleTocEntry
import app.indelible.reader.model.EpubChapterList
import app.indelible.reader.model.EpubToc
import app.indelible.reader.model.EpubTocEntry
import app.indelible.reader.model.HighlightData

/** [key] is the id of the chapter's first contents entry, as web keys it, else `spine:<n>`. */
data class EpubChapter(
    val spine: Int,
    val key: String,
    val title: String?,
    val href: String,
)

fun chapterSequence(toc: EpubToc): List<EpubChapter> {
    val firstEntries = toc.toc.groupBy { it.spineIndex }.mapValues { it.value.first() }
    return toc.metadata.chapters.orEmpty().map { stored ->
        val entry = firstEntries[stored.spineIndex]
        EpubChapter(
            spine = stored.spineIndex,
            key = entry?.id ?: spineKey(stored.spineIndex),
            title = entry?.title?.takeIf { it.isNotBlank() } ?: stored.title.takeIf { it.isNotBlank() },
            href = stored.href,
        )
    }
}

fun spineKey(spine: Int): String = "spine:$spine"

/** The whole book's percent (0..100) at [chapterPercent] into the chapter at [ordinal] of [count] (> 0). */
fun bookPercent(
    ordinal: Int,
    chapterPercent: Float,
    count: Int,
): Float = ((ordinal + chapterPercent / PERCENT_MAX) / count * PERCENT_MAX).coerceIn(0f, PERCENT_MAX)

internal data class ChapterTarget(
    val index: Int,
    val offset: Int,
    val fragment: String?,
)

internal fun openingStatus(
    toc: EpubToc,
    chapters: List<EpubChapter>,
): EpubStatus? =
    when (toc.chapterList) {
        EpubChapterList.READY -> EpubStatus.EMPTY.takeIf { chapters.isEmpty() }
        EpubChapterList.PENDING -> EpubStatus.LIST_PENDING
        EpubChapterList.NEEDS_REPROCESS -> EpubStatus.NEEDS_REPROCESS
        EpubChapterList.NEEDS_REUPLOAD -> EpubStatus.NEEDS_REUPLOAD
    }

// A saved key is a chapter's key, another contents entry's id (web saves the entry in view), or `spine:<n>`.
internal fun resumeTarget(
    chapters: List<EpubChapter>,
    contents: List<EpubTocEntry>,
    saved: ChapterPosition?,
): ChapterTarget {
    val spine = contents.firstOrNull { it.id == saved?.chapter }?.spineIndex
    val index =
        chapters.indexOfFirst { it.key == saved?.chapter }.takeIf { it >= 0 }
            ?: chapters.indexOfFirst { spineKey(it.spine) == saved?.chapter || it.spine == spine }
    return if (saved != null && index >= 0) ChapterTarget(index, saved.offset, null) else ChapterTarget(0, 0, null)
}

/** The book's contents in the shape the Contents panel shows, with the chapter on screen active. */
internal fun EpubReaderState.contentsPanel(): TocPanelState {
    val spine = chapters.getOrNull(current)?.spine
    val status =
        when {
            chapters.isNotEmpty() && contents.isNotEmpty() -> TocStatus.READY
            status == EpubStatus.LOADING -> TocStatus.LOADING
            else -> TocStatus.NONE
        }
    return TocPanelState(
        status = status,
        entries =
            contents.map {
                ArticleTocEntry(it.depth, it.id, sourceHeadingIndex = -1, title = it.title, wordCount = it.wordCount)
            },
        activeIndex = contents.indexOfFirst { it.spineIndex == spine },
    )
}

/** The key of the chapter on screen; null while none is. */
fun EpubReaderState.chapterKey(): String? {
    if (status != EpubStatus.READY) return null
    return chapters.getOrNull(current)?.key
}

fun EpubReaderState.highlightsFor(all: List<HighlightData>): List<HighlightData> {
    val key = chapterKey() ?: return emptyList()
    return all.filter { it.locator?.chapter == key }
}

private const val PERCENT_MAX = 100f
