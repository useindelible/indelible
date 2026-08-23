package app.indelible.core.preferences

import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.prefs_font_geist
import indelible.composeapp.generated.resources.prefs_font_geist_description
import indelible.composeapp.generated.resources.prefs_font_lora
import indelible.composeapp.generated.resources.prefs_font_lora_description
import indelible.composeapp.generated.resources.prefs_font_mono
import indelible.composeapp.generated.resources.prefs_font_mono_description
import org.jetbrains.compose.resources.StringResource

enum class ReaderFontFamilyPreference {
    SERIF,
    SANS,
    MONO,
    ;

    val labelRes: StringResource
        get() =
            when (this) {
                SERIF -> Res.string.prefs_font_lora
                SANS -> Res.string.prefs_font_geist
                MONO -> Res.string.prefs_font_mono
            }

    val descriptionRes: StringResource
        get() =
            when (this) {
                SERIF -> Res.string.prefs_font_lora_description
                SANS -> Res.string.prefs_font_geist_description
                MONO -> Res.string.prefs_font_mono_description
            }
}
