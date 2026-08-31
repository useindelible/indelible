package app.indelible.db

import app.cash.sqldelight.db.SqlDriver
import app.cash.sqldelight.driver.jdbc.sqlite.JdbcSqliteDriver
import java.io.File

actual class DatabaseDriverFactory {
    actual fun createDriver(): SqlDriver {
        val directory = File(System.getProperty("user.home"), ".indelible")
        directory.mkdirs()
        val databaseFile = File(directory, "offline.db")
        val isNewDatabase = !databaseFile.exists()
        val driver = JdbcSqliteDriver("jdbc:sqlite:${databaseFile.absolutePath}")
        if (isNewDatabase) {
            OfflineDatabase.Schema.create(driver)
        }
        return driver
    }
}
