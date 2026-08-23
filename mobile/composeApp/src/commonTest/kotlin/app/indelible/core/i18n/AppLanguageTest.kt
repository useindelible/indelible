package app.indelible.core.i18n

import kotlin.test.Test
import kotlin.test.assertEquals

class AppLanguageTest {
    @Test
    fun localeTagsResolveToSupportedLanguage() {
        assertEquals(AppLanguage.ENGLISH, AppLanguage.fromLanguageTag("en-US"))
        assertEquals(AppLanguage.FRENCH, AppLanguage.fromLanguageTag("fr-CA"))
        assertEquals(AppLanguage.ENGLISH, AppLanguage.fromLanguageTag("de-DE"))
    }
}
