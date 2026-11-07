package app.indelible

import android.app.Application
import app.indelible.core.i18n.LocaleFormatters
import app.indelible.db.DatabaseDriverFactory

class IndelibleApplication : Application() {
    /** Process-scoped so an Activity recreation reuses the open database instead of opening a second one. */
    val databaseDriverFactory: DatabaseDriverFactory by lazy { DatabaseDriverFactory(this) }

    override fun onCreate() {
        super.onCreate()
        LocaleFormatters.initialize(resources)
    }
}
