package app.indelible.db

import app.cash.sqldelight.driver.jdbc.sqlite.JdbcSqliteDriver

actual fun testOfflineDatabase(): OfflineDatabase {
    val driver = JdbcSqliteDriver(JdbcSqliteDriver.IN_MEMORY)
    OfflineDatabase.Schema.create(driver)
    return OfflineDatabase(driver)
}
