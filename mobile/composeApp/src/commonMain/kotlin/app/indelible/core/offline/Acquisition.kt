package app.indelible.core.offline

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.update

enum class DownloadFailure { NO_SPACE, NETWORK, SERVER }

/** Where a requested download stands before its copy exists; a finished one leaves the map. */
sealed interface Acquisition {
    data object Waiting : Acquisition

    /** A pin made while offline; it starts when connectivity returns. */
    data object WaitingForConnection : Acquisition

    data class Downloading(
        val fraction: Float,
    ) : Acquisition

    data class Failed(
        val reason: DownloadFailure,
    ) : Acquisition
}

internal data class DocumentKey(
    val scope: String,
    val documentId: String,
)

/** Live acquisition states; a screen observes only its own scope's documents. */
class AcquisitionBoard {
    private val state = MutableStateFlow<Map<DocumentKey, Acquisition>>(emptyMap())

    fun observe(scope: String): Flow<Map<String, Acquisition>> =
        state
            .map { all -> all.filterKeys { it.scope == scope }.mapKeys { it.key.documentId } }
            .distinctUntilChanged()

    internal fun set(
        key: DocumentKey,
        acquisition: Acquisition,
    ) {
        state.update { it + (key to acquisition) }
    }

    internal fun clear(key: DocumentKey) {
        state.update { it - key }
    }

    internal fun clearScope(scope: String) {
        state.update { all -> all.filterKeys { it.scope != scope } }
    }
}
