package app.indelible.db

import app.cash.sqldelight.db.QueryResult
import app.cash.sqldelight.driver.jdbc.sqlite.JdbcSqliteDriver
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** The outbox as the first dev builds created it, before failed rows carried a typed failure. */
private val V1_CACHED_DOCUMENT =
    """
    CREATE TABLE cached_document (
        scope TEXT NOT NULL,
        document_id TEXT NOT NULL,
        document_type TEXT NOT NULL,
        title TEXT NOT NULL,
        reader_json TEXT NOT NULL,
        pinned INTEGER NOT NULL DEFAULT 0,
        last_opened_at INTEGER NOT NULL,
        last_synced_at INTEGER,
        bytes INTEGER NOT NULL DEFAULT 0,
        generation INTEGER NOT NULL DEFAULT 0,
        note_body TEXT,
        note_server_updated_at INTEGER,
        progress_percent INTEGER,
        max_progress_percent INTEGER,
        chapter_locator TEXT,
        chapter_offset INTEGER,
        PRIMARY KEY (scope, document_id)
    )
    """.trimIndent()

private val V1_OUTBOX =
    """
    CREATE TABLE outbox (
        seq INTEGER PRIMARY KEY AUTOINCREMENT,
        scope TEXT NOT NULL,
        id TEXT NOT NULL,
        kind TEXT NOT NULL,
        entity_id TEXT NOT NULL,
        document_id TEXT NOT NULL,
        payload_json TEXT NOT NULL,
        created_at INTEGER NOT NULL,
        attempts INTEGER NOT NULL DEFAULT 0,
        last_attempt_at INTEGER,
        next_attempt_at INTEGER NOT NULL DEFAULT 0,
        state TEXT NOT NULL DEFAULT 'pending',
        last_error TEXT,
        superseded INTEGER NOT NULL DEFAULT 0
    )
    """.trimIndent()

private val LIST_SNAPSHOT_COLUMNS =
    listOf("scope", "list_key", "position", "item_id", "document_id", "document_type", "title", "row_json", "seen_at")

class OfflineMigrationTest {
    @Test
    fun aVersionOneOutboxGainsTheFailureColumn() {
        val driver = JdbcSqliteDriver(JdbcSqliteDriver.IN_MEMORY)
        driver.execute(null, V1_OUTBOX, 0)
        driver.execute(null, V1_CACHED_DOCUMENT, 0)

        OfflineDatabase.Schema.migrate(driver, 1, OfflineDatabase.Schema.version).value

        val columns = columnsOf(driver, "outbox")
        assertTrue("failure_json" in columns, columns.toString())
        assertTrue("note_server_body" in columnsOf(driver, "cached_document"))
        assertEquals(4L, OfflineDatabase.Schema.version)
    }

    @Test
    fun aCopyCreatedBeforeListSnapshotsGainsTheTable() {
        val driver = versionThreeWithoutListSnapshots()

        OfflineDatabase.Schema.migrate(driver, 3, OfflineDatabase.Schema.version).value

        assertEquals(LIST_SNAPSHOT_COLUMNS, columnsOf(driver, "list_snapshot"))
    }

    @Test
    fun aCopyThatHasListSnapshotsKeepsItsRows() {
        val driver = versionThreeWithoutListSnapshots()
        driver.execute(null, "CREATE TABLE list_snapshot (${LIST_SNAPSHOT_COLUMNS.joinToString()})", 0)
        driver.execute(null, "INSERT INTO list_snapshot (scope) VALUES ('s')", 0)

        OfflineDatabase.Schema.migrate(driver, 3, OfflineDatabase.Schema.version).value

        assertEquals(1L, countOf(driver, "list_snapshot"))
    }

    @Test
    fun aVersionTwoCopyKeepsItsNoteAsTheServersUnlessAnEditIsQueued() {
        val driver = JdbcSqliteDriver(JdbcSqliteDriver.IN_MEMORY)
        driver.execute(null, V1_OUTBOX, 0)
        driver.execute(null, V1_CACHED_DOCUMENT, 0)
        driver.execute(null, "ALTER TABLE outbox ADD COLUMN failure_json TEXT", 0)
        driver.execute(null, cachedDocument("doc_synced", "server words"), 0)
        driver.execute(null, cachedDocument("doc_edited", "local words"), 0)
        driver.execute(null, queuedNote("doc_edited"), 0)

        OfflineDatabase.Schema.migrate(driver, 2, OfflineDatabase.Schema.version).value

        assertEquals(mapOf("doc_synced" to "server words", "doc_edited" to null), serverNotes(driver))
    }

    private fun versionThreeWithoutListSnapshots(): JdbcSqliteDriver {
        val driver = JdbcSqliteDriver(JdbcSqliteDriver.IN_MEMORY)
        driver.execute(null, V1_OUTBOX, 0)
        driver.execute(null, V1_CACHED_DOCUMENT, 0)
        driver.execute(null, "ALTER TABLE outbox ADD COLUMN failure_json TEXT", 0)
        driver.execute(null, "ALTER TABLE cached_document ADD COLUMN note_server_body TEXT", 0)
        return driver
    }

    private fun countOf(
        driver: JdbcSqliteDriver,
        table: String,
    ): Long =
        driver
            .executeQuery(
                null,
                "SELECT count(*) FROM $table",
                { cursor -> QueryResult.Value(if (cursor.next().value) cursor.getLong(0) else null) },
                0,
            ).value ?: 0L

    private fun cachedDocument(
        id: String,
        note: String,
    ): String =
        "INSERT INTO cached_document " +
            "(scope, document_id, document_type, title, reader_json, last_opened_at, note_body) " +
            "VALUES ('s', '$id', 'article', 't', '{}', 0, '$note')"

    private fun queuedNote(documentId: String): String =
        "INSERT INTO outbox (scope, id, kind, entity_id, document_id, payload_json, created_at) " +
            "VALUES ('s', 'obx_1', 'document_note', '$documentId', '$documentId', '{}', 0)"

    private fun serverNotes(driver: JdbcSqliteDriver): Map<String, String?> =
        driver
            .executeQuery(
                null,
                "SELECT document_id, note_server_body FROM cached_document ORDER BY document_id",
                { cursor ->
                    val notes = mutableMapOf<String, String?>()
                    while (cursor.next().value) notes[checkNotNull(cursor.getString(0))] = cursor.getString(1)
                    QueryResult.Value(notes)
                },
                0,
            ).value

    private fun columnsOf(
        driver: JdbcSqliteDriver,
        table: String,
    ): List<String> =
        driver
            .executeQuery(
                null,
                "PRAGMA table_info($table)",
                { cursor ->
                    val names = mutableListOf<String>()
                    while (cursor.next().value) names += checkNotNull(cursor.getString(1))
                    QueryResult.Value(names)
                },
                0,
            ).value
}
