package app.indelible.mila.data

sealed class ChatScope {
    data class SingleDocument(
        val documentId: String,
        val displayTitle: String? = null,
    ) : ChatScope()

    data class Collection(
        val collectionId: String,
        val displayTitle: String? = null,
    ) : ChatScope()

    data object CrossItem : ChatScope()
}

data class ChatMessage(
    val id: String,
    val role: String,
    val content: String,
    val sourceRefs: List<SourceRef> = emptyList(),
    val isStreaming: Boolean = false,
)

data class SourceRef(
    val documentId: String,
    val itemTitle: String,
)

sealed class StreamEvent {
    data class Delta(
        val text: String,
    ) : StreamEvent()

    data class Error(
        val message: String,
    ) : StreamEvent()

    data object Done : StreamEvent()
}
