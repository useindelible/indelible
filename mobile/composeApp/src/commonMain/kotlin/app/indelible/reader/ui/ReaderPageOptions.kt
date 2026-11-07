package app.indelible.reader.ui

/** Everything a reader page carries besides its body, preferences and highlights. */
data class ReaderPageOptions(
    val localization: ReaderHtmlLocalization = ReaderHtmlLocalization.NONE,
    val article: ReaderArticleMeta = ReaderArticleMeta(),
    val summary: ReaderSummary = ReaderSummary.NONE,
    val topContentPaddingPx: Int = 0,
    val assets: ReaderAssets = ReaderAssets.NONE,
    val holdUntilPositioned: Boolean = false,
)

/** The masthead's words; a video passes none because its masthead is drawn natively. */
data class ReaderArticleMeta(
    val title: String = "",
    val author: String? = null,
    val domain: String? = null,
)

data class ReaderSummary(
    val html: String? = null,
    val points: List<String> = emptyList(),
) {
    companion object {
        val NONE = ReaderSummary()
    }
}
