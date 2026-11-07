package app.indelible.reader.model

/** Where a PDF stands between opening the item and showing its pages. */
enum class PdfStatus {
    LOADING,
    WAITING_FOR_COPY,
    READY,
    CONNECTION_FAILED,
    NO_SPACE,
    DOWNLOAD_FAILED,
    UNREADABLE,
    COPY_REMOVED,
}
