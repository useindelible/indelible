package app.indelible.reader.ui.components

import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.reader.viewmodel.EpubChapter
import app.indelible.reader.viewmodel.EpubReaderState
import app.indelible.reader.viewmodel.EpubStatus
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_epub_chapter_number
import indelible.composeapp.generated.resources.reader_epub_next
import indelible.composeapp.generated.resources.reader_epub_previous
import org.jetbrains.compose.resources.stringResource

/** Previous and next chapter, offered at the end of the chapter on screen; each takes at most half the width. */
@Composable
fun EpubChapterBar(
    state: EpubReaderState,
    onPrevious: () -> Unit,
    onNext: () -> Unit,
    modifier: Modifier = Modifier,
) {
    if (state.status != EpubStatus.READY || !state.atChapterEnd) return
    val previous = state.chapters.getOrNull(state.current - 1)
    val next = state.chapters.getOrNull(state.current + 1)
    Surface(
        modifier = modifier.fillMaxWidth().padding(horizontal = IndelibleSpacing.step12),
        shape = IndelibleShape.xxl,
        color = MaterialTheme.colorScheme.surfaceContainer,
        shadowElevation = IndelibleSpacing.step8,
    ) {
        BoxWithConstraints(Modifier.fillMaxWidth().padding(IndelibleSpacing.step6)) {
            val half = maxWidth / 2 - IndelibleSpacing.step6
            if (previous != null) {
                IndelibleButton(
                    text = stringResource(Res.string.reader_epub_previous, titleOf(previous, state.current - 1)),
                    onClick = onPrevious,
                    modifier = Modifier.align(Alignment.CenterStart).widthIn(max = half),
                    style = IndelibleButtonStyle.Text,
                    compact = true,
                    maxLines = 1,
                )
            }
            if (next != null) {
                IndelibleButton(
                    text = stringResource(Res.string.reader_epub_next, titleOf(next, state.current + 1)),
                    onClick = onNext,
                    modifier = Modifier.align(Alignment.CenterEnd).widthIn(max = half),
                    style = IndelibleButtonStyle.Secondary,
                    compact = true,
                    maxLines = 1,
                )
            }
        }
    }
}

@Composable
private fun titleOf(
    chapter: EpubChapter,
    index: Int,
): String = chapter.title ?: stringResource(Res.string.reader_epub_chapter_number, index + 1)

@Preview
@Composable
private fun EpubChapterBarPreview() {
    val chapters =
        listOf(
            EpubChapter(spine = 0, key = "e0", title = "Opening", href = "c0.xhtml"),
            EpubChapter(spine = 1, key = "spine:1", title = null, href = "c1.xhtml"),
            EpubChapter(spine = 2, key = "e2", title = "Closing", href = "c2.xhtml"),
        )
    AppTheme {
        Surface {
            EpubChapterBar(
                EpubReaderState(chapters = chapters, current = 1, atChapterEnd = true, status = EpubStatus.READY),
                onPrevious = {},
                onNext = {},
            )
        }
    }
}
