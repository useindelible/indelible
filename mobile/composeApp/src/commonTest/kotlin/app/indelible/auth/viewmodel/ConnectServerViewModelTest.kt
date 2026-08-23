package app.indelible.auth.viewmodel

import app.indelible.auth.server.ServerHealthChecker
import app.indelible.core.i18n.UiMessage
import app.indelible.core.storage.InMemoryTokenStorage
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.auth_server_address_required
import indelible.composeapp.generated.resources.auth_server_unreachable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class ConnectServerViewModelTest {
    private class FakeServerHealth(
        var result: Result<Unit> = Result.success(Unit),
    ) : ServerHealthChecker {
        val checkedUrls = mutableListOf<String>()

        override suspend fun check(baseUrl: String): Result<Unit> {
            checkedUrls += baseUrl
            return result
        }
    }

    private val storage = InMemoryTokenStorage()
    private val health = FakeServerHealth()

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private fun viewModel(
        bakedDefaultUrl: String = "",
        devPrefillUrl: String = "",
    ) = ConnectServerViewModel(
        tokenStorage = storage,
        healthChecker = health,
        bakedDefaultUrl = bakedDefaultUrl,
        devPrefillUrl = devPrefillUrl,
    )

    @Test
    fun setupIsRequiredWithNoStoredUrlAndNoBakedDefault() =
        runTest {
            val vm = viewModel()
            assertIs<ServerSetupState.Required>(vm.setupState.value)
            assertEquals("", vm.state.value.url)
        }

    @Test
    fun storedUrlConfiguresSetupAndPrefillsTheField() =
        runTest {
            storage.saveServerUrl("https://indelible.acme.dev")
            val vm = viewModel()
            val setup = vm.setupState.value
            assertIs<ServerSetupState.Configured>(setup)
            assertEquals("https://indelible.acme.dev", setup.serverUrl)
            assertEquals("https://indelible.acme.dev", vm.state.value.url)
        }

    @Test
    fun bakedDefaultConfiguresSetupWithoutStoredUrl() =
        runTest {
            val vm = viewModel(bakedDefaultUrl = "https://api.useindelible.com")
            val setup = vm.setupState.value
            assertIs<ServerSetupState.Configured>(setup)
            assertEquals("https://api.useindelible.com", setup.serverUrl)
            assertEquals("https://api.useindelible.com", vm.state.value.url)
        }

    @Test
    fun devPrefillFillsTheFieldButDoesNotConfigureSetup() =
        runTest {
            val vm = viewModel(devPrefillUrl = "http://localhost:38473")
            assertIs<ServerSetupState.Required>(vm.setupState.value)
            assertEquals("http://localhost:38473", vm.state.value.url)
        }

    @Test
    fun invalidAddressSetsErrorWithoutAnyRequestOrPersistence() =
        runTest {
            val vm = viewModel()
            vm.updateUrl("   ")
            vm.connect()
            assertEquals(UiMessage(Res.string.auth_server_address_required), vm.state.value.error)
            assertTrue(health.checkedUrls.isEmpty())
            assertNull(storage.getServerUrl())
        }

    @Test
    fun healthyHttpsServerIsPersistedAndReportedConnected() =
        runTest {
            val vm = viewModel()
            vm.updateUrl(" indelible.acme.dev ")
            vm.connect()
            assertEquals(listOf("https://indelible.acme.dev"), health.checkedUrls)
            assertEquals("https://indelible.acme.dev", storage.getServerUrl())
            assertEquals("https://indelible.acme.dev", vm.connectedUrl.value)
            val setup = vm.setupState.value
            assertIs<ServerSetupState.Configured>(setup)
            assertEquals("https://indelible.acme.dev", setup.serverUrl)
        }

    @Test
    fun httpRemoteHostWaitsForCleartextConsentBeforeAnyRequest() =
        runTest {
            val vm = viewModel()
            vm.updateUrl("http://192.168.1.40:38473")
            vm.connect()
            assertEquals("http://192.168.1.40:38473", vm.state.value.pendingCleartextUrl)
            assertTrue(health.checkedUrls.isEmpty())
            assertNull(storage.getServerUrl())

            vm.confirmCleartext()
            assertEquals(listOf("http://192.168.1.40:38473"), health.checkedUrls)
            assertEquals("http://192.168.1.40:38473", storage.getServerUrl())
        }

    @Test
    fun dismissingTheCleartextWarningKeepsNothing() =
        runTest {
            val vm = viewModel()
            vm.updateUrl("http://192.168.1.40:38473")
            vm.connect()
            vm.dismissCleartextWarning()
            assertNull(vm.state.value.pendingCleartextUrl)
            assertTrue(health.checkedUrls.isEmpty())
            assertNull(storage.getServerUrl())
        }

    @Test
    fun loopbackHttpSkipsTheConsentGate() =
        runTest {
            val vm = viewModel()
            vm.updateUrl("http://localhost:38473")
            vm.connect()
            assertNull(vm.state.value.pendingCleartextUrl)
            assertEquals(listOf("http://localhost:38473"), health.checkedUrls)
        }

    @Test
    fun unreachableServerSurfacesTheErrorAndPersistsNothing() =
        runTest {
            health.result = Result.failure(IllegalStateException("connection refused"))
            val vm = viewModel()
            vm.updateUrl("https://indelible.acme.dev")
            vm.connect()
            assertEquals(UiMessage(Res.string.auth_server_unreachable), vm.state.value.error)
            assertNull(storage.getServerUrl())
            assertNull(vm.connectedUrl.value)
            assertIs<ServerSetupState.Required>(vm.setupState.value)
        }

    @Test
    fun editingTheUrlClearsAPreviousError() =
        runTest {
            val vm = viewModel()
            vm.updateUrl("")
            vm.connect()
            vm.updateUrl("indelible.acme.dev")
            assertNull(vm.state.value.error)
        }

    @Test
    fun consumingTheConnectedEventClearsIt() =
        runTest {
            val vm = viewModel()
            vm.updateUrl("indelible.acme.dev")
            vm.connect()
            vm.consumeConnectedEvent()
            assertNull(vm.connectedUrl.value)
        }
}
