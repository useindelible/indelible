package app.indelible.core.i18n

import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.prefs_language_english
import indelible.composeapp.generated.resources.prefs_language_french
import indelible.composeapp.generated.resources.prefs_language_system_default
import org.jetbrains.compose.resources.StringResource

enum class AppLanguage(
    val languageTag: String?,
    val labelRes: StringResource,
) {
    SYSTEM_DEFAULT(null, Res.string.prefs_language_system_default),
    ENGLISH("en", Res.string.prefs_language_english),
    FRENCH("fr", Res.string.prefs_language_french),
    ;

    companion object {
        fun fromLanguageTag(languageTag: String): AppLanguage =
            when (languageTag.substringBefore('-').substringBefore('_').lowercase()) {
                "fr" -> FRENCH
                else -> ENGLISH
            }
    }
}

sealed interface AppLanguageSettings {
    val language: AppLanguage

    data class Selectable(
        override val language: AppLanguage,
        val onSelected: (AppLanguage) -> Unit,
    ) : AppLanguageSettings

    data class SystemManaged(
        override val language: AppLanguage,
        val onOpenSettings: () -> Unit,
    ) : AppLanguageSettings
}
