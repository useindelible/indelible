package app.indelible.reader.model

data class ReaderReprocessResult(
    val queued: Boolean,
    val retryAfterSeconds: Long?,
)
