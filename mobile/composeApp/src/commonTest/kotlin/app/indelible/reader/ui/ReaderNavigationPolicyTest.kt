package app.indelible.reader.ui

import kotlin.test.Test
import kotlin.test.assertEquals

class ReaderNavigationPolicyTest {
    @Test
    fun allows_subframe_navigation() {
        val decision =
            ReaderNavigationPolicy.decide(
                isMainFrame = false,
                scheme = "https",
                initialReaderDocumentPending = false,
            )

        assertEquals(ReaderNavigationDecision.Allow, decision)
    }

    @Test
    fun allows_only_the_initial_about_document() {
        val decision =
            ReaderNavigationPolicy.decide(
                isMainFrame = true,
                scheme = "about",
                initialReaderDocumentPending = true,
            )

        assertEquals(ReaderNavigationDecision.AllowInitialDocument, decision)
    }

    @Test
    fun opens_early_http_navigation_externally() {
        val decision =
            ReaderNavigationPolicy.decide(
                isMainFrame = true,
                scheme = "https",
                initialReaderDocumentPending = true,
            )

        assertEquals(ReaderNavigationDecision.OpenExternally, decision)
    }

    @Test
    fun cancels_non_http_main_frame_navigation() {
        val decision =
            ReaderNavigationPolicy.decide(
                isMainFrame = true,
                scheme = "javascript",
                initialReaderDocumentPending = false,
            )

        assertEquals(ReaderNavigationDecision.Cancel, decision)
    }

    @Test
    fun cancels_about_navigation_after_initial_document() {
        val decision =
            ReaderNavigationPolicy.decide(
                isMainFrame = true,
                scheme = "about",
                initialReaderDocumentPending = false,
            )

        assertEquals(ReaderNavigationDecision.Cancel, decision)
    }
}
