package app.indelible.core.offline

import app.indelible.core.network.gaveNoAnswer
import kotlinx.coroutines.CancellationException

/** One list row as the server sent it; [rowJson] is the generated DTO, decoded by the screen that owns the key. */
data class SnapshotRow(
    val itemId: String,
    val documentId: String,
    val documentType: String,
    val title: String,
    val rowJson: String,
)

/** A list as last kept; [seenAt] is when its newest rows arrived, null for a list never kept. */
data class KeptList(
    val rows: List<SnapshotRow>,
    val seenAt: Long?,
)

/** Takes the session captured before the request, so a late response never writes or reads another account's rows. */
class ListSnapshots(
    private val store: ListSnapshotStore,
    private val sessionProvider: () -> Session?,
    private val clock: () -> Long,
) {
    suspend fun keep(
        session: Session?,
        key: String,
        rows: List<SnapshotRow>,
        append: Boolean = false,
    ) {
        session ?: return
        bestEffort {
            if (append) {
                store.appendList(session, key, rows, clock())
            } else {
                store.replaceList(session, key, rows, clock())
            }
        }
    }

    suspend fun kept(
        session: Session?,
        key: String,
    ): List<SnapshotRow> = whileCurrent(session) { store.list(it.scope, key) }.orEmpty()

    suspend fun lastSeen(
        session: Session?,
        documentId: String,
    ): SnapshotRow? = whileCurrent(session) { store.lastSeen(it.scope, documentId) }

    suspend fun forget(
        session: Session?,
        id: String,
        keep: (String) -> Boolean = { false },
    ) {
        session ?: return
        bestEffort { store.forget(session, id, keep) }
    }

    /** What the phone keeps of a moved document says [state], for reopening it with no network. */
    suspend fun setTriageState(
        session: Session?,
        documentId: String,
        state: String,
    ) {
        session ?: return
        bestEffort { store.setTriageState(session, documentId, state) }
    }

    fun currentSession(): Session? = sessionProvider()

    private fun current(session: Session?): Session? = session?.takeIf { it === sessionProvider() }

    // Checked again after the read: the session can end while the read waits for the database.
    private suspend fun <T> whileCurrent(
        session: Session?,
        read: suspend (Session) -> T,
    ): T? = current(session)?.let { live -> read(live).takeIf { current(live) != null } }

    // The server's result always stands: a stale session, a full disk or a database error only loses the snapshot.
    private suspend fun bestEffort(write: suspend () -> Unit) {
        runCatching { write() }.onFailure { if (it is CancellationException) throw it }
    }
}

/** The device's value in place of a failure, only when no server answered and the device has one. */
suspend fun <V> Result<V>.orFromDevice(fromDevice: suspend () -> V?): Result<V> {
    val failure = exceptionOrNull()
    if (failure is CancellationException) throw failure
    val kept = if (failure?.gaveNoAnswer() == true) deviceValue(fromDevice) else null
    return if (kept != null) Result.success(kept) else this
}

private suspend fun <V> deviceValue(fromDevice: suspend () -> V?): V? =
    runCatching { fromDevice() }
        .onFailure { if (it is CancellationException) throw it }
        .getOrNull()
