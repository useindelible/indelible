package app.indelible.db

import app.cash.sqldelight.db.QueryResult
import app.cash.sqldelight.driver.jdbc.sqlite.JdbcSqliteDriver
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** The outbox as the first dev builds created it, before failed rows carried a typed failure. */
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

        OfflineDatabase.Schema.migrate(driver, 1, OfflineDatabase.Schema.version).value

        val columns =
            driver
                .executeQuery(
                    null,
                    "PRAGMA table_info(outbox)",
                    { cursor ->
                        val names = mutableListOf<String>()
                        while (cursor.next().value) names += checkNotNull(cursor.getString(1))
                        QueryResult.Value(names)
                    },
                    0,
                ).value
        assertTrue("failure_json" in columns, columns.toString())
        assertEquals(2L, OfflineDatabase.Schema.version)
    }
}
