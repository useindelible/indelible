package app.indelible.core.preferences

import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.prefs_line_height_compact
import indelible.composeapp.generated.resources.prefs_line_height_relaxed
import org.jetbrains.compose.resources.StringResource

enum class ReaderLineHeightPreference {
    COMPACT,
    RELAXED,
    ;

    val labelRes: StringResource
        get() =
            when (this) {
                COMPACT -> Res.string.prefs_line_height_compact
                RELAXED -> Res.string.prefs_line_height_relaxed
            }
}
