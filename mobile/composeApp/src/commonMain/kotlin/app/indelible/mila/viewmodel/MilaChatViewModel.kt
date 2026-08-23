package app.indelible.mila.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.core.i18n.UiMessage
import app.indelible.mila.data.ChatMessage
import app.indelible.mila.data.ChatScope
import app.indelible.mila.data.MilaRepository
import app.indelible.mila.data.SourceRef
import app.indelible.mila.data.StreamEvent
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.mila_chat_failed
import indelible.composeapp.generated.resources.mila_chat_load_failed
import indelible.composeapp.generated.resources.mila_chat_provider_unavailable
import indelible.composeapp.generated.resources.mila_chat_session_failed
import indelible.composeapp.generated.resources.mila_chat_timeout
import io.ktor.util.date.getTimeMillis
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.launch

class MilaChatViewModel(
    private val repository: MilaRepository,
    val scope: ChatScope,
) : ViewModel() {
    private val _uiState = MutableStateFlow(MilaChatUiState())
    val uiState: StateFlow<MilaChatUiState> = _uiState.asStateFlow()

    private val _effects = MutableSharedFlow<MilaChatEffect>()
    val effects: SharedFlow<MilaChatEffect> = _effects.asSharedFlow()

    private var sessionId: String? = null
    private var lastQuestion: String? = null

    init {
        viewModelScope.launch { checkConfigAndInit() }
    }

    private suspend fun checkConfigAndInit() {
        repository
            .getMilaConfig()
            .onSuccess { config ->
                if (!config.enabled) {
                    _uiState.value =
                        _uiState.value.copy(
                            configLoading = false,
                            milaEnabled = false,
                        )
                    return
                }
                _uiState.value =
                    _uiState.value.copy(
                        configLoading = false,
                        milaEnabled = true,
                    )
                resumeOrCreateSession()
            }.onFailure {
                _uiState.value =
                    _uiState.value.copy(
                        configLoading = false,
                        milaEnabled = false,
                    )
            }
    }

    private suspend fun resumeOrCreateSession() {
        _uiState.value = _uiState.value.copy(sessionLoading = true)

        repository
            .listSessions()
            .onSuccess { sessionList ->
                val match =
                    sessionList.sessions.find { session ->
                        when (scope) {
                            is ChatScope.SingleDocument ->
                                session.sessionType == "single_document" && session.documentId == scope.documentId

                            is ChatScope.Collection ->
                                session.sessionType == "collection" && session.collectionId == scope.collectionId

                            is ChatScope.CrossItem ->
                                session.sessionType == "cross_item"
                        }
                    }

                if (match != null) {
                    sessionId = match.id
                    loadMessages(match.id)
                } else {
                    createNewSession()
                }
            }.onFailure {
                createNewSession()
            }
    }

    private suspend fun loadMessages(sessionId: String) {
        repository
            .getMessages(sessionId)
            .onSuccess { conversation ->
                _uiState.value =
                    _uiState.value.copy(
                        sessionLoading = false,
                        messages =
                            conversation.messages.map { msg ->
                                ChatMessage(
                                    id = msg.id,
                                    role = msg.role,
                                    content = msg.content,
                                    sourceRefs =
                                        msg.sourceRefs.map { ref ->
                                            SourceRef(
                                                documentId = ref.documentId,
                                                itemTitle = ref.itemTitle,
                                            )
                                        },
                                )
                            },
                    )
            }.onFailure {
                _uiState.value =
                    _uiState.value.copy(
                        sessionLoading = false,
                        error = UiMessage(Res.string.mila_chat_load_failed),
                    )
            }
    }

    private suspend fun createNewSession() {
        val (sessionType, documentId, collectionId) =
            when (scope) {
                is ChatScope.SingleDocument -> Triple("single_document", scope.documentId, null)
                is ChatScope.Collection -> Triple("collection", null, scope.collectionId)
                is ChatScope.CrossItem -> Triple("cross_item", null, null)
            }

        repository
            .createSession(sessionType, documentId, collectionId)
            .onSuccess { session ->
                sessionId = session.id
                _uiState.value = _uiState.value.copy(sessionLoading = false)
            }.onFailure {
                _uiState.value =
                    _uiState.value.copy(
                        sessionLoading = false,
                        error = UiMessage(Res.string.mila_chat_session_failed),
                    )
            }
    }

    fun sendMessage(question: String) {
        val id = sessionId ?: return
        val trimmed = question.trim()
        if (trimmed.isBlank()) return

        lastQuestion = trimmed
        val userMsgId = "local_user_${getTimeMillis()}"
        val streamingMsgId = "local_stream_${getTimeMillis() + 1}"

        val userMessage =
            ChatMessage(
                id = userMsgId,
                role = "user",
                content = trimmed,
            )
        val streamingPlaceholder =
            ChatMessage(
                id = streamingMsgId,
                role = "assistant",
                content = "",
                isStreaming = true,
            )

        _uiState.value =
            _uiState.value.copy(
                messages = _uiState.value.messages + userMessage + streamingPlaceholder,
                isStreaming = true,
                inputText = "",
                error = null,
            )

        viewModelScope.launch {
            var accumulatedContent = ""
            // The stream throws on any transport failure — a socket timeout
            // mid-answer, a dropped connection. Uncaught it would take the
            // whole process down; it must land as an in-chat error instead.
            val stream =
                repository.streamChat(id, trimmed).catch { failure ->
                    emit(StreamEvent.Error(failure.message.orEmpty()))
                }
            stream.collect { event ->
                when (event) {
                    is StreamEvent.Delta -> {
                        accumulatedContent += event.text
                        updateLastAssistantMessage(streamingMsgId, accumulatedContent, true)
                    }

                    is StreamEvent.Error -> {
                        _uiState.value =
                            _uiState.value.copy(
                                messages = _uiState.value.messages.filter { it.id != streamingMsgId },
                                isStreaming = false,
                                error = friendlyStreamError(event.message),
                            )
                    }

                    is StreamEvent.Done -> {
                        completeStreamingMessage(streamingMsgId)
                        reconcileCanonicalAnswer(
                            sessionId = id,
                            question = trimmed,
                            messageId = streamingMsgId,
                        )
                    }
                }
            }
        }
    }

    // 503 on stream open is the ai_provider_unavailable contract (provider offline).
    private fun friendlyStreamError(message: String): UiMessage =
        when {
            message.contains("503") ->
                UiMessage(Res.string.mila_chat_provider_unavailable)
            message.contains("timeout", ignoreCase = true) ->
                UiMessage(Res.string.mila_chat_timeout)
            else -> UiMessage(Res.string.mila_chat_failed)
        }

    private fun updateLastAssistantMessage(
        messageId: String,
        content: String,
        streaming: Boolean,
    ) {
        _uiState.value =
            _uiState.value.copy(
                messages =
                    _uiState.value.messages.map { msg ->
                        if (msg.id == messageId) {
                            msg.copy(content = content, isStreaming = streaming)
                        } else {
                            msg
                        }
                    },
            )
    }

    private fun completeStreamingMessage(messageId: String) {
        _uiState.value =
            _uiState.value.copy(
                isStreaming = false,
                messages =
                    _uiState.value.messages.map { message ->
                        if (message.id == messageId) {
                            message.copy(isStreaming = false)
                        } else {
                            message
                        }
                    },
            )
    }

    private suspend fun reconcileCanonicalAnswer(
        sessionId: String,
        question: String,
        messageId: String,
    ) {
        repository
            .getMessages(sessionId)
            .onSuccess { conversation ->
                val canonicalAssistant = conversation.messages.lastOrNull()
                val canonicalUser = conversation.messages.getOrNull(conversation.messages.lastIndex - 1)
                if (
                    canonicalAssistant?.role != "assistant" ||
                    canonicalUser?.role != "user" ||
                    canonicalUser.content != question
                ) {
                    return@onSuccess
                }
                _uiState.value =
                    _uiState.value.copy(
                        messages =
                            _uiState.value.messages.map { message ->
                                if (message.id == messageId) {
                                    message.copy(
                                        content = canonicalAssistant.content,
                                        sourceRefs =
                                            canonicalAssistant.sourceRefs.map { ref ->
                                                SourceRef(
                                                    documentId = ref.documentId,
                                                    itemTitle = ref.itemTitle,
                                                )
                                            },
                                    )
                                } else {
                                    message
                                }
                            },
                    )
            }
    }

    fun onInputChange(text: String) {
        _uiState.value = _uiState.value.copy(inputText = text)
    }

    fun onSetupMila() {
        viewModelScope.launch { _effects.emit(MilaChatEffect.NavigateToAiSettings) }
    }

    fun onSourceRef(documentId: String) {
        viewModelScope.launch { _effects.emit(MilaChatEffect.NavigateToItem(documentId)) }
    }

    fun retry() {
        val question = lastQuestion ?: return
        // Remove the user message left from the failed attempt so sendMessage re-adds it cleanly
        val msgs = _uiState.value.messages
        if (msgs.lastOrNull()?.let { it.role == "user" && it.content == question } == true) {
            _uiState.value = _uiState.value.copy(messages = msgs.dropLast(1), error = null)
        }
        sendMessage(question)
    }
}
