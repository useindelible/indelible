package app.indelible.mila.viewmodel

import app.indelible.api.generated.models.MilaSessionListResponse
import app.indelible.mila.data.ChatScope
import app.indelible.mila.data.StreamEvent
import app.indelible.mila.viewmodel.FakeMilaRepository.Companion.conversationWithMessages
import app.indelible.mila.viewmodel.FakeMilaRepository.Companion.disabledConfig
import app.indelible.mila.viewmodel.FakeMilaRepository.Companion.enabledConfig
import app.indelible.mila.viewmodel.FakeMilaRepository.Companion.fakeSession
import app.indelible.mila.viewmodel.FakeMilaRepository.Companion.fakeSessionPreview
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class MilaChatViewModelTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private lateinit var repository: FakeMilaRepository
    private lateinit var viewModel: MilaChatViewModel

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        repository = FakeMilaRepository()
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun disabled_config_shows_not_configured() =
        runTest(testDispatcher) {
            repository.configResult = Result.success(disabledConfig())

            viewModel = MilaChatViewModel(repository, ChatScope.SingleDocument("doc_01"))
            advanceUntilIdle()

            val state = viewModel.uiState.value
            assertFalse(state.configLoading)
            assertFalse(state.milaEnabled)
        }

    @Test
    fun config_failure_shows_not_configured() =
        runTest(testDispatcher) {
            repository.configResult = Result.failure(RuntimeException("Network error"))

            viewModel = MilaChatViewModel(repository, ChatScope.SingleDocument("doc_01"))
            advanceUntilIdle()

            val state = viewModel.uiState.value
            assertFalse(state.configLoading)
            assertFalse(state.milaEnabled)
        }

    @Test
    fun enabled_config_creates_new_session_when_no_match() =
        runTest(testDispatcher) {
            repository.configResult = Result.success(enabledConfig())
            repository.listSessionsResult = Result.success(FakeMilaRepository.emptySessions())
            repository.createSessionResult = Result.success(fakeSession())

            viewModel = MilaChatViewModel(repository, ChatScope.SingleDocument("doc_01"))
            advanceUntilIdle()

            val state = viewModel.uiState.value
            assertFalse(state.configLoading)
            assertTrue(state.milaEnabled)
            assertFalse(state.sessionLoading)
            assertEquals("single_document", repository.lastCreateSessionType)
            assertEquals("doc_01", repository.lastCreateDocumentId)
            assertNull(repository.lastCreateCollectionId)
        }

    @Test
    fun resumes_existing_session_for_single_document() =
        runTest(testDispatcher) {
            repository.configResult = Result.success(enabledConfig())
            repository.listSessionsResult =
                Result.success(
                    MilaSessionListResponse(
                        sessions =
                            listOf(
                                fakeSessionPreview(
                                    id = "existing_session",
                                    sessionType = "single_document",
                                    documentId = "doc_01",
                                ),
                            ),
                    ),
                )
            repository.getMessagesResult = Result.success(conversationWithMessages())

            viewModel = MilaChatViewModel(repository, ChatScope.SingleDocument("doc_01"))
            advanceUntilIdle()

            val state = viewModel.uiState.value
            assertFalse(state.sessionLoading)
            assertEquals(2, state.messages.size)
            assertEquals("user", state.messages[0].role)
            assertEquals("assistant", state.messages[1].role)
            assertEquals(1, state.messages[1].sourceRefs.size)
        }

    @Test
    fun resumes_existing_session_for_collection() =
        runTest(testDispatcher) {
            repository.configResult = Result.success(enabledConfig())
            repository.listSessionsResult =
                Result.success(
                    MilaSessionListResponse(
                        sessions =
                            listOf(
                                fakeSessionPreview(
                                    id = "col_session",
                                    sessionType = "collection",
                                    collectionId = "col_01",
                                    documentId = null,
                                ),
                            ),
                    ),
                )
            repository.getMessagesResult = Result.success(conversationWithMessages())

            viewModel = MilaChatViewModel(repository, ChatScope.Collection("col_01"))
            advanceUntilIdle()

            val state = viewModel.uiState.value
            assertFalse(state.sessionLoading)
            assertEquals(2, state.messages.size)
        }

    @Test
    fun resumes_existing_cross_item_session() =
        runTest(testDispatcher) {
            repository.configResult = Result.success(enabledConfig())
            repository.listSessionsResult =
                Result.success(
                    MilaSessionListResponse(
                        sessions =
                            listOf(
                                fakeSessionPreview(
                                    id = "cross_session",
                                    sessionType = "cross_item",
                                    documentId = null,
                                    collectionId = null,
                                ),
                            ),
                    ),
                )
            repository.getMessagesResult = Result.success(conversationWithMessages())

            viewModel = MilaChatViewModel(repository, ChatScope.CrossItem)
            advanceUntilIdle()

            val state = viewModel.uiState.value
            assertFalse(state.sessionLoading)
            assertEquals(2, state.messages.size)
        }

    @Test
    fun send_message_streams_and_reloads() =
        runTest(testDispatcher) {
            repository.configResult = Result.success(enabledConfig())
            repository.createSessionResult = Result.success(fakeSession(id = "s1"))
            repository.getMessagesResult = Result.success(conversationWithMessages())
            repository.streamEvents =
                listOf(
                    StreamEvent.Delta("Hello"),
                    StreamEvent.Delta(" world"),
                    StreamEvent.Done,
                )

            viewModel = MilaChatViewModel(repository, ChatScope.SingleDocument("doc_01"))
            advanceUntilIdle()

            viewModel.sendMessage("Test question")
            advanceUntilIdle()

            assertEquals("s1", repository.lastStreamSessionId)
            assertEquals("Test question", repository.lastStreamQuestion)

            val state = viewModel.uiState.value
            assertFalse(state.isStreaming)
            assertEquals(2, state.messages.size)
        }

    @Test
    fun stream_completion_preserves_local_ids_and_merges_canonical_answer() =
        runTest(testDispatcher) {
            repository.createSessionResult = Result.success(fakeSession(id = "s1"))
            repository.streamEvents =
                listOf(
                    StreamEvent.Delta("Streamed answer"),
                    StreamEvent.Done,
                )
            repository.getMessagesResult =
                Result.success(
                    conversationWithMessages(
                        userContent = "Test question",
                        assistantContent = "Canonical answer",
                    ),
                )

            viewModel = MilaChatViewModel(repository, ChatScope.SingleDocument("doc_01"))
            advanceUntilIdle()
            viewModel.sendMessage("Test question")
            advanceUntilIdle()

            val state = viewModel.uiState.value
            assertFalse(state.isStreaming)
            assertEquals(2, state.messages.size)
            assertTrue(state.messages[0].id.startsWith("local_user_"))
            assertTrue(state.messages[1].id.startsWith("local_stream_"))
            assertEquals("Canonical answer", state.messages[1].content)
            assertEquals(
                "doc_01",
                state.messages[1]
                    .sourceRefs
                    .single()
                    .documentId,
            )
            assertFalse(state.messages[1].isStreaming)
        }

    @Test
    fun canonical_reload_failure_keeps_completed_streamed_answer() =
        runTest(testDispatcher) {
            repository.createSessionResult = Result.success(fakeSession(id = "s1"))
            repository.streamEvents =
                listOf(
                    StreamEvent.Delta("Keep this answer"),
                    StreamEvent.Done,
                )
            repository.getMessagesResult = Result.failure(RuntimeException("reload failed"))

            viewModel = MilaChatViewModel(repository, ChatScope.SingleDocument("doc_01"))
            advanceUntilIdle()
            viewModel.sendMessage("Test question")
            advanceUntilIdle()

            val state = viewModel.uiState.value
            assertFalse(state.isStreaming)
            assertEquals("Keep this answer", state.messages.last().content)
            assertFalse(state.messages.last().isStreaming)
        }

    @Test
    fun send_message_clears_input() =
        runTest(testDispatcher) {
            repository.configResult = Result.success(enabledConfig())
            repository.createSessionResult = Result.success(fakeSession(id = "s1"))

            viewModel = MilaChatViewModel(repository, ChatScope.SingleDocument("doc_01"))
            advanceUntilIdle()

            viewModel.onInputChange("My question")
            assertEquals("My question", viewModel.uiState.value.inputText)

            viewModel.sendMessage("My question")
            advanceUntilIdle()

            assertEquals("", viewModel.uiState.value.inputText)
        }

    @Test
    fun stream_error_sets_error_state() =
        runTest(testDispatcher) {
            repository.configResult = Result.success(enabledConfig())
            repository.createSessionResult = Result.success(fakeSession(id = "s1"))
            repository.streamEvents =
                listOf(
                    StreamEvent.Delta("partial"),
                    StreamEvent.Error("Provider error"),
                )

            viewModel = MilaChatViewModel(repository, ChatScope.SingleDocument("doc_01"))
            advanceUntilIdle()

            viewModel.sendMessage("What?")
            advanceUntilIdle()

            val state = viewModel.uiState.value
            assertFalse(state.isStreaming)
            assertEquals("Provider error", state.error)
        }

    @Test
    fun provider_unavailable_stream_error_uses_friendly_copy() =
        runTest(testDispatcher) {
            repository.configResult = Result.success(enabledConfig())
            repository.createSessionResult = Result.success(fakeSession(id = "s1"))
            repository.streamEvents = listOf(StreamEvent.Error("Request failed: 503"))

            viewModel = MilaChatViewModel(repository, ChatScope.SingleDocument("doc_01"))
            advanceUntilIdle()

            viewModel.sendMessage("What?")
            advanceUntilIdle()

            val state = viewModel.uiState.value
            assertFalse(state.isStreaming)
            assertEquals(
                "Your AI provider is unreachable. Start it (e.g. LM Studio), then retry.",
                state.error,
            )
        }

    @Test
    fun retry_resends_last_question() =
        runTest(testDispatcher) {
            repository.configResult = Result.success(enabledConfig())
            repository.createSessionResult = Result.success(fakeSession(id = "s1"))
            repository.streamEvents = listOf(StreamEvent.Error("fail"))

            viewModel = MilaChatViewModel(repository, ChatScope.SingleDocument("doc_01"))
            advanceUntilIdle()

            viewModel.sendMessage("Original question")
            advanceUntilIdle()

            repository.streamEvents =
                listOf(
                    StreamEvent.Delta("Success"),
                    StreamEvent.Done,
                )
            repository.getMessagesResult = Result.success(conversationWithMessages())

            viewModel.retry()
            advanceUntilIdle()

            assertEquals("Original question", repository.lastStreamQuestion)
        }

    @Test
    fun on_input_change_updates_state() =
        runTest(testDispatcher) {
            repository.configResult = Result.success(enabledConfig())
            repository.createSessionResult = Result.success(fakeSession(id = "s1"))

            viewModel = MilaChatViewModel(repository, ChatScope.SingleDocument("doc_01"))
            advanceUntilIdle()

            viewModel.onInputChange("hello")
            assertEquals("hello", viewModel.uiState.value.inputText)
        }

    @Test
    fun blank_message_is_not_sent() =
        runTest(testDispatcher) {
            repository.configResult = Result.success(enabledConfig())
            repository.createSessionResult = Result.success(fakeSession(id = "s1"))

            viewModel = MilaChatViewModel(repository, ChatScope.SingleDocument("doc_01"))
            advanceUntilIdle()

            viewModel.sendMessage("   ")
            advanceUntilIdle()

            assertNull(repository.lastStreamQuestion)
        }

    @Test
    fun collection_scope_creates_correct_session_type() =
        runTest(testDispatcher) {
            repository.configResult = Result.success(enabledConfig())
            repository.createSessionResult =
                Result.success(
                    fakeSession(sessionType = "collection", documentId = null, collectionId = "col_01"),
                )

            viewModel = MilaChatViewModel(repository, ChatScope.Collection("col_01"))
            advanceUntilIdle()

            assertEquals("collection", repository.lastCreateSessionType)
            assertNull(repository.lastCreateDocumentId)
            assertEquals("col_01", repository.lastCreateCollectionId)
        }

    @Test
    fun stream_exception_surfaces_an_error_instead_of_crashing() =
        runTest(testDispatcher) {
            viewModel = MilaChatViewModel(repository, ChatScope.SingleDocument("doc_01"))
            advanceUntilIdle()
            repository.streamEvents = listOf(StreamEvent.Delta("partial"))
            repository.streamError = RuntimeException("Socket timeout has expired")

            viewModel.sendMessage("How is a band saw different?")
            advanceUntilIdle()

            val state = viewModel.uiState.value
            assertFalse(state.isStreaming, "a dead stream must end the streaming state")
            assertTrue(state.messages.none { it.isStreaming }, "the placeholder must be removed")
            assertTrue(state.error != null, "the failure must surface as an error")
        }

    @Test
    fun cross_item_scope_creates_correct_session_type() =
        runTest(testDispatcher) {
            repository.configResult = Result.success(enabledConfig())
            repository.createSessionResult =
                Result.success(
                    fakeSession(sessionType = "cross_item", documentId = null, collectionId = null),
                )

            viewModel = MilaChatViewModel(repository, ChatScope.CrossItem)
            advanceUntilIdle()

            assertEquals("cross_item", repository.lastCreateSessionType)
            assertNull(repository.lastCreateDocumentId)
            assertNull(repository.lastCreateCollectionId)
        }
}
