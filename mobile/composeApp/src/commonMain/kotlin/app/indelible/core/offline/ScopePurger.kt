package app.indelible.core.offline

import kotlinx.coroutines.CancellationException

/** Deletes a scope; purge_pending is set first and cleared last, so a crash leaves it for [resumeInterrupted]. */
class ScopePurger(
    private val store: OfflineStore,
    private val deleteFiles: suspend (scope: String) -> Unit = {},
) {
    suspend fun purge(scope: String) {
        store.setPurgePending(scope, true)
        store.purgeRows(scope)
        deleteFiles(scope)
        store.finishPurge(scope)
    }

    suspend fun resumeInterrupted() {
        store
            .scopesWithState()
            .filter { (_, purgePending) -> purgePending }
            .forEach { (scope, _) -> purgeQuietly(scope) }
    }

    suspend fun purgeInactive(active: String?) {
        store
            .scopesWithState()
            .map { (scope, _) -> scope }
            .filter { it != active }
            .forEach { purgeQuietly(it) }
    }

    /** One scope's failure must not block the sweep; it stays purge_pending for the next one. */
    private suspend fun purgeQuietly(scope: String) {
        try {
            purge(scope)
        } catch (e: CancellationException) {
            throw e
        } catch (ignored: Exception) {
        }
    }
}
