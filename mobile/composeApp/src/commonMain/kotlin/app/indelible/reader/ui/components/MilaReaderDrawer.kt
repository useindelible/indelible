package app.indelible.reader.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.mila.ui.components.MilaConversation
import app.indelible.mila.ui.components.MilaNotConfigured
import app.indelible.mila.viewmodel.MilaChatEffect
import app.indelible.mila.viewmodel.MilaChatUiState
import app.indelible.mila.viewmodel.MilaChatViewModel
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_close_mila
import indelible.composeapp.generated.resources.reader_reading
import org.jetbrains.compose.resources.stringResource

private const val DRAWER_WIDTH_FRACTION = 0.86f

/**
 * Right-anchored slide-over Mila chat for the reader. Holds an item-scoped
 * [MilaChatViewModel] (constructed lazily by the caller on first open) so it reuses
 * the full Mila session/streaming stack rather than reimplementing it. The header
 * already names the article being read, so the conversation hides its own scope chip.
 */
@Composable
fun MilaReaderDrawer(
    visible: Boolean,
    title: String,
    viewModel: MilaChatViewModel,
    onDismiss: () -> Unit,
    onNavigateToAiSettings: () -> Unit,
    onNavigateToItem: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.uiState.collectAsState()

    LaunchedEffect(viewModel) {
        viewModel.effects.collect { effect ->
            when (effect) {
                is MilaChatEffect.NavigateToAiSettings -> onNavigateToAiSettings()
                is MilaChatEffect.NavigateToItem -> onNavigateToItem(effect.itemId)
            }
        }
    }

    Box(modifier = modifier.fillMaxSize()) {
        AnimatedVisibility(
            visible = visible,
            enter = fadeIn(),
            exit = fadeOut(),
        ) {
            Box(
                modifier =
                    Modifier
                        .fillMaxSize()
                        .background(MaterialTheme.colorScheme.scrim.copy(alpha = 0.32f))
                        .clickable(
                            interactionSource = remember { MutableInteractionSource() },
                            indication = null,
                            onClick = onDismiss,
                        ),
            )
        }

        AnimatedVisibility(
            visible = visible,
            enter =
                slideInHorizontally(
                    animationSpec = tween(easing = FastOutSlowInEasing),
                    initialOffsetX = { it },
                ) + fadeIn(),
            exit =
                slideOutHorizontally(
                    animationSpec = tween(easing = FastOutSlowInEasing),
                    targetOffsetX = { it },
                ) + fadeOut(),
            modifier = Modifier.align(Alignment.CenterEnd),
        ) {
            Surface(
                modifier =
                    Modifier
                        .fillMaxWidth(DRAWER_WIDTH_FRACTION)
                        .fillMaxHeight(),
                shape = IndelibleShape.drawerEnd,
                color = MaterialTheme.colorScheme.surfaceContainer,
                shadowElevation = IndelibleSpacing.step8,
            ) {
                Column(
                    modifier =
                        Modifier
                            .statusBarsPadding()
                            .navigationBarsPadding()
                            .imePadding(),
                ) {
                    DrawerHeader(title = title, onClose = onDismiss)
                    DrawerBody(
                        state = state,
                        viewModel = viewModel,
                    )
                }
            }
        }
    }
}

@Composable
private fun DrawerHeader(
    title: String,
    onClose: () -> Unit,
) {
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .padding(
                    start = IndelibleSpacing.screenPaddingH,
                    top = IndelibleSpacing.step12,
                    bottom = IndelibleSpacing.step8,
                    end = IndelibleSpacing.step8,
                ),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = stringResource(Res.string.reader_reading),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.primary,
            )
            Text(
                text = title,
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        Spacer(modifier = Modifier.width(IndelibleSpacing.step8))
        IconButton(onClick = onClose) {
            Icon(
                imageVector = Icons.Filled.Close,
                contentDescription = stringResource(Res.string.reader_close_mila),
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

@Composable
private fun ColumnScope.DrawerBody(
    state: MilaChatUiState,
    viewModel: MilaChatViewModel,
) {
    when {
        state.configLoading || state.sessionLoading -> {
            Box(
                modifier =
                    Modifier
                        .weight(1f)
                        .fillMaxWidth(),
                contentAlignment = Alignment.Center,
            ) {
                CircularProgressIndicator()
            }
        }

        !state.milaEnabled -> {
            MilaNotConfigured(
                onSetup = { viewModel.onSetupMila() },
                modifier =
                    Modifier
                        .weight(1f)
                        .fillMaxWidth(),
            )
        }

        else -> {
            MilaConversation(
                state = state,
                scope = viewModel.scope,
                onSendMessage = { viewModel.sendMessage(it) },
                onInputChange = { viewModel.onInputChange(it) },
                onSourceRef = { viewModel.onSourceRef(it) },
                onRetry = { viewModel.retry() },
                showScopeChip = false,
                modifier =
                    Modifier
                        .weight(1f)
                        .fillMaxWidth(),
            )
        }
    }
}

@Preview(showBackground = true)
@Composable
private fun MilaReaderDrawerHeaderPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            DrawerHeader(title = "The Quiet Power of Reading Slowly", onClose = {})
        }
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun MilaReaderDrawerHeaderPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            DrawerHeader(title = "The Quiet Power of Reading Slowly", onClose = {})
        }
    }
}
