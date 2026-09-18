package app.indelible.core.offline

import app.indelible.core.network.ApiException
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.job
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.runTest
import okio.IOException
import okio.Path
import okio.fakefilesystem.FakeFileSystem
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class DownloadSetsTest {
    @Test
    fun articleInstallsReaderHtmlTocHighlightsNoteInOneGeneration() =
        runTest {
            val h = downloadHarness()

            assertEquals(FetchResult.Installed, h.fetcher.download(h.session, DL_DOC, pin = true))

            val copy = checkNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertEquals(1L, copy.generation)
            assertTrue(copy.pinned)
            assertEquals("server note", copy.noteBody)
            assertEquals(20, copy.progressPercent)
            assertEquals(listOf("hlt_1"), h.store.cachedHighlights(DL_SCOPE, DL_DOC).map { it.id })
            val assets = h.store.assetsForDocument(DL_SCOPE, DL_DOC).associateBy { it.kind }
            assertEquals(setOf("readable_html", "article_toc"), assets.keys)
            assertEquals(12L, assets.getValue("readable_html").bytes)
            assertEquals("<p>hello</p>", h.files.readText(h.generationDir(1) / assets.getValue("readable_html").path))
            assertEquals(copy.bytes, assets.values.sumOf { it.bytes })
        }

    @Test
    fun pdfFollowsPresignedRedirectWithoutAuthorization() =
        runTest {
            val h =
                downloadHarness {
                    documentType = "pdf"
                    available = listOf("pdf")
                }

            h.fetcher.download(h.session, DL_DOC, pin = true)

            val asset = h.store.assetsForDocument(DL_SCOPE, DL_DOC).single()
            assertEquals("%PDF-1.7", h.files.readText(h.generationDir(1) / asset.path))
            val proxy = h.server.calls.single { it.path == "/api/v1/assets/documents/$DL_DOC/pdf" }
            val presigned = h.server.calls.single { it.host == "s3.test" }
            assertEquals("Bearer token", proxy.authorization)
            assertNull(presigned.authorization)
        }

    @Test
    fun bookInstallsTocAndEveryChapter() =
        runTest {
            val h =
                downloadHarness {
                    documentType = "book"
                    available = listOf("epub")
                }

            h.fetcher.download(h.session, DL_DOC, pin = true)

            val assets = h.store.assetsForDocument(DL_SCOPE, DL_DOC)
            assertEquals(listOf("epub_toc"), assets.filter { it.kind == "epub_toc" }.map { it.kind })
            assertEquals(listOf(0, 1, 2), assets.filter { it.kind == "epub_chapter" }.map { it.idx }.sorted())
            for (index in 0..2) {
                assertEquals(1, h.server.count("/api/v1/documents/$DL_DOC/epub/chapters/$index"))
                val chapter = assets.single { it.kind == "epub_chapter" && it.idx == index }
                assertEquals("<h1>$index</h1>", h.files.readText(h.generationDir(1) / chapter.path))
            }
        }

    @Test
    fun failedChapterInstallsNothingAndDeletesGeneration() =
        runTest {
            val h =
                downloadHarness {
                    documentType = "book"
                    available = listOf("epub")
                    route("/api/v1/documents/$DL_DOC/epub/chapters/1") {
                        respond("boom", HttpStatusCode.InternalServerError)
                    }
                }

            val failure = assertFailsWith<ApiException> { h.fetcher.download(h.session, DL_DOC, pin = true) }

            assertEquals(500, failure.statusCode)
            assertNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertFalse(h.fs.exists(h.generationDir(1)))
        }

    @Test
    fun htmlSkippedWhenNotInAvailableAssets() =
        runTest {
            val h = downloadHarness { available = emptyList() }

            assertEquals(FetchResult.Skipped, h.fetcher.download(h.session, DL_DOC, pin = false))
            assertNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertEquals(FetchResult.Installed, h.fetcher.download(h.session, DL_DOC, pin = true))

            assertEquals(emptyList(), h.store.assetsForDocument(DL_SCOPE, DL_DOC))
            assertEquals(0, h.server.count("/api/v1/assets/documents/$DL_DOC/readable_html"))
            assertEquals("server note", h.store.cachedDocument(DL_SCOPE, DL_DOC)?.noteBody)
        }

    @Test
    fun staleFirstInstallIsRefetchedThenSucceeds() =
        runTest {
            lateinit var h: DownloadHarness
            var moved = false
            h =
                downloadHarness {
                    route("/api/v1/documents/$DL_DOC/highlights") {
                        if (!moved) {
                            moved = true
                            h.signedIn.run { store.remove(scope, enqueue(OutboxPayload.DocumentNote("mine", null))) }
                        }
                        json("""{"count":0,"highlights":[]}""")
                    }
                }

            assertEquals(FetchResult.Installed, h.fetcher.download(h.session, DL_DOC, pin = true))

            assertEquals(2, h.server.count("/api/v1/documents/$DL_DOC/highlights"))
            assertEquals(1, h.server.count("/api/v1/assets/documents/$DL_DOC/readable_html"))
            assertEquals(1L, h.store.cachedDocument(DL_SCOPE, DL_DOC)?.generation)
        }

    @Test
    fun cancellationAfterCommitKeepsTheInstall() =
        runTest {
            val h =
                downloadHarness(
                    store = { real ->
                        object : OfflineStore by real {
                            override suspend fun installCachedDocument(
                                session: Session,
                                request: InstallRequest,
                            ): InstallResult {
                                val result = real.installCachedDocument(session, request)
                                currentCoroutineContext().job.cancel()
                                currentCoroutineContext().ensureActive()
                                return result
                            }
                        }
                    },
                )

            val download = launch { h.fetcher.download(h.session, DL_DOC, pin = true) }
            download.join()

            assertTrue(download.isCancelled)
            assertEquals(1L, h.store.cachedDocument(DL_SCOPE, DL_DOC)?.generation)
            assertTrue(h.fs.exists(h.generationDir(1)))
        }

    @Test
    fun oldGenerationDeleteFailureLeavesNewInstall() =
        runTest {
            val fs = FailingDeleteFileSystem(FakeFileSystem())
            val h = downloadHarness(fs = fs)
            h.fetcher.download(h.session, DL_DOC, pin = true)
            fs.failUnder = h.generationDir(1)

            assertEquals(FetchResult.Installed, h.fetcher.download(h.session, DL_DOC, pin = false))

            val copy = checkNotNull(h.store.cachedDocument(DL_SCOPE, DL_DOC))
            assertEquals(2L, copy.generation)
            assertTrue(copy.pinned)
            assertTrue(fs.exists(h.generationDir(2)))
        }
}

/** Refuses to delete anything under [failUnder], the way a locked or read-only file would. */
internal class FailingDeleteFileSystem(
    delegate: okio.FileSystem,
) : okio.ForwardingFileSystem(delegate) {
    var failUnder: Path? = null

    override fun delete(
        path: Path,
        mustExist: Boolean,
    ) {
        val blocked = failUnder
        if (blocked != null && (path == blocked || path.toString().startsWith("$blocked/"))) {
            throw IOException("delete refused")
        }
        super.delete(path, mustExist)
    }
}
