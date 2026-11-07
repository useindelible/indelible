package app.indelible.library.repository

import app.indelible.core.model.PaginatedItems
import app.indelible.core.network.ApiException
import app.indelible.core.network.LibraryApiService
import app.indelible.core.offline.DownloadHarness
import app.indelible.core.offline.ListSnapshotStore
import app.indelible.core.offline.ListSnapshots
import app.indelible.core.offline.Session
import app.indelible.core.offline.SnapshotRow
import app.indelible.core.offline.downloadHarness
import app.indelible.core.offline.json
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.async
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.io.IOException
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertTrue

private const val QUERY = "/api/v1/library/query"
private const val SECOND_SCOPE = "http://a.test|u2"

class ApiLibraryRepositorySnapshotTest {
    private class FailingWrites(
        delegate: ListSnapshotStore,
    ) : ListSnapshotStore by delegate {
        override suspend fun replaceList(
            session: Session,
            key: String,
            rows: List<SnapshotRow>,
            at: Long,
        ): Unit = error("database is full")
    }

    private fun DownloadHarness.repository(store: ListSnapshotStore = this.store): ApiLibraryRepository {
        val current = { signedIn.registry.current.value.session }
        return ApiLibraryRepository(LibraryApiService(transport), ListSnapshots(store, current) { 1L })
    }

    private suspend fun TestScope.harness(vararg ids: String): DownloadHarness =
        downloadHarness().apply { server.route(QUERY) { json(page(*ids)) } }

    private fun DownloadHarness.goOffline() = server.route(QUERY) { throw IOException("offline") }

    private fun DownloadHarness.answer(status: HttpStatusCode) = server.route(QUERY) { respond("", status) }

    @Test
    fun firstPageIsKeptAndShownWhenTheServerGivesNoAnswer() =
        runTest {
            val harness = harness("entry_1", "entry_2")
            val repository = harness.repository()
            repository.listItems("inbox", null, null, 50)
            harness.goOffline()

            val page = repository.listItems("inbox", null, null, 50).getOrThrow()

            assertEquals(listOf("entry_1", "entry_2"), page.data.map { it.id })
            assertTrue(page.fromDevice)
            assertFalse(page.page.hasMore)
        }

    @Test
    fun aGatewayGoneAnswerShowsTheKeptPage() =
        runTest {
            val harness = harness("entry_1")
            val repository = harness.repository()
            repository.listItems("inbox", null, null, 50)
            harness.answer(HttpStatusCode.ServiceUnavailable)

            assertTrue(repository.listItems("inbox", null, null, 50).getOrThrow().fromDevice)
        }

    @Test
    fun laterPagesAppendToTheKeptList() =
        runTest {
            val harness = harness("entry_1")
            val repository = harness.repository()
            repository.listItems("inbox", null, null, 50)
            harness.server.route(QUERY) { json(page("entry_2")) }
            repository.listItems("inbox", null, "cursor_2", 50)
            harness.goOffline()

            val page = repository.listItems("inbox", null, null, 50).getOrThrow()

            assertEquals(listOf("entry_1", "entry_2"), page.data.map { it.id })
        }

    @Test
    fun aFreshFirstPageReplacesTheKeptList() =
        runTest {
            val harness = harness("entry_1", "entry_2")
            val repository = harness.repository()
            repository.listItems("inbox", null, null, 50)
            harness.server.route(QUERY) { json(page("entry_3")) }
            repository.listItems("inbox", null, null, 50)
            harness.goOffline()

            assertEquals(listOf("entry_3"), repository.listItems("inbox", null, null, 50).ids())
        }

    @Test
    fun aLaterPageFailureIsNotHidden() =
        runTest {
            val harness = harness("entry_1")
            val repository = harness.repository()
            repository.listItems("inbox", null, null, 50)
            harness.goOffline()

            assertIs<IOException>(repository.listItems("inbox", null, "cursor_2", 50).exceptionOrNull())
        }

    @Test
    fun aServerAnswerIsNotHidden() =
        runTest {
            val harness = harness("entry_1")
            val repository = harness.repository()
            repository.listItems("inbox", null, null, 50)
            listOf(HttpStatusCode.Forbidden, HttpStatusCode.InternalServerError).forEach { status ->
                harness.answer(status)

                val error = repository.listItems("inbox", null, null, 50).exceptionOrNull()

                assertEquals(status.value, (error as ApiException).statusCode)
                assertEquals(true, harness.transport.reachable.value)
            }
        }

    @Test
    fun eachScopeAndFilterHasItsOwnList() =
        runTest {
            val harness = harness("entry_1")
            val repository = harness.repository()
            repository.listItems("inbox", null, null, 50)
            harness.server.route(QUERY) { json(page("entry_2")) }
            repository.listItems("later", "pdf", null, 50)
            harness.server.route("/api/v1/collections/col_1/entries") { json(page("entry_3")) }
            repository.listCollectionItems("col_1", null, 50)
            harness.goOffline()
            harness.server.route("/api/v1/collections/col_1/entries") { throw IOException("offline") }

            assertEquals(listOf("entry_1"), repository.listItems("inbox", null, null, 50).ids())
            assertEquals(listOf("entry_2"), repository.listItems("later", "pdf", null, 50).ids())
            assertEquals(listOf("entry_3"), repository.listCollectionItems("col_1", null, 50).ids())
            assertTrue(repository.listItems("archive", null, null, 50).isFailure)
        }

    @Test
    fun aResponseThatLandsAfterALoginToAnotherAccountIsNotKept() =
        runTest {
            val harness = harness()
            val gate = CompletableDeferred<Unit>()
            harness.server.route(QUERY) {
                gate.await()
                json(page("entry_1"))
            }
            val repository = harness.repository()
            val pending = async { repository.listItems("inbox", null, null, 50) }
            runCurrent()

            harness.signedIn.switchTo(SECOND_SCOPE)
            gate.complete(Unit)
            pending.await()

            assertTrue(harness.store.list(harness.session.scope, "triage:inbox:all").isEmpty())
            assertTrue(harness.store.list(SECOND_SCOPE, "triage:inbox:all").isEmpty())
        }

    @Test
    fun aStoreThatCannotWriteStillReturnsTheServerPage() =
        runTest {
            val harness = harness("entry_1")

            val page = harness.repository(FailingWrites(harness.store)).listItems("inbox", null, null, 50).getOrThrow()

            assertEquals(listOf("entry_1"), page.data.map { it.id })
            assertFalse(page.fromDevice)
        }

    @Test
    fun triagedItemLeavesInboxButStaysInAllAndLater() =
        runTest {
            val harness = harness()
            harness.seed("triage:inbox:all", "triage:all:pdf", "triage:later:all", "collection:c1")
            harness.server.route("/api/v1/library/entry_1/triage") { json(entry("entry_1", triage = "later")) }

            harness.repository().triageItem("entry_1", "later")

            assertEquals(listOf("triage:all:pdf", "triage:later:all", "collection:c1"), harness.keysHolding())
        }

    @Test
    fun aMovedItemSaysItsNewBucketInTheListsThatKeepIt() =
        runTest {
            val harness = harness()
            harness.seed("triage:all:pdf", "collection:c1")
            harness.server.route("/api/v1/library/entry_1/triage") { json(entry("entry_1", triage = "later")) }

            harness.repository().triageItem("entry_1", "later")

            listOf("triage:all:pdf", "collection:c1").forEach { key ->
                val row = harness.store.list(harness.session.scope, key).single()
                assertTrue("\"triage_state\":\"later\"" in row.rowJson, row.rowJson)
            }
        }

    @Test
    fun rearchivedItemSaysInboxInTheListsThatKeepIt() =
        runTest {
            val harness = harness()
            harness.seed("triage:all:pdf")
            harness.server.route("/api/v1/library/entry_1/restore") { json(entry("entry_1", triage = "inbox")) }

            harness.repository().rearchiveItem("entry_1")

            val row = harness.store.list(harness.session.scope, "triage:all:pdf").single()
            assertTrue("\"triage_state\":\"inbox\"" in row.rowJson, row.rowJson)
        }

    @Test
    fun rearchivedItemLeavesArchiveAndStaysInAll() =
        runTest {
            val harness = harness()
            harness.seed("triage:archive:all", "triage:all:pdf", "triage:inbox:pdf")
            harness.server.route("/api/v1/library/entry_1/restore") { json(entry("entry_1", triage = "inbox")) }

            harness.repository().rearchiveItem("entry_1")

            assertEquals(listOf("triage:all:pdf", "triage:inbox:pdf"), harness.keysHolding())
        }

    @Test
    fun deletedItemLeavesEveryList() =
        runTest {
            val harness = harness()
            harness.seed("triage:inbox:all", "triage:all:pdf", "collection:c1")
            harness.server.route("/api/v1/library/entry_1") { respond("", HttpStatusCode.NoContent) }

            harness.repository().deleteItem("entry_1")

            assertEquals(emptyList(), harness.keysHolding())
        }

    private val seededKeys = mutableListOf<String>()

    private suspend fun DownloadHarness.seed(vararg keys: String) {
        seededKeys += keys
        val row = SnapshotRow("entry_1", "doc_1", "pdf", "Title", "{}")
        keys.forEach { store.replaceList(session, it, listOf(row), at = 1) }
    }

    private suspend fun DownloadHarness.keysHolding(): List<String> =
        seededKeys.filter { store.list(session.scope, it).isNotEmpty() }
}

private fun Result<PaginatedItems>.ids(): List<String> = getOrThrow().data.map { it.id }

private fun entry(
    id: String,
    triage: String = "inbox",
): String = libraryEntryJson(id, triage = triage)

private fun page(vararg ids: String): String =
    """{"data":[${ids.joinToString(",") { entry(it) }}],"page":{"has_more":true,"next_cursor":"next"}}"""
