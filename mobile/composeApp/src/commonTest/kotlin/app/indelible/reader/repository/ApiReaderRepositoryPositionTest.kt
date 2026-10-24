package app.indelible.reader.repository

import app.indelible.core.offline.ChapterPosition
import app.indelible.core.offline.DL_DOC
import app.indelible.core.offline.json
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals

private val MINE = ChapterPosition("ch-mine", 120)

class ApiReaderRepositoryPositionTest {
    @Test
    fun copy_keeps_the_server_chapter_position() =
        runTest {
            val h = bookAtServerPosition()
            h.keepCopy()
            h.goOffline()

            val document = h.document()

            assertEquals("ch-server" to 40, document.chapterLocator to document.chapterOffset)
        }

    @Test
    fun queued_position_wins_over_the_copy() =
        runTest {
            val h = bookAtServerPosition()
            h.keepCopy()
            h.repository.recordProgress(DL_DOC, 50f, "s", MINE)
            h.goOffline()

            val document = h.document()

            assertEquals(MINE.chapter to MINE.offset, document.chapterLocator to document.chapterOffset)
        }

    @Test
    fun queued_position_wins_online_without_a_copy() =
        runTest {
            val h = bookAtServerPosition()
            h.repository.recordProgress(DL_DOC, 50f, "s", MINE)

            val document = h.document()

            assertEquals(MINE.chapter to MINE.offset, document.chapterLocator to document.chapterOffset)
        }

    private suspend fun TestScope.bookAtServerPosition() =
        readerOfflineHarness {
            val position = ""","chapter_locator":"ch-server","chapter_offset":40}"""
            route(READER_PATH) { json(readerJson().dropLast(1) + position) }
        }
}
