package app.indelible.core.preferences

enum class ThemePreference {
    AUTO,
    LIGHT,
    DARK,
    ;

    val displayName: String
        get() =
            when (this) {
                AUTO -> "Auto"
                LIGHT -> "Light"
                DARK -> "Dark"
            }
}
