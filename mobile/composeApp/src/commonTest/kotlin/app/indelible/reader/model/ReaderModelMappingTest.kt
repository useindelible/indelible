package app.indelible.reader.model

import app.indelible.api.generated.models.DocumentReaderResponse
import app.indelible.api.generated.models.HighlightNoteResponse
import app.indelible.api.generated.models.HighlightWithNoteResponse
import app.indelible.api.generated.models.LocatorSchemaFlat
import app.indelible.api.generated.models.PdfRectSchema
import app.indelible.api.generated.models.TagResponse
import kotlinx.datetime.Instant
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class ReaderModelMappingTest {
    @Test
    fun onlyPagedAndChapteredContentCapturesItsPosition() {
        assertEquals(
            listOf(ReaderContentMode.PDF, ReaderContentMode.EPUB),
            ReaderContentMode.entries.filter { it.capturesPosition },
        )
    }

    @Test
    fun epubLocatorRoundTripsItsChapter() {
        val server = LocatorSchemaFlat(type = "epub", chapter = "ch3", startOffset = 12, endOffset = 40)

        val locator = server.toHighlightLocator()

        assertEquals(HighlightLocator(type = "epub", startOffset = 12, endOffset = 40, chapter = "ch3"), locator)
        assertEquals(server, locator.toLocatorSchemaFlat())
    }

    @Test
    fun pdfLocatorRoundTripsItsGeometry() {
        val server =
            LocatorSchemaFlat(
                type = "pdf",
                page = 3,
                x = 0.1,
                y = 0.2,
                width = 0.5,
                height = 0.04,
                textSnapshot = "quick brown",
                rects =
                    listOf(
                        PdfRectSchema(height = 0.02, width = 0.5, x = 0.1, y = 0.2),
                        PdfRectSchema(height = 0.02, width = 0.3, x = 0.1, y = 0.22),
                    ),
            )

        val locator = server.toHighlightLocator()

        assertEquals(3, locator.page)
        assertEquals(listOf(PdfRect(0.1, 0.2, 0.5, 0.02), PdfRect(0.1, 0.22, 0.3, 0.02)), locator.rects)
        assertEquals("quick brown", locator.textSnapshot)
        assertEquals(server, locator.toLocatorSchemaFlat())
    }

    @Test
    fun serverPdfHighlightKeepsItsRects() {
        val at = Instant.parse("2026-04-01T12:00:00Z")
        val rects = listOf(PdfRectSchema(height = 0.02, width = 0.5, x = 0.1, y = 0.2))
        val highlight =
            HighlightWithNoteResponse(
                color = "blue",
                createdAt = at,
                documentId = "doc_pdf",
                id = "hl_pdf",
                locator =
                    LocatorSchemaFlat(
                        type = "pdf",
                        page = 2,
                        x = 0.1,
                        y = 0.2,
                        width = 0.5,
                        height = 0.02,
                        rects = rects,
                    ),
                tags = emptyList(),
                textContent = "selected",
                updatedAt = at,
            )

        val mapped = highlight.toHighlightData()

        assertEquals(2, mapped.locator?.page)
        assertEquals(listOf(PdfRect(0.1, 0.2, 0.5, 0.02)), mapped.locator?.rects)
    }

    @Test
    fun mapsGeneratedReaderDocumentIntoAppModel() {
        val lastReadAt = Instant.parse("2026-04-01T12:00:00Z")
        val publishedAt = Instant.parse("2026-03-30T12:00:00Z")
        val savedAt = Instant.parse("2026-03-01T12:00:00Z")
        val response =
            DocumentReaderResponse(
                assets = emptyList(),
                author = "Ada",
                availableAssets = listOf("readable_html"),
                documentId = "doc_123",
                documentType = "article",
                domain = "example.com",
                lastReadAt = lastReadAt,
                leadImageUrl = "https://example.com/lead.jpg",
                libraryEntryId = "lib_123",
                progressPercent = 42,
                publishedAt = publishedAt,
                readableReady = true,
                readingTimeMinutes = 11,
                saved = true,
                savedAt = savedAt,
                summary = "Summary",
                thumbnailUrl = "https://example.com/thumb.jpg",
                title = "Mapped Reader",
                triageState = "later",
                url = "https://example.com/reader",
                wordCount = 2400,
            )

        val document = response.toReaderDocument()

        assertEquals("doc_123", document.id)
        assertEquals("article", document.itemType)
        assertEquals("later", document.triageState)
        assertEquals("example.com", document.domain)
        assertEquals(publishedAt, document.publishedAt)
        assertEquals(savedAt, document.savedAt)
        assertEquals(11, document.readingTimeMinutes)
        assertEquals(2400, document.wordCount)
        assertEquals("https://example.com/thumb.jpg", document.thumbnailUrl)
        assertEquals("https://example.com/lead.jpg", document.leadImageUrl)
    }

    @Test
    fun aServerWithoutTheFieldsLeavesTheBucketAndSavedDateUnknown() {
        val response =
            DocumentReaderResponse(
                assets = emptyList(),
                availableAssets = emptyList(),
                documentId = "doc_123",
                documentType = "article",
                lastReadAt = Instant.parse("2026-04-01T12:00:00Z"),
                libraryEntryId = "lib_123",
                readableReady = true,
                saved = true,
                title = "Older server",
            )

        val document = response.toReaderDocument()

        assertNull(document.triageState)
        assertNull(document.savedAt)
    }

    @Test
    fun mapsGeneratedHighlightAndTagIntoAppModels() {
        val createdAt = Instant.parse("2026-04-01T12:00:00Z")
        val note =
            HighlightNoteResponse(
                body = "Reader note",
                createdAt = createdAt,
                highlightId = "hl_123",
                id = "note_123",
                updatedAt = createdAt,
            )
        val highlight =
            HighlightWithNoteResponse(
                color = "yellow",
                createdAt = createdAt,
                documentId = "doc_123",
                id = "hl_123",
                locator = LocatorSchemaFlat(type = "html", startOffset = 4, endOffset = 12),
                note = note,
                tags = listOf("research"),
                textContent = "selected text",
                updatedAt = createdAt,
            )
        val tag =
            TagResponse(
                aliases = listOf("ai"),
                color = "blue",
                createdAt = createdAt,
                highlightCount = 2,
                id = "tag_123",
                itemCount = 3,
                name = "AI",
                `object` = "tag",
            )

        val mappedHighlight = highlight.toHighlightData()
        val mappedTag = tag.toTagData()

        assertEquals("hl_123", mappedHighlight.id)
        assertEquals("html", mappedHighlight.locator?.type)
        assertEquals(4, mappedHighlight.locator?.startOffset)
        assertEquals("Reader note", mappedHighlight.note?.body)
        assertEquals("tag_123", mappedTag.id)
        assertEquals("AI", mappedTag.name)
        assertEquals(2, mappedTag.highlightCount)
    }

    @Test
    fun appReaderModelsDoNotRequireWireOnlyFields() {
        val document =
            ReaderDocument(
                id = "doc_local",
                itemType = "article",
                title = "Local Reader",
                url = "https://example.com/local",
                saved = false,
                readableReady = true,
                availableAssets = listOf("readable_html"),
            )
        val highlight =
            HighlightData(
                id = "hl_local",
                color = "green",
                textContent = "local text",
                locator = HighlightLocator(type = "html", startOffset = 0, endOffset = 10),
                tags = emptyList(),
                createdAt = Instant.parse("2026-04-01T12:00:00Z"),
                updatedAt = Instant.parse("2026-04-01T12:00:00Z"),
            )

        assertEquals("doc_local", document.id)
        assertNull(document.libraryEntryId)
        assertFalse(document.saved)
        assertEquals(10, highlight.locator?.endOffset)
        assertTrue(highlight.tags.isEmpty())
    }
}
