package app.indelible.db

import app.indelible.core.offline.CachedDocumentRow
import app.indelible.core.offline.SqlDelightOfflineStore
import kotlinx.coroutines.test.runTest
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals

class DatabaseDriverFactoryTest {
    @Test
    fun reopeningTheFileBackedDatabaseKeepsItsRows() =
        runTest {
            val directory = Files.createTempDirectory("indelible-offline").toFile()
            val scope = "scope"

            val first = DatabaseDriverFactory(directory)
            SqlDelightOfflineStore(OfflineDatabase(first.createDriver()))
                .upsertCachedDocument(scope, cachedDocument())
            first.close()

            val second = DatabaseDriverFactory(directory)
            val reopened = SqlDelightOfflineStore(OfflineDatabase(second.createDriver()))
            val row = reopened.cachedDocument(scope, "doc_1")
            second.close()

            assertEquals(cachedDocument(), row)
        }

    private fun cachedDocument() =
        CachedDocumentRow(
            "doc_1",
            "article",
            "Title",
            "{}",
            pinned = false,
            lastOpenedAt = 1L,
            lastSyncedAt = null,
            bytes = 5L,
        )
}
