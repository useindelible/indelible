package app.indelible.core.preferences

enum class TriageModePreference {
    MANUAL,
    FOCUS,
    ;

    val displayName: String
        get() =
            when (this) {
                MANUAL -> "Manual"
                FOCUS -> "Focus"
            }
}
