package app.indelible.reader.repository

import app.indelible.core.offline.DL_DOC
import app.indelible.core.offline.DL_SCOPE
import app.indelible.core.offline.json
import app.indelible.core.offline.scopeDir
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

private const val TAGS_PATH = "/api/v1/tags"
private const val WHEN = "2026-01-01T00:00:00Z"

private fun tagJson(
    id: String,
    name: String,
) = """{"id":"$id","name":"$name","aliases":[],"created_at":"$WHEN","highlight_count":2,""" +
    """"item_count":1,"object":"tag"}"""

private val TAGS =
    """{"data":[${tagJson("tag_1", "research")},${tagJson("tag_2", "ideas")}],""" +
        """"page":{"has_more":false}}"""

class ApiReaderRepositoryTagCacheTest {
    @Test
    fun tags_seen_online_are_offered_offline() =
        runTest {
            val h = readerOfflineHarness { route(TAGS_PATH) { json(TAGS) } }
            h.repository.listTags().getOrThrow()
            h.goOffline()

            val tags = h.repository.listTags().getOrThrow()

            assertEquals(listOf("research", "ideas"), tags.map { it.name })
            assertEquals(listOf(2L, 2L), tags.map { it.highlightCount })
        }

    @Test
    fun names_tagged_offline_join_the_list_once() =
        runTest {
            val h = readerOfflineHarness { route(TAGS_PATH) { json(TAGS) } }
            h.keepCopy()
            h.repository.listTags().getOrThrow()
            h.goOffline()

            h.repository.setHighlightTags(DL_DOC, "hlt_1", listOf("research", "draft")).getOrThrow()
            val tags = h.repository.listTags().getOrThrow()

            assertEquals(listOf("research", "ideas", "draft"), tags.map { it.name })
            assertEquals(tags.size, tags.map { it.id }.toSet().size)
        }

    @Test
    fun names_tagged_offline_stay_offered_until_they_sync() =
        runTest {
            val h = readerOfflineHarness { route(TAGS_PATH) { json(TAGS) } }
            h.keepCopy()
            h.repository.listTags().getOrThrow()
            h.goOffline()
            h.repository.setHighlightTags(DL_DOC, "hlt_1", listOf("draft")).getOrThrow()
            h.server.route(TAGS_PATH) { json(TAGS) }

            assertEquals(listOf("research", "ideas", "draft"), h.tagNames())
            h.acknowledgeQueued()
            assertEquals(listOf("research", "ideas"), h.tagNames())
        }

    @Test
    fun a_list_answered_after_sign_out_is_not_kept() =
        runTest {
            val h = readerOfflineHarness { route(TAGS_PATH) { json(TAGS) } }
            h.duringFetchOf(TAGS_PATH) { h.signOut() }

            h.repository.listTags().getOrThrow()

            assertNull(h.keptTagsFile())
        }

    @Test
    fun a_name_kept_after_sign_out_is_not_written() =
        runTest {
            val h = readerOfflineHarness()
            val session = h.download.session
            h.signOut()

            h.offlineCopy.tags.keep(session, listOf("draft"))

            assertNull(h.keptTagsFile())
        }

    @Test
    fun without_a_kept_list_an_unreachable_server_is_a_failure() =
        runTest {
            val h = readerOfflineHarness { route(TAGS_PATH) { json(TAGS) } }
            h.goOffline()

            assertTrue(h.repository.listTags().isFailure)
        }

    @Test
    fun a_refusal_is_not_hidden_by_the_kept_list() =
        runTest {
            val h = readerOfflineHarness { route(TAGS_PATH) { json(TAGS) } }
            h.repository.listTags().getOrThrow()
            h.server.route(TAGS_PATH) { respond("no", HttpStatusCode.Forbidden) }

            assertTrue(h.repository.listTags().isFailure)
        }
}

private suspend fun ReaderOfflineHarness.tagNames() = repository.listTags().getOrThrow().map { it.name }

private fun ReaderOfflineHarness.keptTagsFile(): String? {
    val files = download.files
    return files.readText(files.scopeDir(DL_SCOPE) / "tags.json")
}
