package app.indelible.mila.viewmodel

import app.indelible.api.generated.models.MilaConfigResponse
import app.indelible.api.generated.models.MilaConversationResponse
import app.indelible.api.generated.models.MilaMessageResponse
import app.indelible.api.generated.models.MilaSessionListResponse
import app.indelible.api.generated.models.MilaSessionPreviewResponse
import app.indelible.api.generated.models.MilaSessionResponse
import app.indelible.api.generated.models.MilaSourceRef
import app.indelible.core.network.ApiClient
import app.indelible.core.storage.InMemoryTokenStorage
import app.indelible.mila.data.MilaRepository
import app.indelible.mila.data.StreamEvent
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.datetime.Instant

class FakeMilaRepository :
    MilaRepository(
        ApiClient(InMemoryTokenStorage()).milaApiService,
    ) {
    var configResult: Result<MilaConfigResponse> = Result.success(enabledConfig())
    var listSessionsResult: Result<MilaSessionListResponse> = Result.success(emptySessions())
    var createSessionResult: Result<MilaSessionResponse> = Result.success(fakeSession())
    var getMessagesResult: Result<MilaConversationResponse> = Result.success(emptyConversation())
    var streamEvents: List<StreamEvent> =
        listOf(
            StreamEvent.Delta("Hello"),
            StreamEvent.Delta(" world"),
            StreamEvent.Done,
        )

    /** Thrown after [streamEvents] to simulate a mid-stream network failure. */
    var streamError: Throwable? = null

    var lastCreateSessionType: String? = null
    var lastCreateDocumentId: String? = null
    var lastCreateCollectionId: String? = null
    var lastStreamSessionId: String? = null
    var lastStreamQuestion: String? = null

    override suspend fun getMilaConfig(): Result<MilaConfigResponse> = configResult

    override suspend fun listSessions(limit: Int): Result<MilaSessionListResponse> = listSessionsResult

    override suspend fun createSession(
        sessionType: String,
        documentId: String?,
        collectionId: String?,
    ): Result<MilaSessionResponse> {
        lastCreateSessionType = sessionType
        lastCreateDocumentId = documentId
        lastCreateCollectionId = collectionId
        return createSessionResult
    }

    override suspend fun getMessages(sessionId: String): Result<MilaConversationResponse> = getMessagesResult

    override fun streamChat(
        sessionId: String,
        question: String,
    ): Flow<StreamEvent> {
        lastStreamSessionId = sessionId
        lastStreamQuestion = question
        return flow {
            streamEvents.forEach { emit(it) }
            streamError?.let { throw it }
        }
    }

    companion object {
        private val fixedInstant = Instant.parse("2024-01-01T00:00:00Z")

        fun enabledConfig() =
            MilaConfigResponse(
                byoEnabled = true,
                chatApiBase = "http://localhost:11434/v1",
                chatContextPct = 70,
                chatModel = "llama3.2",
                crossItemMaxPerItem = 3,
                crossItemTopK = 20,
                embeddingApiBase = "http://localhost:11434/v1",
                embeddingDim = 768,
                embeddingModel = "nomic-embed-text",
                enabled = true,
                hasChatApiKey = true,
                hasEmbeddingApiKey = true,
                modelContextWindow = 16_000,
                supportsReasoningEffort = true,
                supportsStructuredOutput = true,
                topK = 6,
            )

        fun disabledConfig() = enabledConfig().copy(enabled = false)

        fun emptySessions() = MilaSessionListResponse(sessions = emptyList())

        fun fakeSession(
            id: String = "session_01",
            sessionType: String = "single_document",
            documentId: String? = "doc_01",
            collectionId: String? = null,
        ) = MilaSessionResponse(
            id = id,
            sessionType = sessionType,
            documentId = documentId,
            collectionId = collectionId,
            createdAt = fixedInstant,
            lastActive = fixedInstant,
        )

        fun fakeSessionPreview(
            id: String = "session_01",
            sessionType: String = "single_document",
            documentId: String? = "doc_01",
            collectionId: String? = null,
        ) = MilaSessionPreviewResponse(
            id = id,
            sessionType = sessionType,
            documentId = documentId,
            collectionId = collectionId,
            createdAt = fixedInstant,
            lastActive = fixedInstant,
        )

        fun emptyConversation() =
            MilaConversationResponse(
                session = fakeSession(),
                messages = emptyList(),
            )

        fun conversationWithMessages(
            userContent: String = "What is this about?",
            assistantContent: String = "This article discusses testing patterns.",
            sourceRefs: List<MilaSourceRef> =
                listOf(
                    MilaSourceRef(
                        documentId = "doc_01",
                        itemTitle = "Testing Best Practices",
                        sourceLabel = "example.com",
                    ),
                ),
        ) = MilaConversationResponse(
            session = fakeSession(),
            messages =
                listOf(
                    MilaMessageResponse(
                        id = "msg_01",
                        role = "user",
                        content = userContent,
                        sourceRefs = emptyList(),
                        createdAt = fixedInstant,
                    ),
                    MilaMessageResponse(
                        id = "msg_02",
                        role = "assistant",
                        content = assistantContent,
                        sourceRefs = sourceRefs,
                        createdAt = fixedInstant,
                    ),
                ),
        )
    }
}
