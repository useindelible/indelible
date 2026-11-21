package app.indelible.offline.viewmodel

import app.indelible.core.offline.downloadHarness
import app.indelible.core.storage.InMemoryUserPreferencesStorage
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

@OptIn(ExperimentalCoroutinesApi::class)
class StorageViewModelWaitingTest {
    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(StandardTestDispatcher())
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun theWaitingCountIsUnknownUntilTheQueueIsRead() =
        runTest {
            val preferences = InMemoryUserPreferencesStorage()
            val h = downloadHarness()
            h.capSource = { preferences.getOfflineCapBytes() }
            val viewModel =
                StorageViewModel(h.signedIn.registry, h.store, h.manager, preferences, requestDrain = {})
            assertNull(viewModel.state.value.waitingChanges)

            backgroundScope.launch { viewModel.state.collect {} }
            advanceUntilIdle()

            assertEquals(0, viewModel.state.value.waitingChanges)
        }
}
