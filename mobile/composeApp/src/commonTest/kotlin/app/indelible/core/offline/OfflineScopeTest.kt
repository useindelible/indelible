package app.indelible.core.offline

import app.indelible.core.storage.InMemoryTokenStorage
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class OfflineScopeTest {
    @Test
    fun scopeIsNullWithoutServerUrl() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveUserId("usr_1")

            assertNull(tokenStorage.currentOfflineScope())
        }

    @Test
    fun scopeIsNullWithoutCachedUserId() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveServerUrl("https://library.useindelible.test")

            assertNull(tokenStorage.currentOfflineScope())
        }

    @Test
    fun scopeCombinesNormalizedOriginAndUserId() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveServerUrl("  HTTPS://Library.UseIndelible.Test:443/  ")
            tokenStorage.saveUserId("usr_1")

            assertEquals("https://library.useindelible.test|usr_1", tokenStorage.currentOfflineScope())
        }

    @Test
    fun differentUsersOnTheSameServerYieldDifferentScopes() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveServerUrl("https://library.useindelible.test")
            tokenStorage.saveUserId("usr_1")
            val first = tokenStorage.currentOfflineScope()

            tokenStorage.saveUserId("usr_2")
            val second = tokenStorage.currentOfflineScope()

            assertEquals("https://library.useindelible.test|usr_1", first)
            assertEquals("https://library.useindelible.test|usr_2", second)
        }
}
