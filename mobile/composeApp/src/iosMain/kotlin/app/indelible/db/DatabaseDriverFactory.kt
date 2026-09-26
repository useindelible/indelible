package app.indelible.db

import app.cash.sqldelight.db.SqlDriver
import app.cash.sqldelight.driver.native.NativeSqliteDriver

actual class DatabaseDriverFactory {
    // One driver per factory, and the view controller remembers the factory: SqlDelightOfflineStore
    // serializes its writes with a per-instance lock, so a second driver over the same file
    // would let two stores write concurrently and lose that serialization.
    private val driver: SqlDriver by lazy {
        NativeSqliteDriver(OfflineDatabase.Schema, DATABASE_NAME)
    }

    actual fun createDriver(): SqlDriver = driver

    private companion object {
        const val DATABASE_NAME = "indelible_offline.db"
    }
}
