package app.indelible.reader.repository

import app.indelible.core.network.ApiException
import app.indelible.core.offline.ChapterPosition
import app.indelible.core.offline.DL_DOC
import app.indelible.core.offline.PagePosition
import app.indelible.core.offline.SnapshotRow
import app.indelible.core.offline.json
import app.indelible.home.repository.homeRow
import app.indelible.library.repository.libraryEntryJson
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.test.runTest
import kotlinx.datetime.Instant
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertNull
import kotlin.test.assertTrue

class ApiReaderRepositoryLastSeenTest {
    private fun libraryRow(
        type: String = "pdf",
        progress: Int = 40,
    ): SnapshotRow {
        val json = libraryEntryJson("entry_1", DL_DOC, type, progress = progress)
        return SnapshotRow("entry_1", DL_DOC, type, "Title entry_1", json)
    }

    private suspend fun ReaderOfflineHarness.seen(
        key: String,
        row: SnapshotRow,
        at: Long = 1,
    ) = store.replaceList(download.session, key, listOf(row), at)

    @Test
    fun pdfWithNoCopyOpensFromItsLastSeenRowWhenTheServerGivesNoAnswer() =
        runTest {
            val harness = readerOfflineHarness()
            harness.seen("triage:inbox:all", libraryRow())
            harness.goOffline()

            val document = harness.document()

            assertEquals("pdf", document.itemType)
            assertEquals("Title entry_1", document.title)
            assertTrue(document.saved)
            assertFalse(document.readableReady)
            assertTrue(document.availableAssets.isEmpty())
            assertEquals(40, document.progressPercent)
            assertEquals("entry_1", document.libraryEntryId)
            assertTrue(document.lastSeen)
        }

    @Test
    fun aLibraryRowGivesItsBucketAndSavedDate() =
        runTest {
            val harness = readerOfflineHarness()
            val json = libraryEntryJson("entry_1", DL_DOC, "pdf", triage = "later")
            harness.seen("triage:later:all", SnapshotRow("entry_1", DL_DOC, "pdf", "Title entry_1", json))
            harness.goOffline()

            val document = harness.document()

            assertEquals("later", document.triageState)
            assertEquals(Instant.parse("2026-01-01T00:00:00Z"), document.savedAt)
        }

    @Test
    fun aHomeRowLeavesTheBucketAndSavedDateUnknown() =
        runTest {
            val harness = readerOfflineHarness()
            harness.seen("home:recent", homeRow(DL_DOC, "pdf"))
            harness.goOffline()

            val document = harness.document()

            assertNull(document.triageState)
            assertNull(document.savedAt)
        }

    @Test
    fun aLibraryRowWinsOverANewerHomeRow() =
        runTest {
            val harness = readerOfflineHarness()
            harness.seen("triage:inbox:all", libraryRow(), at = 1)
            harness.seen("home:recent", homeRow(DL_DOC, "pdf"), at = 9)
            harness.goOffline()

            assertEquals("entry_1", harness.document().libraryEntryId)
        }

    @Test
    fun queuedProgressWinsOverTheRowsProgress() =
        runTest {
            val harness = readerOfflineHarness()
            harness.seen("triage:inbox:all", libraryRow())
            harness.repository.recordProgress(DL_DOC, 60f, "s", PagePosition(3, 2500))
            harness.goOffline()

            val document = harness.document()

            assertEquals(60, document.progressPercent)
            assertEquals("page:3", document.chapterLocator)
            assertEquals(2500, document.chapterOffset)
        }

    @Test
    fun queuedChapterWinsForABook() =
        runTest {
            val harness = readerOfflineHarness()
            harness.seen("triage:inbox:all", libraryRow("book"))
            harness.repository.recordProgress(DL_DOC, 55f, "s", ChapterPosition("e5", 120))
            harness.goOffline()

            val document = harness.document()

            assertEquals(55, document.progressPercent)
            assertEquals("e5", document.chapterLocator)
        }

    @Test
    fun bookOpensFromAHomeRow() =
        runTest {
            val harness = readerOfflineHarness()
            harness.seen("home:continue", homeRow(DL_DOC, "book", progress = 62.5f))
            harness.goOffline()

            val document = harness.document()

            assertEquals("book", document.itemType)
            assertEquals(62, document.progressPercent)
            assertNull(document.libraryEntryId)
        }

    @Test
    fun articleKeepsTheLoadFailure() =
        runTest {
            val harness = readerOfflineHarness()
            harness.seen("triage:inbox:all", libraryRow("article"))
            harness.goOffline()

            assertTrue(harness.repository.getItem(DL_DOC).isFailure)
        }

    @Test
    fun aServerAnswerIsNotHiddenByALastSeenRow() =
        runTest {
            val harness = readerOfflineHarness()
            harness.seen("triage:inbox:all", libraryRow())
            listOf(HttpStatusCode.NotFound, HttpStatusCode.InternalServerError).forEach { status ->
                harness.server.route(READER_PATH) { respond("", status) }

                val failure = harness.repository.getItem(DL_DOC).exceptionOrNull()

                assertEquals(status.value, assertIs<ApiException>(failure).statusCode)
            }
        }

    @Test
    fun aCopyStillWinsOverTheLastSeenRow() =
        runTest {
            val harness = readerOfflineHarness()
            harness.keepCopy()
            harness.seen("triage:inbox:all", libraryRow())
            harness.goOffline()

            assertFalse(harness.document().lastSeen)
        }

    @Test
    fun signedOutNeverReadsTheLists() =
        runTest {
            val harness = readerOfflineHarness()
            harness.seen("triage:inbox:all", libraryRow())
            harness.goOffline()
            harness.signOut()

            assertTrue(harness.repository.getItem(DL_DOC).isFailure)
        }

    @Test
    fun readerTriageLeavesInboxButStaysInAll() =
        runTest {
            val harness = readerOfflineHarness()
            harness.seen("triage:inbox:all", libraryRow())
            harness.seen("triage:all:pdf", libraryRow())
            harness.server.route("/api/v1/library/entry_1/triage") {
                json(libraryEntryJson("entry_1", DL_DOC, triage = "later"))
            }

            harness.repository.triageItem("entry_1", "later")

            assertTrue(harness.store.list(harness.download.session.scope, "triage:inbox:all").isEmpty())
            assertEquals(1, harness.store.list(harness.download.session.scope, "triage:all:pdf").size)
        }
}
