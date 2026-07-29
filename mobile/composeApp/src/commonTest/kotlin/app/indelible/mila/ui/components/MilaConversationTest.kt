package app.indelible.mila.ui.components

import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.runComposeUiTest
import androidx.compose.ui.test.swipeDown
import androidx.compose.ui.test.swipeUp
import androidx.compose.ui.unit.dp
import app.indelible.mila.data.ChatMessage
import app.indelible.mila.data.ChatScope
import app.indelible.mila.data.SourceRef
import app.indelible.mila.viewmodel.MilaChatUiState
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test

@OptIn(ExperimentalTestApi::class)
class MilaConversationTest {
    @Test
    fun long_stream_stays_at_the_bottom() =
        runComposeUiTest {
            var state by mutableStateOf(conversationState(streamingContent(24)))
            setContent {
                AppTheme {
                    MilaConversation(
                        state = state,
                        scope = ChatScope.SingleDocument("doc_01"),
                        onSendMessage = {},
                        onInputChange = {},
                        onSourceRef = {},
                        onRetry = {},
                        modifier = Modifier.width(360.dp).height(320.dp),
                    )
                }
            }

            onNodeWithTag(MILA_BOTTOM_ANCHOR_TEST_TAG).assertIsDisplayed()

            runOnIdle {
                state = conversationState(streamingContent(80))
            }

            onNodeWithTag(MILA_BOTTOM_ANCHOR_TEST_TAG).assertIsDisplayed()
        }

    @Test
    fun manual_scroll_pauses_follow_until_the_user_returns_to_the_bottom() =
        runComposeUiTest {
            var state by mutableStateOf(conversationState(streamingContent(80)))
            setContent {
                AppTheme {
                    MilaConversation(
                        state = state,
                        scope = ChatScope.SingleDocument("doc_01"),
                        onSendMessage = {},
                        onInputChange = {},
                        onSourceRef = {},
                        onRetry = {},
                        modifier = Modifier.width(360.dp).height(320.dp),
                    )
                }
            }

            onNodeWithTag(MILA_BOTTOM_ANCHOR_TEST_TAG).assertIsDisplayed()
            onNodeWithTag(MILA_MESSAGE_LIST_TEST_TAG).performTouchInput { swipeDown() }
            onAllNodesWithTag(MILA_BOTTOM_ANCHOR_TEST_TAG).assertCountEquals(0)

            runOnIdle {
                state = conversationState(streamingContent(100))
            }

            onAllNodesWithTag(MILA_BOTTOM_ANCHOR_TEST_TAG).assertCountEquals(0)

            repeat(12) {
                onNodeWithTag(MILA_MESSAGE_LIST_TEST_TAG).performTouchInput { swipeUp() }
            }
            onNodeWithTag(MILA_BOTTOM_ANCHOR_TEST_TAG).assertIsDisplayed()

            runOnIdle {
                state = conversationState(streamingContent(120))
            }
            onNodeWithTag(MILA_BOTTOM_ANCHOR_TEST_TAG).assertIsDisplayed()
        }

    @Test
    fun a_new_message_resumes_follow_after_the_user_scrolled_away() =
        runComposeUiTest {
            var state by mutableStateOf(conversationState(streamingContent(80)))
            setContent {
                AppTheme {
                    MilaConversation(
                        state = state,
                        scope = ChatScope.SingleDocument("doc_01"),
                        onSendMessage = {},
                        onInputChange = {},
                        onSourceRef = {},
                        onRetry = {},
                        modifier = Modifier.width(360.dp).height(320.dp),
                    )
                }
            }

            onNodeWithTag(MILA_MESSAGE_LIST_TEST_TAG).performTouchInput { swipeDown() }
            onAllNodesWithTag(MILA_BOTTOM_ANCHOR_TEST_TAG).assertCountEquals(0)

            runOnIdle {
                state =
                    state.copy(
                        messages =
                            state.messages +
                                ChatMessage(
                                    id = "user_2",
                                    role = "user",
                                    content = "Follow-up",
                                ) +
                                ChatMessage(
                                    id = "assistant_2",
                                    role = "assistant",
                                    content = "Starting the follow-up response",
                                    isStreaming = true,
                                ),
                    )
            }

            onNodeWithTag(MILA_BOTTOM_ANCHOR_TEST_TAG).assertIsDisplayed()
        }

    @Test
    fun viewport_shrink_and_markdown_completion_keep_the_tail_visible() =
        runComposeUiTest {
            var height by mutableStateOf(360.dp)
            var state by mutableStateOf(conversationState(streamingContent(48)))
            setContent {
                AppTheme {
                    MilaConversation(
                        state = state,
                        scope = ChatScope.SingleDocument("doc_01"),
                        onSendMessage = {},
                        onInputChange = {},
                        onSourceRef = {},
                        onRetry = {},
                        modifier = Modifier.width(360.dp).height(height),
                    )
                }
            }

            onNodeWithTag(MILA_BOTTOM_ANCHOR_TEST_TAG).assertIsDisplayed()

            runOnIdle {
                height = 240.dp
                state =
                    conversationState(
                        content = "## Complete\n\n${streamingContent(48)}",
                        isStreaming = false,
                        sourceRefs =
                            listOf(
                                SourceRef(
                                    documentId = "doc_01",
                                    itemTitle = "Source title",
                                ),
                            ),
                    )
            }

            onNodeWithTag(MILA_BOTTOM_ANCHOR_TEST_TAG).assertIsDisplayed()
        }

    private fun conversationState(
        content: String,
        isStreaming: Boolean = true,
        sourceRefs: List<SourceRef> = emptyList(),
    ) = MilaChatUiState(
        milaEnabled = true,
        isStreaming = isStreaming,
        messages =
            listOf(
                ChatMessage(
                    id = "user_1",
                    role = "user",
                    content = "Explain this article",
                ),
                ChatMessage(
                    id = "assistant_1",
                    role = "assistant",
                    content = content,
                    sourceRefs = sourceRefs,
                    isStreaming = isStreaming,
                ),
            ),
    )

    private fun streamingContent(lines: Int): String =
        (1..lines).joinToString("\n") { line -> "Streaming response line $line with enough text to wrap." }
}
