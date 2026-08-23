package app.indelible.mila.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.collectIsDraggedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Send
import androidx.compose.material.icons.filled.AutoAwesome
import androidx.compose.material.icons.filled.Description
import androidx.compose.material.icons.filled.Folder
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.core.i18n.UiMessage
import app.indelible.core.i18n.resolve
import app.indelible.mila.data.ChatMessage
import app.indelible.mila.data.ChatScope
import app.indelible.mila.data.SourceRef
import app.indelible.mila.viewmodel.MilaChatUiState
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_retry
import indelible.composeapp.generated.resources.mila_all_items
import indelible.composeapp.generated.resources.mila_empty_hint
import indelible.composeapp.generated.resources.mila_input_placeholder
import indelible.composeapp.generated.resources.mila_not_configured_body
import indelible.composeapp.generated.resources.mila_not_configured_title
import indelible.composeapp.generated.resources.mila_send_cd
import indelible.composeapp.generated.resources.mila_set_up
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.first
import org.jetbrains.compose.resources.stringResource

/**
 * Reusable Mila chat surface: scope chip, message thread, error row, and composer.
 * Shared by the full-screen [app.indelible.mila.ui.MilaChatScreen] and the in-reader
 * Mila drawer so neither reimplements the conversation UI. The host owns the
 * [MilaChatUiState] and supplies callbacks; set [showScopeChip] to false when the
 * host already shows the chat's scope in its own header.
 */
@Composable
fun MilaConversation(
    state: MilaChatUiState,
    scope: ChatScope,
    onSendMessage: (String) -> Unit,
    onInputChange: (String) -> Unit,
    onSourceRef: (String) -> Unit,
    onRetry: () -> Unit,
    modifier: Modifier = Modifier,
    showScopeChip: Boolean = true,
) {
    val listState = rememberLazyListState()
    val isDragged by listState.interactionSource.collectIsDraggedAsState()
    var followTail by remember { mutableStateOf(true) }
    var previousMessageCount by remember { mutableStateOf(state.messages.size) }

    LaunchedEffect(state.messages.size) {
        if (state.messages.size > previousMessageCount) {
            followTail = true
        }
        previousMessageCount = state.messages.size
    }

    LaunchedEffect(isDragged) {
        if (isDragged) {
            followTail = false
        } else {
            snapshotFlow { listState.isScrollInProgress }
                .filter { scrolling -> !scrolling }
                .first()
            if (!listState.canScrollForward) {
                followTail = true
            }
        }
    }

    LaunchedEffect(listState, followTail) {
        if (!followTail) return@LaunchedEffect
        snapshotFlow {
            listState.layoutInfo.totalItemsCount to listState.canScrollForward
        }.filter { (itemCount, canScrollForward) ->
            itemCount > 0 && canScrollForward
        }.collectLatest {
            withFrameNanos { }
            val itemCount = listState.layoutInfo.totalItemsCount
            if (itemCount > 0 && listState.canScrollForward) {
                listState.scrollToItem(itemCount - 1)
            }
        }
    }

    Column(modifier = modifier) {
        if (showScopeChip) {
            ScopeChipRow(scope = scope)
        }

        LazyColumn(
            state = listState,
            modifier =
                Modifier
                    .weight(1f)
                    .fillMaxWidth()
                    .testTag(MILA_MESSAGE_LIST_TEST_TAG),
            contentPadding =
                PaddingValues(
                    horizontal = IndelibleSpacing.step16,
                    vertical = IndelibleSpacing.step12,
                ),
            verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
        ) {
            if (state.messages.isEmpty()) {
                item {
                    EmptyHint()
                }
            }
            items(state.messages, key = { it.id }) { message ->
                MilaMessage(
                    message = message,
                    onSourceRef = onSourceRef,
                )
            }
            item(key = MILA_BOTTOM_ANCHOR_KEY) {
                Spacer(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .height(IndelibleSpacing.step2)
                            .testTag(MILA_BOTTOM_ANCHOR_TEST_TAG),
                )
            }
        }

        if (state.error != null) {
            ErrorRow(
                error = state.error,
                onRetry = onRetry,
            )
        }

        InputRow(
            inputText = state.inputText,
            isStreaming = state.isStreaming,
            onInputChange = onInputChange,
            onSend = { onSendMessage(state.inputText) },
        )
    }
}

/**
 * Empty-provider state shown when Mila has no AI provider configured. Shared by the
 * chat screen and the reader drawer; the host passes a [modifier] that sizes/pads it.
 */
@Composable
fun MilaNotConfigured(
    onSetup: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier.padding(horizontal = IndelibleSpacing.screenPaddingH),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Icon(
            imageVector = Icons.Filled.AutoAwesome,
            contentDescription = null,
            modifier = Modifier.size(IndelibleSpacing.step64),
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(modifier = Modifier.height(IndelibleSpacing.sectionGap))
        Text(
            text = stringResource(Res.string.mila_not_configured_title),
            style = MaterialTheme.typography.headlineMedium,
            textAlign = TextAlign.Center,
        )
        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
        Text(
            text = stringResource(Res.string.mila_not_configured_body),
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
        )
        Spacer(modifier = Modifier.height(IndelibleSpacing.sectionGap))
        IndelibleButton(
            text = stringResource(Res.string.mila_set_up),
            onClick = onSetup,
        )
    }
}

@Composable
private fun ScopeChipRow(scope: ChatScope) {
    val (icon, label) =
        when (scope) {
            is ChatScope.SingleDocument ->
                Icons.Filled.Description to (scope.displayTitle ?: scope.documentId).take(MAX_CHIP_TITLE_LENGTH)

            is ChatScope.Collection ->
                Icons.Filled.Folder to (scope.displayTitle ?: scope.collectionId).take(MAX_CHIP_TITLE_LENGTH)

            is ChatScope.CrossItem ->
                Icons.Filled.AutoAwesome to stringResource(Res.string.mila_all_items)
        }

    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .padding(
                    horizontal = IndelibleSpacing.screenPaddingH,
                    vertical = IndelibleSpacing.step4,
                ),
    ) {
        Surface(
            shape = MaterialTheme.shapes.small,
            color = MaterialTheme.colorScheme.primaryContainer,
        ) {
            Row(
                modifier =
                    Modifier.padding(
                        horizontal = IndelibleSpacing.step8,
                        vertical = IndelibleSpacing.step4,
                    ),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    modifier = Modifier.size(IndelibleSpacing.step12),
                    tint = MaterialTheme.colorScheme.onPrimaryContainer,
                )
                Spacer(modifier = Modifier.width(IndelibleSpacing.step4))
                Text(
                    text = label,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onPrimaryContainer,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}

@Composable
private fun EmptyHint() {
    Box(
        modifier =
            Modifier
                .fillMaxWidth()
                .padding(vertical = IndelibleSpacing.step40),
        contentAlignment = Alignment.Center,
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Icon(
                imageVector = Icons.Filled.AutoAwesome,
                contentDescription = null,
                modifier = Modifier.size(IndelibleSpacing.step32),
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
            Text(
                text = stringResource(Res.string.mila_empty_hint),
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center,
            )
        }
    }
}

@Composable
private fun ErrorRow(
    error: UiMessage,
    onRetry: () -> Unit,
) {
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .padding(horizontal = IndelibleSpacing.screenPaddingH)
                .background(MaterialTheme.colorScheme.surfaceVariant)
                .padding(
                    horizontal = IndelibleSpacing.step12,
                    vertical = IndelibleSpacing.step8,
                ),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = error.resolve(),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.error,
            modifier = Modifier.weight(1f),
        )
        IndelibleButton(
            text = stringResource(Res.string.common_retry),
            onClick = onRetry,
            style = IndelibleButtonStyle.Text,
        )
    }
}

@Composable
private fun InputRow(
    inputText: String,
    isStreaming: Boolean,
    onInputChange: (String) -> Unit,
    onSend: () -> Unit,
) {
    val canSend = inputText.isNotBlank() && !isStreaming
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .padding(
                    horizontal = IndelibleSpacing.step16,
                    vertical = IndelibleSpacing.step12,
                ),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
    ) {
        Box(
            modifier =
                Modifier
                    .weight(1f)
                    .height(IndelibleSpacing.touchTarget)
                    .background(MaterialTheme.colorScheme.surfaceVariant, IndelibleShape.xl)
                    .border(
                        IndelibleSpacing.hairline,
                        MaterialTheme.colorScheme.outline,
                        IndelibleShape.xl,
                    ).padding(horizontal = IndelibleSpacing.step14),
            contentAlignment = Alignment.CenterStart,
        ) {
            BasicTextField(
                value = inputText,
                onValueChange = onInputChange,
                textStyle =
                    MaterialTheme.typography.bodyLarge.copy(
                        color = MaterialTheme.colorScheme.onSurface,
                    ),
                cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                singleLine = true,
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
                keyboardActions = KeyboardActions(onSend = { if (canSend) onSend() }),
                modifier = Modifier.fillMaxWidth(),
            )
            if (inputText.isEmpty()) {
                Text(
                    text = stringResource(Res.string.mila_input_placeholder),
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
        Box(
            modifier =
                Modifier
                    .size(IndelibleSpacing.touchTarget)
                    .background(
                        if (canSend) {
                            MaterialTheme.colorScheme.primary
                        } else {
                            MaterialTheme.colorScheme.surfaceVariant
                        },
                        IndelibleShape.xl,
                    ).clickable(enabled = canSend, onClick = onSend),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                imageVector = Icons.AutoMirrored.Filled.Send,
                contentDescription = stringResource(Res.string.mila_send_cd),
                modifier = Modifier.size(IndelibleSpacing.step20),
                tint =
                    if (canSend) {
                        MaterialTheme.colorScheme.onPrimary
                    } else {
                        MaterialTheme.colorScheme.onSurfaceVariant
                    },
            )
        }
    }
}

private const val MAX_CHIP_TITLE_LENGTH = 30
private const val MILA_BOTTOM_ANCHOR_KEY = "mila-bottom-anchor"
internal const val MILA_MESSAGE_LIST_TEST_TAG = "mila-message-list"
internal const val MILA_BOTTOM_ANCHOR_TEST_TAG = "mila-bottom-anchor"

private val sampleConversationState =
    MilaChatUiState(
        milaEnabled = true,
        messages =
            listOf(
                ChatMessage(id = "1", role = "user", content = "What is this about?"),
                ChatMessage(
                    id = "2",
                    role = "assistant",
                    content = "This article discusses testing patterns.",
                    sourceRefs = listOf(SourceRef("doc_01", "Testing Best Practices")),
                ),
            ),
    )

@Preview(showBackground = true)
@Composable
private fun MilaConversationPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface(modifier = Modifier.fillMaxSize()) {
            MilaConversation(
                state = sampleConversationState,
                scope = ChatScope.SingleDocument("doc_01", displayTitle = "Testing Best Practices"),
                onSendMessage = {},
                onInputChange = {},
                onSourceRef = {},
                onRetry = {},
                modifier = Modifier.fillMaxSize(),
            )
        }
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun MilaConversationPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface(modifier = Modifier.fillMaxSize()) {
            MilaConversation(
                state = sampleConversationState,
                scope = ChatScope.SingleDocument("doc_01", displayTitle = "Testing Best Practices"),
                onSendMessage = {},
                onInputChange = {},
                onSourceRef = {},
                onRetry = {},
                modifier = Modifier.fillMaxSize(),
            )
        }
    }
}

@Preview(showBackground = true)
@Composable
private fun MilaNotConfiguredPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface(modifier = Modifier.fillMaxSize()) {
            MilaNotConfigured(onSetup = {}, modifier = Modifier.fillMaxSize())
        }
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun MilaNotConfiguredPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface(modifier = Modifier.fillMaxSize()) {
            MilaNotConfigured(onSetup = {}, modifier = Modifier.fillMaxSize())
        }
    }
}
