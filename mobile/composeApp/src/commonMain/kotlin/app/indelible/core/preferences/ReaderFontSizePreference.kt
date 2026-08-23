package app.indelible.core.preferences

import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.prefs_font_size_large
import indelible.composeapp.generated.resources.prefs_font_size_medium
import indelible.composeapp.generated.resources.prefs_font_size_small
import org.jetbrains.compose.resources.StringResource

enum class ReaderFontSizePreference {
    SMALL,
    MEDIUM,
    LARGE,
    ;

    val labelRes: StringResource
        get() =
            when (this) {
                SMALL -> Res.string.prefs_font_size_small
                MEDIUM -> Res.string.prefs_font_size_medium
                LARGE -> Res.string.prefs_font_size_large
            }
}
