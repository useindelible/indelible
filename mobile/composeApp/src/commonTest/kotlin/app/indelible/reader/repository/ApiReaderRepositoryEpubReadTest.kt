package app.indelible.reader.repository

import app.indelible.core.offline.CAPABILITIES
import app.indelible.core.offline.CapabilitiesUnknownException
import app.indelible.core.offline.ChapterListUnavailableException
import app.indelible.core.offline.DL_DOC
import app.indelible.core.offline.EPUB_TOC_PATH
import app.indelible.core.offline.FakeServer
import app.indelible.core.offline.chapterPath
import app.indelible.core.offline.epubToc
import app.indelible.core.offline.json
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

class ApiReaderRepositoryEpubReadTest {
    @Test
    fun cachedTocDoesNotCallTheServer() =
        runTest {
            val h = book()
            h.keepCopy()
            val before = h.server.count(EPUB_TOC_PATH)

            val toc = h.repository.getEpubToc(DL_DOC).getOrThrow()

            assertEquals(listOf("e0", "e1", "e2"), toc.toc.map { it.id })
            assertEquals(before, h.server.count(EPUB_TOC_PATH))
        }

    @Test
    fun cachedChapterDoesNotCallTheServer() =
        runTest {
            val h = book()
            h.keepCopy()

            val chapter = h.repository.getEpubChapter(DL_DOC, 1).getOrThrow()

            assertEquals("<h1>1</h1>", chapter)
            assertEquals(1, h.server.count(chapterPath(1)))
        }

    @Test
    fun chapterIsReadFromItsSpineFile() =
        runTest {
            val h = book { route(EPUB_TOC_PATH) { json(epubToc(listOf(1, 2))) } }
            h.keepCopy()
            h.goOffline()

            assertEquals("<h1>2</h1>", h.repository.getEpubChapter(DL_DOC, 2).getOrThrow())
            assertEquals("<h1>1</h1>", h.repository.getEpubChapter(DL_DOC, 1).getOrThrow())
        }

    @Test
    fun chapterFileThatCannotBeReadIsFetchedFromTheServer() =
        runTest {
            val h = book()
            h.keepCopy()
            val chapter = h.download.generationDir(1) / "chapters/1.html"
            h.download.fs.delete(chapter)
            h.download.fs.createDirectory(chapter)

            assertEquals("<h1>1</h1>", h.repository.getEpubChapter(DL_DOC, 1).getOrThrow())
            assertEquals(2, h.server.count(chapterPath(1)))
        }

    @Test
    fun missingChapterFileFetchesFromTheServer() =
        runTest {
            val h = book()
            h.keepCopy()
            h.download.fs.delete(h.download.generationDir(1) / "chapters/1.html")
            h.server.route(chapterPath(1)) { respond("<h1>fresh</h1>", HttpStatusCode.OK) }

            assertEquals("<h1>fresh</h1>", h.repository.getEpubChapter(DL_DOC, 1).getOrThrow())
        }

    @Test
    fun corruptTocIsFetchedFromTheServer() =
        runTest {
            val h = book()
            h.keepCopy()
            h.download.fs.write(h.download.generationDir(1) / "epub_toc.json") { writeUtf8("not json") }

            val toc = h.repository.getEpubToc(DL_DOC).getOrThrow()

            assertEquals(3, toc.toc.size)
            assertEquals(2, h.server.count(EPUB_TOC_PATH))
        }

    @Test
    fun uncachedChapterIsFetched() =
        runTest {
            val h = book()

            assertEquals("<h1>1</h1>", h.repository.getEpubChapter(DL_DOC, 1).getOrThrow())
            assertEquals(1, h.server.count(chapterPath(1)))
        }

    @Test
    fun unreadableChapterWithoutNetworkIsAFailure() =
        runTest {
            val h = book()
            h.keepCopy()
            h.download.fs.delete(h.download.generationDir(1) / "chapters/1.html")
            h.goOffline()

            assertTrue(h.repository.getEpubChapter(DL_DOC, 1).isFailure)
        }

    @Test
    fun copyOpensWithoutAskingTheServer() =
        runTest {
            val h = book()
            h.keepCopy()
            val asked = h.server.count(CAPABILITIES)

            assertTrue(h.repository.getEpubToc(DL_DOC).isSuccess)
            assertEquals(asked, h.server.count(CAPABILITIES))
        }

    @Test
    fun oldServerRefusesTheBook() =
        runTest {
            val h = book { route(CAPABILITIES) { json("""{"capabilities":[]}""") } }

            assertIs<ChapterListUnavailableException>(h.repository.getEpubToc(DL_DOC).exceptionOrNull())
            assertEquals(0, h.server.count(EPUB_TOC_PATH))
        }

    @Test
    fun unknownCapabilityIsAConnectionFailure() =
        runTest {
            val h = book { route(CAPABILITIES) { respond("", HttpStatusCode.ServiceUnavailable) } }

            assertIs<CapabilitiesUnknownException>(h.repository.getEpubToc(DL_DOC).exceptionOrNull())
        }

    private suspend fun TestScope.book(configure: FakeServer.() -> Unit = {}) =
        readerOfflineHarness {
            documentType = "book"
            available = listOf("epub")
            configure()
        }
}
