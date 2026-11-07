package app.indelible.db

import app.cash.sqldelight.db.SqlDriver
import app.cash.sqldelight.driver.native.NativeSqliteDriver

actual class DatabaseDriverFactory {
    // One driver per file: a second one would bypass SqlDelightOfflineStore's per-instance write lock.
    private val driver: SqlDriver by lazy {
        NativeSqliteDriver(OfflineDatabase.Schema, DATABASE_NAME)
    }

    actual fun createDriver(): SqlDriver = driver

    private companion object {
        const val DATABASE_NAME = "indelible_offline.db"
    }
}
