package app.indelible.db

import android.content.Context
import app.cash.sqldelight.db.SqlDriver
import app.cash.sqldelight.driver.android.AndroidSqliteDriver

actual class DatabaseDriverFactory(
    private val context: Context,
) {
    // One driver per factory, and the factory lives on the Application: SqlDelightOfflineStore
    // serializes its writes with a per-instance lock, so a second driver over the same file
    // would let two stores write concurrently and lose that serialization.
    private val driver: SqlDriver by lazy {
        AndroidSqliteDriver(OfflineDatabase.Schema, context, DATABASE_NAME)
    }

    actual fun createDriver(): SqlDriver = driver

    private companion object {
        const val DATABASE_NAME = "indelible_offline.db"
    }
}
