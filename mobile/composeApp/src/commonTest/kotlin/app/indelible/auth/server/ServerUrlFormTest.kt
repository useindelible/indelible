package app.indelible.auth.server

import app.indelible.core.i18n.UiMessage
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.auth_server_address_invalid
import indelible.composeapp.generated.resources.auth_server_address_required
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class ServerUrlFormTest {
    @Test
    fun normalizeTrimsWhitespaceAndTrailingSlashes() {
        assertEquals(
            "https://indelible.example.com",
            ServerUrlForm.normalize("  https://indelible.example.com/  "),
        )
    }

    @Test
    fun normalizePrefixesHttpsWhenSchemeOmitted() {
        assertEquals(
            "https://indelible.example.com",
            ServerUrlForm.normalize("indelible.example.com"),
        )
    }

    @Test
    fun normalizeKeepsExplicitHttpScheme() {
        assertEquals(
            "http://192.168.1.40:38473",
            ServerUrlForm.normalize("http://192.168.1.40:38473/"),
        )
    }

    @Test
    fun normalizeKeepsSubpaths() {
        assertEquals(
            "https://home.example.com/indelible",
            ServerUrlForm.normalize("home.example.com/indelible/"),
        )
    }

    @Test
    fun blankInputIsInvalid() {
        val result = ServerUrlForm.validate("   ")
        assertIs<ServerUrlValidation.Invalid>(result)
        assertEquals(UiMessage(Res.string.auth_server_address_required), result.message)
    }

    @Test
    fun unparseableInputIsInvalid() {
        val result = ServerUrlForm.validate("ht tp://broken host")
        assertIs<ServerUrlValidation.Invalid>(result)
        assertEquals(UiMessage(Res.string.auth_server_address_invalid), result.message)
    }

    @Test
    fun schemeOnlyInputIsInvalid() {
        assertIs<ServerUrlValidation.Invalid>(ServerUrlForm.validate("https://"))
    }

    @Test
    fun httpsRemoteHostIsReady() {
        val result = ServerUrlForm.validate("indelible.example.com")
        assertIs<ServerUrlValidation.Ready>(result)
        assertEquals("https://indelible.example.com", result.url)
    }

    @Test
    fun httpRemoteHostNeedsCleartextConsent() {
        val result = ServerUrlForm.validate("http://192.168.1.40:38473")
        assertIs<ServerUrlValidation.NeedsCleartextConsent>(result)
        assertEquals("http://192.168.1.40:38473", result.url)
    }

    @Test
    fun httpLoopbackHostsSkipTheConsentGate() {
        listOf(
            "http://localhost:38473",
            "http://127.0.0.1:38473",
            "http://10.0.2.2:38473",
            "http://10.0.3.2",
        ).forEach { candidate ->
            val result = ServerUrlForm.validate(candidate)
            assertIs<ServerUrlValidation.Ready>(result, "expected Ready for $candidate")
        }
    }

    @Test
    fun httpsLoopbackIsReadyWithoutConsent() {
        assertIs<ServerUrlValidation.Ready>(ServerUrlForm.validate("https://localhost:38473"))
    }

    @Test
    fun bareTailscaleStyleHttpAddressAsksForConsentNotRejection() {
        val result = ServerUrlForm.validate("http://100.64.0.7:38473")
        assertIs<ServerUrlValidation.NeedsCleartextConsent>(result)
        assertEquals("http://100.64.0.7:38473", result.url)
    }

    @Test
    fun pastingAFullUrlAfterTheHttpsStubDropsTheStub() {
        assertEquals(
            "http://100.64.0.7:38473",
            ServerUrlForm.normalize("https://http://100.64.0.7:38473"),
        )
        assertIs<ServerUrlValidation.NeedsCleartextConsent>(
            ServerUrlForm.validate("https://http://100.64.0.7:38473"),
        )
    }

    @Test
    fun schemeInsideAQueryStringIsNotMistakenForAStub() {
        assertEquals(
            "https://example.com/proxy?url=http://inner",
            ServerUrlForm.normalize("https://example.com/proxy?url=http://inner"),
        )
    }

    @Test
    fun displayHostStripsSchemePortAndPath() {
        assertEquals("indelible.acme.dev", ServerUrlForm.displayHost("https://indelible.acme.dev:38473/base"))
        assertEquals("192.168.1.40", ServerUrlForm.displayHost("http://192.168.1.40:38473"))
    }

    @Test
    fun displayHostFallsBackToRawInputWhenUnparseable() {
        assertEquals("not a url", ServerUrlForm.displayHost("not a url"))
    }
}
