package app.indelible.offline.viewmodel

import app.indelible.core.offline.AcquisitionBoard
import app.indelible.core.offline.OfflineStore
import app.indelible.core.offline.SessionRegistry
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map

/** The offline status of every document the signed-in account has on the device, acquiring or queued. */
class OfflineStatuses(
    private val registry: SessionRegistry,
    private val store: OfflineStore,
    private val acquisitions: AcquisitionBoard,
) {
    @OptIn(ExperimentalCoroutinesApi::class)
    fun observe(): Flow<Map<String, DocumentOfflineStatus>> =
        registry.current
            .map { it.session?.scope }
            .distinctUntilChanged()
            .flatMapLatest { scope -> scope?.let(::statusesOf) ?: flowOf(emptyMap()) }

    private fun statusesOf(scope: String): Flow<Map<String, DocumentOfflineStatus>> =
        combine(
            store.observeCatalog(scope),
            acquisitions.observe(scope),
            store.observeDocumentSyncCounts(scope),
        ) { catalog, acquiring, counts ->
            val copies = catalog.associateBy { it.documentId }
            (copies.keys + acquiring.keys + counts.keys).associateWith { id ->
                documentOfflineStatus(copies[id], acquiring[id], counts[id])
            }
        }
}
