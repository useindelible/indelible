package app.indelible.core.preferences

import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.prefs_theme_auto
import indelible.composeapp.generated.resources.prefs_theme_dark
import indelible.composeapp.generated.resources.prefs_theme_light
import org.jetbrains.compose.resources.StringResource

enum class ThemePreference {
    AUTO,
    LIGHT,
    DARK,
    ;

    val labelRes: StringResource
        get() =
            when (this) {
                AUTO -> Res.string.prefs_theme_auto
                LIGHT -> Res.string.prefs_theme_light
                DARK -> Res.string.prefs_theme_dark
            }
}
