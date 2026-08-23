package app.indelible.core.preferences

import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.prefs_default_view_feed
import indelible.composeapp.generated.resources.prefs_default_view_library
import indelible.composeapp.generated.resources.prefs_default_view_search
import org.jetbrains.compose.resources.StringResource

enum class DefaultViewPreference {
    LIBRARY,
    FEED,
    SEARCH,
    ;

    val labelRes: StringResource
        get() =
            when (this) {
                LIBRARY -> Res.string.prefs_default_view_library
                FEED -> Res.string.prefs_default_view_feed
                SEARCH -> Res.string.prefs_default_view_search
            }
}
