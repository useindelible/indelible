package app.indelible.core.offline

import io.ktor.utils.io.ByteReadChannel
import kotlinx.coroutines.test.runTest
import okio.FileSystem
import okio.ForwardingFileSystem
import okio.IOException
import okio.Path
import okio.Path.Companion.toPath
import okio.Sink
import okio.fakefilesystem.FakeFileSystem
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertNull
import kotlin.test.assertSame
import kotlin.test.assertTrue

private val ROOT = "/offline".toPath()

/** Fails every file write with [failure], the way a full disk surfaces through a sink. */
internal class FailingWriteFileSystem(
    delegate: FileSystem,
    private val failure: () -> IOException,
) : ForwardingFileSystem(delegate) {
    override fun sink(
        file: Path,
        mustCreate: Boolean,
    ): Sink = throw failure()
}

class OfflineFilesTest {
    private val fs = FakeFileSystem()
    private val files = OfflineFiles(ROOT, fs)

    @Test
    fun writeReportsBytesAndReadsBack() =
        runTest {
            val path = ROOT / "scope" / "doc" / "1" / "readable.html"

            val written = files.write(path, ByteReadChannel("<p>hello</p>".encodeToByteArray()))

            assertEquals(12L, written)
            assertEquals("<p>hello</p>", files.readText(path))
            assertEquals(12L, files.size(path))
            assertNull(files.readText(ROOT / "missing"))
            assertNull(files.size(ROOT / "missing"))
        }

    @Test
    fun deleteTreeIsRecursiveAndIdempotent() =
        runTest {
            val doc = ROOT / "scope" / "doc"
            files.write(doc / "1" / "a.html", ByteReadChannel("a".encodeToByteArray()))
            files.write(doc / "1" / "chapters" / "b.html", ByteReadChannel("b".encodeToByteArray()))

            files.deleteTree(doc)
            files.deleteTree(doc)

            assertFalse(fs.exists(doc))
            assertTrue(fs.exists(ROOT / "scope"))
        }

    @Test
    fun listDirectoriesSkipsFiles() =
        runTest {
            val scope = ROOT / "scope"
            files.write(scope / "doc_a" / "1" / "a.html", ByteReadChannel("a".encodeToByteArray()))
            files.write(scope / "doc_b" / "2" / "b.html", ByteReadChannel("b".encodeToByteArray()))
            files.write(scope / "stray.tmp", ByteReadChannel("x".encodeToByteArray()))

            assertEquals(listOf("doc_a", "doc_b"), files.listDirectories(scope).map { it.name }.sorted())
            assertEquals(emptyList(), files.listDirectories(ROOT / "absent"))
        }

    @Test
    fun sizeOfTreeSumsNestedFiles() =
        runTest {
            val doc = ROOT / "scope" / "doc"
            files.write(doc / "1" / "a.html", ByteReadChannel(ByteArray(10)))
            files.write(doc / "1" / "chapters" / "b.html", ByteReadChannel(ByteArray(32)))

            assertEquals(42L, files.sizeOfTree(doc))
            assertEquals(0L, files.sizeOfTree(ROOT / "absent"))
        }

    @Test
    fun scopeDirNameIsHexSha256() {
        assertEquals("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad", scopeDirName("abc"))
        assertEquals(ROOT / scopeDirName("s") / "doc" / "7", files.generationDir("s", "doc", 7))
    }

    @Test
    fun noSpaceIsRecognisedInCauseChain() =
        runTest {
            val diskFull = IOException("write failed", IOException("No space left on device"))
            val full = OfflineFiles(ROOT, FailingWriteFileSystem(fs) { diskFull })
            val broken = IOException("disk unplugged")
            val failing = OfflineFiles(ROOT, FailingWriteFileSystem(fs) { broken })
            val bytes = "a".encodeToByteArray()

            val noSpace = assertFailsWith<NoSpaceException> { full.write(ROOT / "a", ByteReadChannel(bytes)) }
            val other = assertFailsWith<IOException> { failing.write(ROOT / "b", ByteReadChannel(bytes)) }

            assertIs<IOException>(noSpace.cause)
            assertSame(broken, other)
            assertTrue(IOException("write failed: ENOSPC").isNoSpace())
        }
}
