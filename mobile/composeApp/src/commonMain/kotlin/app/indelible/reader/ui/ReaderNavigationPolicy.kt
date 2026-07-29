package app.indelible.reader.ui

internal enum class ReaderNavigationDecision {
    Allow,
    AllowInitialDocument,
    OpenExternally,
    Cancel,
}

internal object ReaderNavigationPolicy {
    fun decide(
        isMainFrame: Boolean,
        scheme: String?,
        initialReaderDocumentPending: Boolean,
    ): ReaderNavigationDecision {
        if (!isMainFrame) return ReaderNavigationDecision.Allow

        val normalizedScheme = scheme?.lowercase()
        if (initialReaderDocumentPending && (normalizedScheme == null || normalizedScheme == "about")) {
            return ReaderNavigationDecision.AllowInitialDocument
        }

        return when (normalizedScheme) {
            "http", "https" -> ReaderNavigationDecision.OpenExternally
            else -> ReaderNavigationDecision.Cancel
        }
    }
}
