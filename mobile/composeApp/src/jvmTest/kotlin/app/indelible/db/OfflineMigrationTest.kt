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
        assertEquals(2L, OfflineDatabase.Schema.version)
    }

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
