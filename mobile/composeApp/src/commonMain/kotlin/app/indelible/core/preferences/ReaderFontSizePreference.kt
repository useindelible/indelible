package app.indelible.core.preferences

enum class ReaderFontSizePreference {
    SMALL,
    MEDIUM,
    LARGE,
    ;

    val displayName: String
        get() =
            when (this) {
                SMALL -> "Small"
                MEDIUM -> "Medium"
                LARGE -> "Large"
            }
}
