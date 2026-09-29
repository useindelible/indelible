package app.indelible.reader.ui.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.width
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertWidthIsAtLeast
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.reader.viewmodel.EpubChapter
import app.indelible.reader.viewmodel.EpubReaderState
import app.indelible.reader.viewmodel.EpubStatus
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalTestApi::class)
class EpubChapterBarTest {
    private val chapters =
        listOf(
            EpubChapter(spine = 0, key = "e0", title = "Opening", href = "c0.xhtml"),
            EpubChapter(spine = 2, key = "spine:2", title = null, href = "c2.xhtml"),
            EpubChapter(spine = 5, key = "e5", title = "Closing", href = "c5.xhtml"),
        )

    private fun atEndOf(current: Int) =
        EpubReaderState(chapters = chapters, current = current, atChapterEnd = true, status = EpubStatus.READY)

    @Test
    fun chapterBarOffersNextWithItsTitle() =
        runComposeUiTest {
            var moved = ""
            setContent {
                AppTheme {
                    EpubChapterBar(atEndOf(1), onPrevious = { moved = "previous" }, onNext = { moved = "next" })
                }
            }

            onNodeWithText("Next: Closing").performClick()

            assertEquals("next", moved)
        }

    @Test
    fun unlistedNextChapterShowsItsOrdinalTitle() =
        runComposeUiTest {
            setContent { AppTheme { EpubChapterBar(atEndOf(0), onPrevious = {}, onNext = {}) } }

            onNodeWithText("Next: Chapter 2").assertIsDisplayed()
        }

    @Test
    fun firstChapterHasNoPrevious() =
        runComposeUiTest {
            setContent { AppTheme { EpubChapterBar(atEndOf(0), onPrevious = {}, onNext = {}) } }

            onNodeWithText("Previous", substring = true).assertDoesNotExist()
        }

    @Test
    fun lastChapterHasNoNext() =
        runComposeUiTest {
            setContent { AppTheme { EpubChapterBar(atEndOf(2), onPrevious = {}, onNext = {}) } }

            onNodeWithText("Previous: Chapter 2").assertIsDisplayed()
            onNodeWithText("Next", substring = true).assertDoesNotExist()
        }

    @Test
    fun longTitlesLeaveRoomForNext() =
        runComposeUiTest {
            val long = chapters.toMutableList()
            long[0] = long[0].copy(title = "Opening, in which a great many things happen before anything else can")
            val state = atEndOf(1).copy(chapters = long)
            setContent {
                AppTheme {
                    Box(Modifier.width(IndelibleSpacing.step96 * 3)) {
                        EpubChapterBar(state, onPrevious = {}, onNext = {})
                    }
                }
            }

            onNodeWithText("Next: Closing").assertWidthIsAtLeast(IndelibleSpacing.step64)
        }

    @Test
    fun chapterBarIsHiddenBeforeTheEnd() =
        runComposeUiTest {
            val midChapter = atEndOf(1).copy(atChapterEnd = false)
            setContent { AppTheme { EpubChapterBar(midChapter, onPrevious = {}, onNext = {}) } }

            onNodeWithText("Next", substring = true).assertDoesNotExist()
        }
}
