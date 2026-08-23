package app.indelible.core.i18n

import android.app.LocaleManager
import android.content.Context
import android.os.LocaleList
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.SdkSuppress
import androidx.test.platform.app.InstrumentationRegistry
import kotlinx.datetime.Instant
import org.junit.Test
import org.junit.runner.RunWith
import java.util.Locale
import kotlin.test.assertEquals

@RunWith(AndroidJUnit4::class)
class LocaleFormattersAndroidTest {
    @Test
    @SdkSuppress(minSdkVersion = 33)
    fun formattingUsesTheApplicationResourceLocale() {
        val applicationContext = ApplicationProvider.getApplicationContext<Context>()
        val localeManager = applicationContext.getSystemService(LocaleManager::class.java)
        val previousApplicationLocales = localeManager.applicationLocales
        val previousLocale = Locale.getDefault()

        try {
            localeManager.applicationLocales = LocaleList.forLanguageTags("fr")
            InstrumentationRegistry.getInstrumentation().waitForIdleSync()
            Locale.setDefault(Locale.US)
            LocaleFormatters.initialize(applicationContext.resources)

            assertEquals(
                "fr",
                applicationContext.resources.configuration.locales[0]
                    .language,
            )
            assertEquals("1\u202f234", LocaleFormatters.number(1234))
            assertEquals(
                "2 janv. 2025",
                LocaleFormatters.date(Instant.parse("2025-01-02T12:00:00Z"), LocalizedDateStyle.MEDIUM),
            )
        } finally {
            localeManager.applicationLocales = previousApplicationLocales
            Locale.setDefault(previousLocale)
            LocaleFormatters.initialize(applicationContext.resources)
        }
    }
}
