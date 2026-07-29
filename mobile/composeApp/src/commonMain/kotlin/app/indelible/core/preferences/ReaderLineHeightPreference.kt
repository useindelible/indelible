package app.indelible.core.preferences

enum class ReaderLineHeightPreference {
    COMPACT,
    RELAXED,
    ;

    val displayName: String
        get() =
            when (this) {
                COMPACT -> "Compact"
                RELAXED -> "Relaxed"
            }
}
