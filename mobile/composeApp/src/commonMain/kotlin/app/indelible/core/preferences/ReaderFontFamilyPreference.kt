package app.indelible.core.preferences

enum class ReaderFontFamilyPreference {
    SERIF,
    SANS,
    MONO,
    ;

    val displayName: String
        get() =
            when (this) {
                SERIF -> "Lora"
                SANS -> "Geist"
                MONO -> "Mono"
            }

    val description: String
        get() =
            when (this) {
                SERIF -> "Lora · Serif"
                SANS -> "Geist · Sans"
                MONO -> "JetBrains Mono · Mono"
            }
}
