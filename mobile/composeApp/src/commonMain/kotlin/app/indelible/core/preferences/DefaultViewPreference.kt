package app.indelible.core.preferences

enum class DefaultViewPreference {
    LIBRARY,
    FEED,
    SEARCH,
    ;

    val displayName: String
        get() =
            when (this) {
                LIBRARY -> "Library"
                FEED -> "Feed"
                SEARCH -> "Search"
            }
}
