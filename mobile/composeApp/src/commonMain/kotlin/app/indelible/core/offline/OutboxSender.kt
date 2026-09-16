package app.indelible.core.offline

/** A batch is more than one row only when every row in it is READING_EVENT for the same document. */
interface OutboxSender {
    suspend fun send(
        scope: String,
        batch: List<OutboxRow>,
    ): SendOutcome
}
