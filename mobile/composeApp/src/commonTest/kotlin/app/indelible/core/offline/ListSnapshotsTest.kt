package app.indelible.core.offline

import app.indelible.core.network.ApiException
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.async
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import kotlinx.io.IOException
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull
import kotlin.test.assertSame
import kotlin.test.assertTrue

class ListSnapshotsTest {
    private val rows = listOf(SnapshotRow("doc_1", "doc_1", "pdf", "Title", "{}"))

    private class ThrowingWrites(
        delegate: ListSnapshotStore,
        private val error: Exception,
    ) : ListSnapshotStore by delegate {
        override suspend fun replaceList(
            session: Session,
            key: String,
            rows: List<SnapshotRow>,
            at: Long,
        ): Unit = throw error
    }

    private class GatedReads(
        delegate: ListSnapshotStore,
        private val gate: CompletableDeferred<Unit>,
    ) : ListSnapshotStore by delegate {
        private val store = delegate

        override suspend fun list(
            scope: String,
            key: String,
        ): List<SnapshotRow> = gate.await().let { store.list(scope, key) }

        override suspend fun lastSeen(
            scope: String,
            documentId: String,
        ): SnapshotRow? = gate.await().let { store.lastSeen(scope, documentId) }
    }

    private fun SignedIn.snapshots(store: ListSnapshotStore = this.store): ListSnapshots {
        val current = { registry.current.value.session }
        return ListSnapshots(store, current) { 7L }
    }

    private suspend fun TestScope.signedIn() = signedInTestStore("origin|a")

    @Test
    fun noSessionKeepsAndReadsNothing() =
        runTest {
            val signedIn = signedIn()
            val snapshots = signedIn.snapshots()

            snapshots.keep(null, "home:recent", rows)

            assertTrue(signedIn.store.list(signedIn.scope, "home:recent").isEmpty())
            assertTrue(snapshots.kept(null, "home:recent").isEmpty())
            assertNull(snapshots.lastSeen(null, "doc_1"))
        }

    @Test
    fun aSessionThatEndedBeforeTheResponseWritesNothing() =
        runTest {
            val signedIn = signedIn()
            val snapshots = signedIn.snapshots()
            val first = signedIn.session
            signedIn.switchTo("origin|b")

            snapshots.keep(first, "home:recent", rows)

            assertTrue(signedIn.store.list("origin|a", "home:recent").isEmpty())
            assertTrue(signedIn.store.list("origin|b", "home:recent").isEmpty())
        }

    @Test
    fun aSessionThatEndedBeforeTheResponseReadsNothing() =
        runTest {
            val signedIn = signedIn()
            val snapshots = signedIn.snapshots()
            snapshots.keep(signedIn.session, "home:recent", rows)

            signedIn.registry.publish(SessionState(signedIn.session.epoch + 1, null))

            assertTrue(snapshots.kept(signedIn.session, "home:recent").isEmpty())
            assertNull(snapshots.lastSeen(signedIn.session, "doc_1"))
            assertEquals(rows, signedIn.store.list(signedIn.scope, "home:recent"))
        }

    @Test
    fun aSessionThatEndsDuringTheReadReadsNothing() =
        runTest {
            val signedIn = signedIn()
            val gate = CompletableDeferred<Unit>()
            val snapshots = signedIn.snapshots(GatedReads(signedIn.store, gate))
            snapshots.keep(signedIn.session, "home:recent", rows)
            val kept = async { snapshots.kept(signedIn.session, "home:recent") }
            val seen = async { snapshots.lastSeen(signedIn.session, "doc_1") }
            testScheduler.advanceUntilIdle()

            signedIn.registry.publish(SessionState(signedIn.session.epoch + 1, null))
            gate.complete(Unit)

            assertTrue(kept.await().isEmpty())
            assertNull(seen.await())
        }

    @Test
    fun aWriteFailureIsDroppedQuietly() =
        runTest {
            val signedIn = signedIn()
            val snapshots = signedIn.snapshots(ThrowingWrites(signedIn.store, IllegalStateException("disk full")))

            snapshots.keep(signedIn.session, "home:recent", rows)

            assertTrue(signedIn.store.list(signedIn.scope, "home:recent").isEmpty())
        }

    @Test
    fun aCancelledWriteIsRethrown() =
        runTest {
            val signedIn = signedIn()
            val snapshots = signedIn.snapshots(ThrowingWrites(signedIn.store, CancellationException("left")))

            assertFailsWith<CancellationException> { snapshots.keep(signedIn.session, "home:recent", rows) }
        }

    @Test
    fun noAnswerShowsTheDeviceValue() =
        runTest {
            listOf(IOException("offline"), ApiException(503, "unavailable")).forEach { error ->
                val result = Result.failure<String>(error).orFromDevice { "kept" }

                assertEquals("kept", result.getOrNull(), "${error::class.simpleName}")
            }
        }

    @Test
    fun aServerAnswerIsKept() =
        runTest {
            listOf(404, 500).forEach { status ->
                val error = ApiException(status, "answer")

                val result = Result.failure<String>(error).orFromDevice { "kept" }

                assertSame(error, result.exceptionOrNull(), "status $status")
            }
        }

    @Test
    fun noDeviceValueKeepsTheFailure() =
        runTest {
            val error = IOException("offline")

            assertSame(error, Result.failure<String>(error).orFromDevice { null }.exceptionOrNull())
        }

    @Test
    fun aDeviceReadFailureKeepsTheServerFailure() =
        runTest {
            val error = IOException("offline")

            val result = Result.failure<String>(error).orFromDevice { throw IllegalStateException("database") }

            assertSame(error, result.exceptionOrNull())
        }

    @Test
    fun cancellationIsRethrown() =
        runTest {
            assertFailsWith<CancellationException> {
                Result.failure<String>(CancellationException("left")).orFromDevice { "kept" }
            }
        }
}
