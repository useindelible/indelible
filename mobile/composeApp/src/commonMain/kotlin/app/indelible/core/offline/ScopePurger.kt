package app.indelible.core.offline

import kotlinx.coroutines.CancellationException

/**
 * Deletes every local trace of an offline scope. The step order is load-bearing: `client_state`
 * carries the `purge_pending` recovery flag, so it must be written first and removed last. A
 * crash between any two steps leaves the flag set, and [resumeInterrupted] re-runs [purge] for
 * that scope from the top; every step is idempotent so replaying them is safe.
 */
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

    /**
     * A scope stuck on one bad file must not block the rest of a sweep, nor the sign-in and
     * startup paths that trigger one: the failure is swallowed per scope, which stays
     * purge_pending and is retried on the next sweep.
     */
    private suspend fun purgeQuietly(scope: String) {
        try {
            purge(scope)
        } catch (e: CancellationException) {
            throw e
        } catch (ignored: Exception) {
            // scope remains purge_pending
        }
    }
}
