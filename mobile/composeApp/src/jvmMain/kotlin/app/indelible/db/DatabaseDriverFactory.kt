package app.indelible.db

import app.cash.sqldelight.db.SqlDriver
import app.cash.sqldelight.driver.jdbc.sqlite.JdbcSqliteDriver
import java.io.File
import java.util.Properties

actual class DatabaseDriverFactory(
    private val directory: File = File(System.getProperty("user.home"), ".indelible"),
) {
    // One driver per factory: a recreated window must reuse this pool rather than open a second
    // one over the same file, which would defeat SqlDelightOfflineStore's per-instance write lock.
    private val lazyDriver =
        lazy {
            directory.mkdirs()
            // Handing the schema to the driver lets SQLDelight own user_version, so an existing
            // file is migrated instead of being treated as either new or already current.
            JdbcSqliteDriver(
                "jdbc:sqlite:${File(directory, DATABASE_NAME).absolutePath}",
                Properties(),
                OfflineDatabase.Schema,
            )
        }

    actual fun createDriver(): SqlDriver = lazyDriver.value

    fun close() {
        if (lazyDriver.isInitialized()) lazyDriver.value.close()
    }

    private companion object {
        const val DATABASE_NAME = "offline.db"
    }
}
