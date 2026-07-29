package app.indelible.core.storage

import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class TokenStorageTest {
    @Test
    fun saveAndRetrieveToken() =
        runTest {
            val storage = InMemoryTokenStorage()
            storage.saveToken("test-token")
            assertEquals("test-token", storage.getToken())
        }

    @Test
    fun getTokenReturnsNullWhenEmpty() =
        runTest {
            val storage = InMemoryTokenStorage()
            assertNull(storage.getToken())
        }

    @Test
    fun clearTokenRemovesToken() =
        runTest {
            val storage = InMemoryTokenStorage()
            storage.saveToken("test-token")
            storage.clearToken()
            assertNull(storage.getToken())
        }

    @Test
    fun saveAndRetrieveServerUrl() =
        runTest {
            val storage = InMemoryTokenStorage()
            storage.saveServerUrl("https://custom.server.com")
            assertEquals("https://custom.server.com", storage.getServerUrl())
        }

    @Test
    fun getServerUrlReturnsNullWhenEmpty() =
        runTest {
            val storage = InMemoryTokenStorage()
            assertNull(storage.getServerUrl())
        }

    @Test
    fun saveTokenOverwritesPrevious() =
        runTest {
            val storage = InMemoryTokenStorage()
            storage.saveToken("first")
            storage.saveToken("second")
            assertEquals("second", storage.getToken())
        }

    @Test
    fun saveAndRetrieveRefreshToken() =
        runTest {
            val storage = InMemoryTokenStorage()
            storage.saveRefreshToken("refresh-token")
            assertEquals("refresh-token", storage.getRefreshToken())
        }

    @Test
    fun saveAndRetrieveExpiresAt() =
        runTest {
            val storage = InMemoryTokenStorage()
            storage.saveExpiresAt(12345L)
            assertEquals(12345L, storage.getExpiresAt())
        }

    @Test
    fun clearAllRemovesAllTokens() =
        runTest {
            val storage = InMemoryTokenStorage()
            storage.saveToken("access")
            storage.saveRefreshToken("refresh")
            storage.saveExpiresAt(12345L)

            storage.clearAll()

            assertNull(storage.getToken())
            assertNull(storage.getRefreshToken())
            assertNull(storage.getExpiresAt())
        }

    @Test
    fun clearAllPreservesServerUrl() =
        runTest {
            val storage = InMemoryTokenStorage()
            storage.saveToken("access")
            storage.saveRefreshToken("refresh")
            storage.saveExpiresAt(12345L)
            storage.saveServerUrl("https://custom.server.com")

            storage.clearAll()

            assertEquals("https://custom.server.com", storage.getServerUrl())
        }

    @Test
    fun clearAllRemovesPendingItems() =
        runTest {
            val storage = InMemoryTokenStorage()
            storage.savePendingItems("""[{"id":"pending-1"}]""")

            storage.clearAll()

            assertNull(storage.getPendingItems())
        }
}
