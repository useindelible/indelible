package app.indelible.reader.ui

import app.indelible.reader.model.ReaderPreferences
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertContains
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class ReaderFontsTest {
    private fun page(assets: ReaderAssets = ReaderAssets.NONE): String =
        ReaderHtmlTemplate.build(
            articleHtml = "<p>Test</p>",
            preferences = ReaderPreferences(),
            highlights = emptyList(),
            assets = assets,
        )

    @Test
    fun theReaderAsksNoServerForFonts() {
        val out = page()

        assertFalse("fonts.googleapis" in out)
        assertFalse("fonts.gstatic" in out)
    }

    @Test
    fun bundledFacesAreEmbeddedInThePage() {
        val probe = "@font-face { font-family: 'Probe'; }"

        assertContains(page(ReaderAssets(artwork = null, fontFaceCss = probe)), probe)
    }

    @Test
    fun eachFaceIsAnInlineWoff2WithItsWeightRange() {
        val newsreader = READER_FONT_FACES.first { it.family == "Newsreader" && it.style == "normal" }

        val css = fontFaceCss(listOf(newsreader to byteArrayOf(1, 2, 3)))

        assertContains(css, "font-family: 'Newsreader'")
        assertContains(css, "font-style: normal")
        assertContains(css, "font-weight: 200 800")
        assertContains(css, "url(data:font/woff2;base64,AQID)")
    }

    @Test
    fun everyFamilyTheReaderNamesIsBundled() {
        assertEquals(setOf("Newsreader", "Geist", "Geist Mono"), READER_FONT_FACES.map { it.family }.toSet())
        assertTrue(READER_FONT_FACES.any { it.family == "Newsreader" && it.style == "italic" })
    }

    @Test
    fun theBundledFilesLoad() =
        runTest {
            val css = loadReaderFontCss()

            READER_FONT_FACES.forEach { assertContains(css, "font-family: '${it.family}'") }
            assertTrue(css.length > 100_000)
        }
}
