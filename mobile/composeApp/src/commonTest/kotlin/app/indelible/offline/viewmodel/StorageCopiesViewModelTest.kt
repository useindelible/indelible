package app.indelible.offline.viewmodel

import app.indelible.core.offline.DL_DOC
import app.indelible.core.offline.DL_SCOPE
import app.indelible.core.offline.DownloadHarness
import app.indelible.core.offline.FailingWriteFileSystem
import app.indelible.core.offline.downloadHarness
import app.indelible.core.offline.seed
import app.indelible.core.storage.InMemoryUserPreferencesStorage
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import okio.IOException
import okio.fakefilesystem.FakeFileSystem
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class StorageCopiesViewModelTest {
    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(StandardTestDispatcher())
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private fun TestScope.storage(h: DownloadHarness): StorageViewModel {
        val viewModel =
            StorageViewModel(
                registry = h.signedIn.registry,
                store = h.store,
                downloads = h.manager,
                preferences = InMemoryUserPreferencesStorage(),
                requestDrain = {},
            )
        backgroundScope.launch { viewModel.state.collect {} }
        return viewModel
    }

    @Test
    fun aFullDiskPauseReachesTheState() =
        runTest {
            val full = FailingWriteFileSystem(FakeFileSystem()) { IOException("write failed: ENOSPC") }
            val h = downloadHarness(fs = full)
            val viewModel = storage(h)
            advanceUntilIdle()
            assertFalse(viewModel.state.value.autoCachePaused)

            h.manager.cacheOnOpen(h.session, DL_DOC)
            advanceUntilIdle()

            assertTrue(viewModel.state.value.autoCachePaused)
        }

    @Test
    fun keepingACachedCopyPinsItInPlace() =
        runTest {
            val h = downloadHarness()
            h.seed(DL_DOC, bytes = 10, pinned = false)
            val viewModel = storage(h)
            advanceUntilIdle()
            assertEquals(1, viewModel.state.value.cached.size)

            viewModel.keep(DL_DOC)
            advanceUntilIdle()

            val copy = checkNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertTrue(copy.pinned)
            assertEquals(1L, copy.generation)
            assertEquals(1, viewModel.state.value.kept.size)
            assertTrue(viewModel.state.value.cached.isEmpty())
        }
}
