package app.indelible.offline.viewmodel

import app.indelible.core.offline.DownloadHarness
import app.indelible.core.offline.downloadHarness
import app.indelible.core.offline.serverUnreachable
import app.indelible.core.storage.DEFAULT_OFFLINE_CAP_BYTES
import app.indelible.core.storage.InMemoryUserPreferencesStorage
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.yield

internal class Storage(
    val h: DownloadHarness,
    val preferences: InMemoryUserPreferencesStorage,
    online: Flow<Boolean> = flowOf(true),
    reachable: Flow<Boolean?> = flowOf(null),
) {
    var drains = 0

    init {
        h.capSource = { preferences.getOfflineCapBytes() }
    }

    val viewModel =
        StorageViewModel(
            registry = h.signedIn.registry,
            store = h.store,
            downloads = h.manager,
            preferences = preferences,
            requestDrain = { drains++ },
            unreachable = serverUnreachable(online, reachable),
        )

    val state: StorageUiState get() = viewModel.state.value

    private val shown = mutableListOf<StorageNotice>()

    // The collector runs in the background, which advanceUntilIdle leaves alone; one yield lets it take what was sent.
    suspend fun notices(): List<StorageNotice> {
        yield()
        return shown
    }

    fun change(id: String): PendingChange = state.changes.single { it.id == id }

    fun collectIn(scope: CoroutineScope) {
        scope.launch { viewModel.state.collect {} }
        scope.launch { viewModel.notices.collect { shown += it } }
    }
}

/** A Storage screen over a real offline store, with its state collected so the flows run. */
internal suspend fun TestScope.storage(
    cap: Long = DEFAULT_OFFLINE_CAP_BYTES,
    online: Flow<Boolean> = flowOf(true),
    reachable: Flow<Boolean?> = flowOf(null),
): Storage {
    val preferences = InMemoryUserPreferencesStorage().apply { saveOfflineCapBytes(cap) }
    return Storage(downloadHarness(), preferences, online, reachable).also { it.collectIn(backgroundScope) }
}
