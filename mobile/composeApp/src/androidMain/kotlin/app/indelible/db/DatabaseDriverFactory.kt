package app.indelible.db

import android.content.Context
import app.cash.sqldelight.db.SqlDriver
import app.cash.sqldelight.driver.android.AndroidSqliteDriver

actual class DatabaseDriverFactory(
    private val context: Context,
) {
    // One driver per file: a second one would bypass SqlDelightOfflineStore's per-instance write lock.
    private val driver: SqlDriver by lazy {
        AndroidSqliteDriver(OfflineDatabase.Schema, context, DATABASE_NAME)
    }

    actual fun createDriver(): SqlDriver = driver

    private companion object {
        const val DATABASE_NAME = "indelible_offline.db"
    }
}
