package app.indelible.reader.viewmodel

import app.indelible.api.generated.models.EpubStoredChapter
import app.indelible.core.offline.CapabilitiesUnknownException
import app.indelible.core.offline.ChapterPosition
import app.indelible.core.offline.ServerTooOldException
import app.indelible.reader.model.EpubChapterList
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import okio.IOException
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class EpubReaderSessionTest {
    @Test
    fun opensAtTheStoredChapterAndOffset() =
        runTest {
            val h = epubHarness(resumeAt = ChapterPosition("e5", 120))

            h.session.open()
            advanceUntilIdle()

            assertEquals(EpubStatus.READY, h.state.status)
            assertEquals(2, h.state.current)
            assertEquals("<p>chapter 5</p>", h.state.html)
            assertEquals(120, h.state.restore?.offset)
        }

    @Test
    fun positionKeyedByASubEntryOpensItsChapter() =
        runTest {
            val h = epubHarness(resumeAt = ChapterPosition("e5b", 40))

            h.session.open()
            advanceUntilIdle()

            assertEquals(2, h.state.current)
            assertEquals(40, h.state.restore?.offset)
        }

    @Test
    fun unknownChapterKeyOpensAtTheStart() =
        runTest {
            val h = epubHarness(resumeAt = ChapterPosition("gone", 50))

            h.session.open()
            advanceUntilIdle()

            assertEquals(0, h.state.current)
            assertEquals(0, h.state.restore?.offset)
        }

    @Test
    fun offsetPastChapterEndStillOpensTheChapter() =
        runTest {
            val h = epubHarness(resumeAt = ChapterPosition("spine:2", 999_999))

            h.session.open()
            advanceUntilIdle()

            assertEquals(EpubStatus.READY, h.state.status)
            assertEquals(1, h.state.current)
            assertEquals(999_999, h.state.restore?.offset)
        }

    @Test
    fun completeSampleSchedulesProgressAndPosition() =
        runTest {
            val h = epubHarness()
            openedAndPositioned(h)

            h.onSample(sample(percent = 50f, offset = 30))
            advanceUntilIdle()

            val expected: Pair<Float, ChapterPosition?> = bookPercent(0, 50f, 3) to ChapterPosition("e0", 30)
            assertEquals(listOf(expected), h.written)
        }

    @Test
    fun percentOnlyFrameWritesNothing() =
        runTest {
            val h = epubHarness()
            openedAndPositioned(h)

            h.onSample(sample(percent = 50f))
            advanceUntilIdle()

            assertTrue(h.written.isEmpty())
            assertEquals(bookPercent(0, 50f, 3), h.state.percent)
        }

    @Test
    fun trackingStartsAfterPositioned() =
        runTest {
            val h = epubHarness()
            h.session.open()
            advanceUntilIdle()

            h.onSample(sample(percent = 50f, offset = 30))
            advanceUntilIdle()

            assertTrue(h.written.isEmpty())
            assertNull(h.state.percent)
        }

    @Test
    fun positionedChapterAsksForASample() =
        runTest {
            val h = epubHarness()
            h.session.open()
            advanceUntilIdle()
            val before = h.state.capture

            h.onPositioned()

            assertNotEquals(before, h.state.capture)
        }

    @Test
    fun signalsFromAnotherChapterAreIgnored() =
        runTest {
            val h = epubHarness()
            openedAndPositioned(h)
            h.session.next()
            advanceUntilIdle()

            h.session.onPositioned(0)
            h.session.onSample(2, sample(percent = 10f, offset = 4))
            h.onPositioned()
            h.session.onSample(0, sample(percent = 10f, offset = 4))
            advanceUntilIdle()

            assertTrue(h.written.isEmpty())
        }

    @Test
    fun lastChapterThatFitsFinishesTheBook() =
        runTest {
            val h = epubHarness(resumeAt = ChapterPosition("e5", 0))
            openedAndPositioned(h)

            h.onSample(sample(percent = 0f, offset = 0, atEnd = true))
            advanceUntilIdle()

            assertEquals(100f, h.written.last().first)
        }

    @Test
    fun chapterThatFitsIsAtItsEnd() =
        runTest {
            val h = epubHarness()
            openedAndPositioned(h)

            h.onSample(sample(percent = 0f, offset = 0, atEnd = true))

            assertTrue(h.state.atChapterEnd)
        }

    @Test
    fun laterChapterRequestWinsOverASlowerEarlierOne() =
        runTest {
            val h = epubHarness()
            openedAndPositioned(h)
            h.repository.epubChapterResult = { spine ->
                delay(if (spine == 2) 500 else 10)
                Result.success("<p>chapter $spine</p>")
            }

            h.session.goTo(1)
            h.session.goTo(2)
            advanceUntilIdle()

            assertEquals(2, h.state.current)
            assertEquals("<p>chapter 5</p>", h.state.html)
        }

    @Test
    fun missingChapterShowsUnavailableAndRetrySucceeds() =
        runTest {
            val h = epubHarness()
            h.repository.epubChapterResult = { Result.failure(IOException("gone")) }
            h.session.open()
            advanceUntilIdle()
            assertEquals(EpubStatus.CHAPTER_UNAVAILABLE, h.state.status)

            h.repository.epubChapterResult = { Result.success("<p>back $it</p>") }
            h.session.retry()
            advanceUntilIdle()

            assertEquals(EpubStatus.READY, h.state.status)
            assertEquals("<p>back 0</p>", h.state.html)
        }

    @Test
    fun chapterListStatesMapToReaderStates() =
        runTest {
            val expected =
                mapOf(
                    EpubChapterList.PENDING to EpubStatus.LIST_PENDING,
                    EpubChapterList.NEEDS_REPROCESS to EpubStatus.NEEDS_REPROCESS,
                    EpubChapterList.NEEDS_REUPLOAD to EpubStatus.NEEDS_REUPLOAD,
                )
            for ((list, status) in expected) {
                val h = epubHarness(toc = Result.success(bookToc(chapterList = list)))
                h.session.open()
                advanceUntilIdle()

                h.session.goTo(1)
                advanceUntilIdle()

                assertEquals(status, h.state.status, "$list")
                assertEquals(emptyList(), h.repository.epubChapterCalls)
            }
        }

    @Test
    fun oldServerIsTooOld() =
        runTest {
            val h = epubHarness(toc = Result.failure(ServerTooOldException()))

            h.session.open()
            advanceUntilIdle()

            assertEquals(EpubStatus.SERVER_TOO_OLD, h.state.status)
        }

    @Test
    fun networkFailureIsAConnectionErrorWithRetry() =
        runTest {
            for (failure in listOf(CapabilitiesUnknownException(), IOException("offline"))) {
                val h = epubHarness(toc = Result.failure(failure))
                h.session.open()
                advanceUntilIdle()
                assertEquals(EpubStatus.CONNECTION_FAILED, h.state.status)

                h.repository.epubTocResult = Result.success(bookToc())
                h.session.retry()
                advanceUntilIdle()

                assertEquals(EpubStatus.READY, h.state.status)
            }
        }

    @Test
    fun emptyBookShowsEmptyWithoutCrashing() =
        runTest {
            for (chapters in listOf(emptyList<EpubStoredChapter>(), null)) {
                val h = epubHarness(toc = Result.success(bookToc(chapters = chapters)))
                h.session.open()
                advanceUntilIdle()

                h.onPositioned()
                h.onSample(sample(percent = 50f, offset = 3))
                h.session.next()
                advanceUntilIdle()

                assertEquals(EpubStatus.EMPTY, h.state.status)
                assertTrue(h.written.isEmpty())
            }
        }
}
