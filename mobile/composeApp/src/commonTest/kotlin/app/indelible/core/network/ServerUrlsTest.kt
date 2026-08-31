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

    @Test
    fun normalizedOriginLowercasesSchemeAndHost() {
        assertEquals("https://my.host", normalizedOrigin("HTTPS://My.Host"))
    }

    @Test
    fun normalizedOriginDropsDefaultHttpsPort() {
        assertEquals("https://h", normalizedOrigin("https://h:443"))
    }

    @Test
    fun normalizedOriginDropsDefaultHttpPort() {
        assertEquals("http://h", normalizedOrigin("http://h:80"))
    }

    @Test
    fun normalizedOriginKeepsExplicitNonDefaultPort() {
        assertEquals("https://h:8443", normalizedOrigin("https://h:8443"))
    }

    @Test
    fun normalizedOriginBracketsIpv6Hosts() {
        assertEquals("https://[::1]:8443", normalizedOrigin("https://[::1]:8443/"))
    }

    @Test
    fun normalizedOriginKeepsNonRootPathPrefixMinusTrailingSlash() {
        assertEquals("https://h/indelible", normalizedOrigin("https://h/indelible/"))
    }

    @Test
    fun normalizedOriginProducesByteEqualOutputForDifferentSpellings() {
        assertEquals(
            normalizedOrigin("https://Library.UseIndelible.Test:443/"),
            normalizedOrigin("  HTTPS://library.useindelible.test/  "),
        )
    }

    @Test
    fun normalizedOriginFallsBackToTrimForUnparseableInput() {
        assertEquals("not a url", normalizedOrigin("  not a url/  "))
    }

    @Test
    fun normalizedOriginNeverThrowsOnBlankInput() {
        assertEquals("", normalizedOrigin("   "))
    }
}
