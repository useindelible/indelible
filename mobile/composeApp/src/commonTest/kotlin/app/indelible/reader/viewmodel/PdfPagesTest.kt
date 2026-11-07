package app.indelible.reader.viewmodel

import app.indelible.core.offline.PagePosition
import kotlin.test.Test
import kotlin.test.assertEquals

class PdfPagesTest {
    @Test
    fun savedPageAndOffsetBecomeTheRestore() {
        val restore = pdfResumeTarget(PagePosition(12, 4200), percent = 60f, pageCount = 20, nonce = 1)

        assertEquals(PdfRestoreRequest(pageIndex = 11, pageOffset = 0.42f, nonce = 1), restore)
    }

    @Test
    fun savedPageBeyondTheLastPageOpensTheLastPageAtItsTop() {
        val restore = pdfResumeTarget(PagePosition(40, 4200), percent = 60f, pageCount = 20, nonce = 1)

        assertEquals(PdfRestoreRequest(pageIndex = 19, pageOffset = 0f, nonce = 1), restore)
    }

    @Test
    fun webPageLocatorOpensAtTheTopOfThatPage() {
        val restore = pdfResumeTarget(PagePosition(5, 0), percent = 60f, pageCount = 20, nonce = 1)

        assertEquals(PdfRestoreRequest(pageIndex = 4, pageOffset = 0f, nonce = 1), restore)
    }

    @Test
    fun noPositionFallsBackToThePercent() {
        val restore = pdfResumeTarget(saved = null, percent = 50f, pageCount = 20, nonce = 2)

        assertEquals(PdfRestoreRequest(pageIndex = 10, pageOffset = 0f, nonce = 2), restore)
    }

    @Test
    fun percentOfAHundredOpensTheLastPage() {
        val restore = pdfResumeTarget(saved = null, percent = 100f, pageCount = 20, nonce = 3)

        assertEquals(PdfRestoreRequest(pageIndex = 19, pageOffset = 0f, nonce = 3), restore)
    }
}
