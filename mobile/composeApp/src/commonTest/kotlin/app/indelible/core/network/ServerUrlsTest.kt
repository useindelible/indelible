package app.indelible.core.network

import app.indelible.core.storage.InMemoryTokenStorage
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals

class ServerUrlsTest {
    @Test
    fun resolvedServerUrlUsesCanonicalBakedDefaultBeforeLocalFallback() {
        assertEquals(
            "https://baked.useindelible.test",
            resolveServerUrl(
                storedUrl = null,
                bakedDefaultUrl = "  https://baked.useindelible.test/  ",
            ),
        )
    }

    @Test
    fun resolvedServerUrlCanonicalizesStoredUrlLikeTransport() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveServerUrl("  https://library.useindelible.test/  ")

            assertEquals("https://library.useindelible.test", tokenStorage.resolvedServerUrl())
        }
}
