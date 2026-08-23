package app.indelible.mila.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.AutoAwesome
import androidx.compose.material.icons.filled.Description
import androidx.compose.material.icons.filled.Folder
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import app.indelible.mila.data.ChatScope
import app.indelible.mila.ui.components.MilaConversation
import app.indelible.mila.ui.components.MilaNotConfigured
import app.indelible.mila.viewmodel.MilaChatEffect
import app.indelible.mila.viewmodel.MilaChatViewModel
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_back
import indelible.composeapp.generated.resources.mila_collection_chat
import indelible.composeapp.generated.resources.mila_cross_item_chat
import indelible.composeapp.generated.resources.mila_document_chat
import indelible.composeapp.generated.resources.mila_title
import org.jetbrains.compose.resources.stringResource

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MilaChatScreen(
    viewModel: MilaChatViewModel,
    onBack: () -> Unit,
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

    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(
                windowInsets = WindowInsets(0),
                title = {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text(
                            text = stringResource(Res.string.mila_title),
                            style = MaterialTheme.typography.titleLarge,
                        )
                        ScopeSubtitle(scope = viewModel.scope)
                    }
                },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = stringResource(Res.string.common_back),
                            tint = MaterialTheme.colorScheme.primary,
                        )
                    }
                },
            )
        },
    ) { paddingValues ->
        when {
            state.configLoading -> {
                Box(
                    modifier =
                        Modifier
                            .fillMaxSize()
                            .padding(paddingValues),
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
                            .fillMaxSize()
                            .padding(paddingValues),
                )
            }

            state.sessionLoading -> {
                Box(
                    modifier =
                        Modifier
                            .fillMaxSize()
                            .padding(paddingValues),
                    contentAlignment = Alignment.Center,
                ) {
                    CircularProgressIndicator()
                }
            }

            else -> {
                MilaConversation(
                    state = state,
                    scope = viewModel.scope,
                    onSendMessage = { viewModel.sendMessage(it) },
                    onInputChange = { viewModel.onInputChange(it) },
                    onSourceRef = { viewModel.onSourceRef(it) },
                    onRetry = { viewModel.retry() },
                    modifier =
                        Modifier
                            .fillMaxSize()
                            .padding(paddingValues),
                )
            }
        }
    }
}

@Composable
private fun ScopeSubtitle(scope: ChatScope) {
    val (icon, labelRes) =
        when (scope) {
            is ChatScope.SingleDocument -> Icons.Filled.Description to Res.string.mila_document_chat
            is ChatScope.Collection -> Icons.Filled.Folder to Res.string.mila_collection_chat
            is ChatScope.CrossItem -> Icons.Filled.AutoAwesome to Res.string.mila_cross_item_chat
        }
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.Center,
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            modifier = Modifier.size(IndelibleSpacing.step12),
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(modifier = Modifier.width(IndelibleSpacing.step4))
        Text(
            text = stringResource(labelRes),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
