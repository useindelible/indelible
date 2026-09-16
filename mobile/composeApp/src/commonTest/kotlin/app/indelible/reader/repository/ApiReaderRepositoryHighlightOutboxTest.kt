package app.indelible.reader.repository

import app.indelible.core.offline.OutboxKind
import app.indelible.core.offline.OutboxPayload
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class ApiReaderRepositoryHighlightOutboxTest {
    private fun cached(payloadJson: String): JsonObject = Json.parseToJsonElement(payloadJson).jsonObject

    @Test
    fun create_enqueues_highlight_create_and_caches_the_highlight_in_one_transaction() =
        runTest {
            val harness = readerOutboxHarness(backgroundScope)

            val created =
                harness.repository
                    .createHighlight(OFFLINE_DOCUMENT_ID, "Yellow", "quoted text", 12, 40)
                    .getOrThrow()

            assertTrue(created.id.startsWith("hlt_"))
            val row = harness.store.rowsOf(OFFLINE_SCOPE, OutboxKind.HIGHLIGHT_CREATE).single()
            assertEquals(created.id, row.entityId)
            assertEquals(OFFLINE_DOCUMENT_ID, row.documentId)
            val payload = assertIs<OutboxPayload.HighlightCreate>(row.payload)
            assertEquals(created.id, payload.highlightId)
            assertEquals("Yellow", payload.color)
            assertEquals("quoted text", payload.textContent)
            val locator = Json.parseToJsonElement(assertNotNull(payload.locatorJson)).jsonObject
            assertEquals("html", locator.getValue("type").jsonPrimitive.content)
            assertEquals(
                12,
                locator
                    .getValue("start_offset")
                    .jsonPrimitive.content
                    .toInt(),
            )
            assertEquals(
                40,
                locator
                    .getValue("end_offset")
                    .jsonPrimitive.content
                    .toInt(),
            )

            val cachedRow = assertNotNull(harness.store.probeCachedHighlight(OFFLINE_SCOPE, created.id))
            assertEquals(OFFLINE_DOCUMENT_ID, cachedRow.documentId)
            assertEquals("Yellow", cached(cachedRow.payloadJson).getValue("color").jsonPrimitive.content)
        }

    @Test
    fun create_triggers_a_drain() =
        runTest {
            val harness = readerOutboxHarness(backgroundScope)

            harness.repository.createHighlight(OFFLINE_DOCUMENT_ID, "Yellow", "quoted text", 0, 5)

            assertTrue(awaitStore { harness.sender.calls.takeIf { it > 0 } } > 0)
        }

    @Test
    fun color_change_rewrites_the_cached_color_in_the_same_transaction() =
        runTest {
            val harness = readerOutboxHarness(backgroundScope)
            val created =
                harness.repository
                    .createHighlight(OFFLINE_DOCUMENT_ID, "Yellow", "quoted text", 0, 5)
                    .getOrThrow()

            val updated = harness.repository.updateHighlightColor(OFFLINE_DOCUMENT_ID, created.id, "Blue").getOrThrow()

            assertEquals("Blue", updated.color)
            val row = harness.store.rowsOf(OFFLINE_SCOPE, OutboxKind.HIGHLIGHT_COLOR).single()
            assertEquals(OutboxPayload.HighlightColor(created.id, "Blue"), row.payload)
            val cachedRow = assertNotNull(harness.store.probeCachedHighlight(OFFLINE_SCOPE, created.id))
            val payload = cached(cachedRow.payloadJson)
            assertEquals("Blue", payload.getValue("color").jsonPrimitive.content)
            assertEquals("quoted text", payload.getValue("textContent").jsonPrimitive.content)
        }

    @Test
    fun color_change_on_an_uncached_highlight_enqueues_without_touching_the_cache() =
        runTest {
            val harness = readerOutboxHarness(backgroundScope)

            harness.repository.updateHighlightColor(OFFLINE_DOCUMENT_ID, "hlt_unknown", "Blue").getOrThrow()

            val row = harness.store.rowsOf(OFFLINE_SCOPE, OutboxKind.HIGHLIGHT_COLOR).single()
            assertEquals("hlt_unknown", row.entityId)
            assertEquals(OFFLINE_DOCUMENT_ID, row.documentId)
            assertNull(harness.store.probeCachedHighlight(OFFLINE_SCOPE, "hlt_unknown"))
        }

    @Test
    fun note_upsert_and_delete_rewrite_the_cached_note_body() =
        runTest {
            val harness = readerOutboxHarness(backgroundScope)
            val created =
                harness.repository
                    .createHighlight(OFFLINE_DOCUMENT_ID, "Yellow", "quoted text", 0, 5)
                    .getOrThrow()

            val note = harness.repository.upsertHighlightNote(OFFLINE_DOCUMENT_ID, created.id, "my note").getOrThrow()

            assertEquals("my note", note.body)
            assertEquals(created.id, note.highlightId)
            val withNote = assertNotNull(harness.store.probeCachedHighlight(OFFLINE_SCOPE, created.id))
            assertEquals(
                "my note",
                cached(withNote.payloadJson)
                    .getValue("note")
                    .jsonObject
                    .getValue("body")
                    .jsonPrimitive.content,
            )

            harness.repository.deleteHighlightNote(OFFLINE_DOCUMENT_ID, created.id).getOrThrow()

            val withoutNote = assertNotNull(harness.store.probeCachedHighlight(OFFLINE_SCOPE, created.id))
            assertNull(cached(withoutNote.payloadJson)["note"])
            assertEquals(
                listOf(
                    OutboxPayload.HighlightNote(created.id, "my note"),
                    OutboxPayload.HighlightNote(created.id, null),
                ),
                harness.store.rowsOf(OFFLINE_SCOPE, OutboxKind.HIGHLIGHT_NOTE).map { it.payload },
            )
        }

    @Test
    fun tag_change_rewrites_the_cached_tags() =
        runTest {
            val harness = readerOutboxHarness(backgroundScope)
            val created =
                harness.repository
                    .createHighlight(OFFLINE_DOCUMENT_ID, "Yellow", "quoted text", 0, 5)
                    .getOrThrow()

            val tags =
                harness.repository
                    .setHighlightTags(OFFLINE_DOCUMENT_ID, created.id, listOf("ideas", "later"))
                    .getOrThrow()

            assertEquals(listOf("ideas", "later"), tags)
            assertEquals(
                OutboxPayload.HighlightTags(created.id, listOf("ideas", "later")),
                harness.store
                    .rowsOf(OFFLINE_SCOPE, OutboxKind.HIGHLIGHT_TAGS)
                    .single()
                    .payload,
            )
            val cachedRow = assertNotNull(harness.store.probeCachedHighlight(OFFLINE_SCOPE, created.id))
            assertEquals(
                listOf("ideas", "later"),
                cached(cachedRow.payloadJson).getValue("tags").jsonArray.map { it.jsonPrimitive.content },
            )
        }

    @Test
    fun every_highlight_row_carries_the_document_it_was_written_in() =
        runTest {
            val harness = readerOutboxHarness(backgroundScope)
            val created =
                harness.repository
                    .createHighlight(OFFLINE_DOCUMENT_ID, "Yellow", "quoted text", 0, 5)
                    .getOrThrow()

            harness.repository.updateHighlightColor(OFFLINE_DOCUMENT_ID, created.id, "Blue").getOrThrow()
            harness.repository.upsertHighlightNote(OFFLINE_DOCUMENT_ID, created.id, "note").getOrThrow()
            harness.repository.deleteHighlightNote(OFFLINE_DOCUMENT_ID, created.id).getOrThrow()
            harness.repository.setHighlightTags(OFFLINE_DOCUMENT_ID, created.id, listOf("t")).getOrThrow()
            harness.repository.deleteHighlight(OFFLINE_DOCUMENT_ID, created.id).getOrThrow()
            harness.repository.updateHighlightColor(OFFLINE_DOCUMENT_ID, "hlt_uncached", "Blue").getOrThrow()

            val documents = harness.store.drainable(OFFLINE_SCOPE, Long.MAX_VALUE).map { it.documentId }
            assertEquals(List(documents.size) { OFFLINE_DOCUMENT_ID }, documents)
        }

    @Test
    fun delete_removes_the_cached_row_in_the_same_transaction() =
        runTest {
            val harness = readerOutboxHarness(backgroundScope)
            val created =
                harness.repository
                    .createHighlight(OFFLINE_DOCUMENT_ID, "Yellow", "quoted text", 0, 5)
                    .getOrThrow()

            harness.repository.deleteHighlight(OFFLINE_DOCUMENT_ID, created.id).getOrThrow()

            assertEquals(
                OutboxPayload.HighlightDelete(created.id),
                harness.store
                    .rowsOf(OFFLINE_SCOPE, OutboxKind.HIGHLIGHT_DELETE)
                    .single()
                    .payload,
            )
            assertNull(harness.store.probeCachedHighlight(OFFLINE_SCOPE, created.id))
        }
}
