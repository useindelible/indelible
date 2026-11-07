package app.indelible.core.offline

import app.indelible.api.generated.models.DocumentReaderResponse
import app.indelible.api.generated.models.ReadingAnchorSchemaFlat
import app.indelible.api.generated.models.ReadingPositionSchema
import kotlinx.serialization.json.Json
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class ReadingPositionTest {
    @Test
    fun pageLocatorParsesLikeTheServer() {
        assertEquals(PagePosition(12, 7), readingPositionOf("page:12", 7))
        assertEquals(ChapterPosition("page:0", 7), readingPositionOf("page:0", 7))
        assertEquals(ChapterPosition("page:x", 7), readingPositionOf("page:x", 7))
        assertEquals(ChapterPosition("e5", 7), readingPositionOf("e5", 7))
    }

    @Test
    fun oddPageLocatorsAreChapters() {
        assertEquals(ChapterPosition("page:", 7), readingPositionOf("page:", 7))
        assertEquals(ChapterPosition("page:-1", 7), readingPositionOf("page:-1", 7))
        assertEquals(ChapterPosition("page:2147483648", 7), readingPositionOf("page:2147483648", 7))
    }

    @Test
    fun pagePositionWritesAPageAnchor() {
        val json = readingPositionJson(PagePosition(12, 4200), fraction = 0.37)

        val written = Json.decodeFromString(ReadingPositionSchema.serializer(), json)
        assertEquals(ReadingAnchorSchemaFlat(type = "page", page = 12), written.anchor)
        assertNull(written.anchor?.chapter)
        assertEquals(4200, written.offset)
        assertEquals(0.37, written.fraction)
    }

    @Test
    fun pageAnchorReadsBackAsAPagePosition() {
        val json = readingPositionJson(PagePosition(12, 4200), fraction = 0.37)

        assertEquals(PagePosition(12, 4200), readingPositionOf(json))
    }

    @Test
    fun spineAnchorStillReadsBackAsAChapter() {
        val json = readingPositionJson(ChapterPosition("e5", 120), fraction = 0.5)

        assertEquals(ChapterPosition("e5", 120), readingPositionOf(json))
    }

    @Test
    fun serverPageLocatorBecomesAPagePosition() {
        val response =
            DocumentReaderResponse(
                assets = emptyList(),
                availableAssets = listOf("pdf"),
                chapterLocator = "page:3",
                chapterOffset = 2500,
                documentId = "doc_pdf",
                documentType = "pdf",
                progressPercent = 12,
                readableReady = true,
                saved = true,
                title = "A PDF",
            )

        assertEquals(Progress(12, null, PagePosition(3, 2500)), response.progress())
    }
}
