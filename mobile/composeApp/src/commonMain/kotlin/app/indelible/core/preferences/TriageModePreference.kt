package app.indelible.core.preferences

import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.prefs_triage_mode_focus
import indelible.composeapp.generated.resources.prefs_triage_mode_manual
import org.jetbrains.compose.resources.StringResource

enum class TriageModePreference {
    MANUAL,
    FOCUS,
    ;

    val labelRes: StringResource
        get() =
            when (this) {
                MANUAL -> Res.string.prefs_triage_mode_manual
                FOCUS -> Res.string.prefs_triage_mode_focus
            }
}
