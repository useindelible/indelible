package app.indelible.reader.ui

data class ReaderHtmlLocalization(
    val publishedDate: String?,
    val readingTime: String?,
    val summaryLabel: String,
    val askFollowUpLabel: String,
) {
    companion object {
        val NONE =
            ReaderHtmlLocalization(publishedDate = null, readingTime = null, summaryLabel = "", askFollowUpLabel = "")
    }
}
