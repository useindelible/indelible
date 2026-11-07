package app.indelible.reader.viewmodel

import app.indelible.core.offline.ChapterPosition
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull

@OptIn(ExperimentalCoroutinesApi::class)
class EpubReaderNavigationTest {
    @Test
    fun nextAndPreviousMoveOneChapter() =
        runTest {
            val h = epubHarness()
            openedAndPositioned(h)

            h.session.next()
            advanceUntilIdle()
            assertEquals(1, h.state.current)
            assertEquals("<p>chapter 2</p>", h.state.html)

            h.session.previous()
            advanceUntilIdle()
            h.session.previous()
            advanceUntilIdle()
            assertEquals(0, h.state.current)
            assertEquals(listOf(0, 2, 0), h.repository.epubChapterCalls)
        }

    @Test
    fun sameChapterSubEntryScrollsWithoutReload() =
        runTest {
            val h = epubHarness(resumeAt = ChapterPosition("e5", 0))
            openedAndPositioned(h)

            h.session.onTocEntry("e5b")
            advanceUntilIdle()

            assertEquals("later", h.state.anchor?.id)
            assertEquals(listOf(5), h.repository.epubChapterCalls)
        }

    @Test
    fun crossChapterSubEntryOpensAtItsFragment() =
        runTest {
            val h = epubHarness()
            openedAndPositioned(h)

            h.session.onTocEntry("e5b")
            advanceUntilIdle()

            assertEquals(2, h.state.current)
            assertEquals("later", h.state.anchor?.id)
            assertNull(h.state.restore)
        }

    @Test
    fun fragmentLinkScrollsInTheChapter() =
        runTest {
            val h = epubHarness()
            openedAndPositioned(h)

            h.session.onBookLink("#fn1")
            advanceUntilIdle()

            assertEquals(0, h.state.current)
            assertEquals("fn1", h.state.anchor?.id)
            assertEquals(listOf(0), h.repository.epubChapterCalls)
        }

    @Test
    fun linkToAnUnlistedChapterOpensItAtItsFragment() =
        runTest {
            val h = epubHarness()
            openedAndPositioned(h)

            h.session.onBookLink("../text/c2.xhtml#n4")
            advanceUntilIdle()

            assertEquals(1, h.state.current)
            assertEquals("<p>chapter 2</p>", h.state.html)
            assertEquals("n4", h.state.anchor?.id)
        }

    @Test
    fun unknownFileLinkDoesNothing() =
        runTest {
            val h = epubHarness()
            openedAndPositioned(h)
            val before = h.state

            h.session.onBookLink("elsewhere.xhtml#x")
            advanceUntilIdle()

            assertEquals(before, h.state)
        }

    @Test
    fun backCapturesAndFlushesBeforeNavigating() =
        runTest {
            val h = epubHarness()
            openedAndPositioned(h)
            val order = mutableListOf<String>()

            launch {
                h.session.captureAndFlush()
                order += "flushed:${h.written.lastOrNull()?.second}"
            }
            runCurrent()
            assertNotNull(h.state.capture)
            assertEquals(emptyList(), order)

            h.onSample(sample(percent = 40f, offset = 12))
            advanceUntilIdle()

            assertEquals(listOf("flushed:${ChapterPosition("e0", 12)}"), order)
        }

    @Test
    fun captureTimeoutFlushesTheLastCompleteSample() =
        runTest {
            val h = epubHarness()
            openedAndPositioned(h)
            h.onSample(sample(percent = 10f, offset = 5))
            runCurrent()
            h.onSample(sample(percent = 20f, offset = 9))
            var done = false

            launch {
                h.session.captureAndFlush()
                done = true
            }
            advanceTimeBy(CAPTURE_WAIT_MS + 1)
            runCurrent()

            assertEquals(true, done)
            assertEquals(ChapterPosition("e0", 9), h.written.last().second)
        }
}
