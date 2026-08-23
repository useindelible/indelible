package app.indelible

import android.app.Application
import app.indelible.core.i18n.LocaleFormatters

class IndelibleApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        LocaleFormatters.initialize(resources)
    }
}
