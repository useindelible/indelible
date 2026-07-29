package app.indelible.core.model

import app.indelible.api.generated.models.LibraryEntryResponse
import kotlinx.datetime.Instant
import kotlin.test.Test
import kotlin.test.assertEquals

class LibraryItemMappingTest {
    @Test
    fun mapsGeneratedLibraryEntryIntoAppDomainModel() {
        val response =
            LibraryEntryResponse(
                author = "Ada Lovelace",
                canonicalUrl = "https://example.com/canonical",
                createdAt = Instant.parse("2026-01-01T00:00:00Z"),
                documentId = "doc_123",
                documentType = "article",
                domain = "example.com",
                excerpt = "Fallback excerpt",
                isFavorite = false,
                isShortlisted = true,
                language = "en",
                leadImageUrl = "https://example.com/lead.jpg",
                libraryEntryId = "lib_123",
                `object` = "library_entry",
                publishedAt = Instant.parse("2025-12-31T00:00:00Z"),
                readingTimeMinutes = 8,
                savedAt = Instant.parse("2026-01-02T00:00:00Z"),
                source = "web",
                sourceDeliveryId = "del_123",
                summary = "Resolved summary",
                thumbnailUrl = "https://example.com/thumb.jpg",
                title = "A mapped item",
                triageState = "inbox",
                updatedAt = Instant.parse("2026-01-03T00:00:00Z"),
                url = "https://example.com/item",
                wordCount = 1800,
            )

        val item = response.toLibraryItem()

        assertEquals("lib_123", item.id)
        assertEquals("doc_123", item.documentId)
        assertEquals("article", item.itemType)
        assertEquals("Resolved summary", item.summary)
        assertEquals(8, item.readingTimeMinutes)
        assertEquals(1800, item.wordCount)
        assertEquals("https://example.com/thumb.jpg", item.thumbnailUrl)
        assertEquals("https://example.com/lead.jpg", item.leadImageUrl)
        assertEquals(null, item.deletedAt)
    }
}
