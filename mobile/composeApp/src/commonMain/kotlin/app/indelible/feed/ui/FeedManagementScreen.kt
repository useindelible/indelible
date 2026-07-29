package app.indelible.feed.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import app.indelible.feed.model.FeedSubscription
import app.indelible.feed.model.UpdateSubscriptionRequest
import app.indelible.feed.ui.components.SubscriptionRow
import app.indelible.feed.viewmodel.FeedManagementEffect
import app.indelible.feed.viewmodel.FeedManagementUiState
import app.indelible.feed.viewmodel.FeedManagementViewModel
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.components.IndelibleTextField
import app.indelible.ui.theme.IndelibleSpacing

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FeedManagementScreen(
    viewModel: FeedManagementViewModel,
    onNavigateBack: () -> Unit,
    onNavigateToAddFeed: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val uiState by viewModel.uiState.collectAsState()
    var editingSubscription by remember { mutableStateOf<FeedSubscription?>(null) }
    val snackbarHostState = remember { SnackbarHostState() }
    var searchQuery by remember { mutableStateOf("") }

    LaunchedEffect(viewModel) {
        viewModel.effects.collect { effect ->
            when (effect) {
                is FeedManagementEffect.ShowSnackbar ->
                    snackbarHostState.showSnackbar(effect.message)
            }
        }
    }

    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = "Manage Feeds",
                        style = MaterialTheme.typography.headlineSmall,
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = "Back",
                        )
                    }
                },
            )
        },
        snackbarHost = { SnackbarHost(snackbarHostState) },
    ) { paddingValues ->
        Box(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(paddingValues),
        ) {
            when (val state = uiState) {
                is FeedManagementUiState.Loading -> {
                    CircularProgressIndicator(
                        modifier = Modifier.align(Alignment.Center),
                    )
                }

                is FeedManagementUiState.Error -> {
                    Text(
                        text = state.message,
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.error,
                        modifier = Modifier.align(Alignment.Center),
                    )
                }

                is FeedManagementUiState.Success -> {
                    FeedManagementContent(
                        state = state,
                        searchQuery = searchQuery,
                        onQueryChange = { searchQuery = it },
                        onEdit = { editingSubscription = it },
                        onDelete = { viewModel.deleteSubscription(it) },
                        onToggle = { viewModel.toggleStatus(it) },
                        onToggleAutoSave = { viewModel.toggleAutoSave(it) },
                        onNavigateToAddFeed = onNavigateToAddFeed,
                    )
                }
            }
        }
    }

    editingSubscription?.let { subscription ->
        EditSubscriptionSheet(
            subscription = subscription,
            onDismiss = { editingSubscription = null },
            onSave = { request ->
                viewModel.updateSubscription(subscription.id, request)
                editingSubscription = null
            },
            onDelete = {
                viewModel.deleteSubscription(subscription.id)
                editingSubscription = null
            },
        )
    }
}

@Composable
private fun SearchBar(
    query: String,
    onQueryChange: (String) -> Unit,
    onClear: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier =
            modifier
                .fillMaxWidth()
                .padding(
                    horizontal = IndelibleSpacing.step16,
                    vertical = IndelibleSpacing.step12,
                ).height(IndelibleSpacing.step40)
                .background(
                    MaterialTheme.colorScheme.surfaceVariant,
                    MaterialTheme.shapes.medium,
                ).border(0.5.dp, MaterialTheme.colorScheme.outline, MaterialTheme.shapes.medium),
    ) {
        Row(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(horizontal = IndelibleSpacing.step12),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
        ) {
            Icon(
                imageVector = Icons.Filled.Search,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.size(IndelibleSpacing.step20),
            )
            BasicTextField(
                value = query,
                onValueChange = onQueryChange,
                modifier = Modifier.weight(1f),
                textStyle =
                    MaterialTheme.typography.bodyLarge.copy(
                        color = MaterialTheme.colorScheme.onSurface,
                    ),
                singleLine = true,
                decorationBox = { innerTextField ->
                    if (query.isEmpty()) {
                        Text(
                            text = "Search feeds\u2026",
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    innerTextField()
                },
            )
            if (query.isNotEmpty()) {
                IconButton(
                    onClick = onClear,
                    modifier = Modifier.size(IndelibleSpacing.step20),
                ) {
                    Icon(
                        imageVector = Icons.Filled.Close,
                        contentDescription = "Clear search",
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }
    }
}

@Composable
private fun FeedManagementContent(
    state: FeedManagementUiState.Success,
    searchQuery: String,
    onQueryChange: (String) -> Unit,
    onEdit: (FeedSubscription) -> Unit,
    onDelete: (String) -> Unit,
    onToggle: (FeedSubscription) -> Unit,
    onToggleAutoSave: (FeedSubscription) -> Unit,
    onNavigateToAddFeed: () -> Unit,
) {
    val filtered =
        if (searchQuery.isBlank()) {
            state.subscriptions
        } else {
            state.subscriptions.filter { sub ->
                val name = sub.titleOverride ?: sub.source.name
                name.contains(searchQuery, ignoreCase = true) ||
                    sub.source.domain?.contains(searchQuery, ignoreCase = true) == true
            }
        }

    Column(
        modifier =
            Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState()),
    ) {
        SearchBar(
            query = searchQuery,
            onQueryChange = onQueryChange,
            onClear = { onQueryChange("") },
        )

        when {
            state.subscriptions.isEmpty() -> {
                Box(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .padding(vertical = IndelibleSpacing.step32),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        text = "No subscriptions yet",
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }

            filtered.isEmpty() -> {
                Box(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .padding(vertical = IndelibleSpacing.step32),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        text = "No results for \"$searchQuery\"",
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }

            else -> {
                Card(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .padding(
                                horizontal = IndelibleSpacing.step16,
                                vertical = IndelibleSpacing.step8,
                            ),
                    shape = MaterialTheme.shapes.extraLarge,
                    colors =
                        CardDefaults.cardColors(
                            containerColor = MaterialTheme.colorScheme.surfaceVariant,
                        ),
                    border =
                        androidx.compose.foundation.BorderStroke(
                            0.5.dp,
                            MaterialTheme.colorScheme.outlineVariant,
                        ),
                    elevation = CardDefaults.cardElevation(defaultElevation = 0.dp),
                ) {
                    Column {
                        filtered.forEachIndexed { index, subscription ->
                            SubscriptionRow(
                                subscription = subscription,
                                onToggleStatus = { onToggle(subscription) },
                                onToggleAutoSave = { onToggleAutoSave(subscription) },
                                onEdit = { onEdit(subscription) },
                                onDelete = { onDelete(subscription.id) },
                            )
                            if (index < filtered.lastIndex) {
                                HorizontalDivider(
                                    color = MaterialTheme.colorScheme.outlineVariant,
                                )
                            }
                        }
                    }
                }
            }
        }

        IndelibleButton(
            text = "+ Add Feed",
            style = IndelibleButtonStyle.Secondary,
            onClick = onNavigateToAddFeed,
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(
                        horizontal = IndelibleSpacing.step16,
                        vertical = IndelibleSpacing.step12,
                    ),
        )
    }
}

@Composable
private fun AutoSaveToggleRow(
    autoSave: Boolean,
    onCheckedChange: (Boolean) -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = "Auto-save to Library",
            style = MaterialTheme.typography.bodyLarge,
        )
        Switch(
            checked = autoSave,
            onCheckedChange = onCheckedChange,
        )
    }
}

@Composable
private fun SubscriptionSheetActions(
    isPaused: Boolean,
    titleInput: String,
    autoSave: Boolean,
    onSave: (UpdateSubscriptionRequest) -> Unit,
) {
    IndelibleButton(
        text = if (isPaused) "Resume" else "Pause",
        onClick = {
            val newStatus = if (isPaused) "active" else "paused"
            onSave(
                UpdateSubscriptionRequest(
                    title = titleInput.trim().ifBlank { null },
                    autoSave = autoSave,
                    status = newStatus,
                ),
            )
        },
    )
    Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
    IndelibleButton(
        text = "Save Changes",
        onClick = {
            onSave(
                UpdateSubscriptionRequest(
                    title = titleInput.trim().ifBlank { null },
                    autoSave = autoSave,
                ),
            )
        },
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun EditSubscriptionSheet(
    subscription: FeedSubscription,
    onDismiss: () -> Unit,
    onSave: (UpdateSubscriptionRequest) -> Unit,
    onDelete: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val sheetState = rememberModalBottomSheetState()
    var titleInput by remember { mutableStateOf(subscription.titleOverride ?: subscription.source.name) }
    var autoSave by remember { mutableStateOf(subscription.autoSave) }
    val isPaused = subscription.status == "paused"

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        modifier = modifier,
    ) {
        Column(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = IndelibleSpacing.screenPaddingH)
                    .padding(bottom = IndelibleSpacing.screenPaddingV),
        ) {
            Text(
                text = "Edit Subscription",
                style = MaterialTheme.typography.headlineMedium,
            )
            Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

            IndelibleTextField(
                value = titleInput,
                onValueChange = { titleInput = it },
                label = "Title",
                modifier = Modifier.fillMaxWidth(),
            )
            Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

            AutoSaveToggleRow(
                autoSave = autoSave,
                onCheckedChange = { autoSave = it },
            )
            Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

            SubscriptionSheetActions(
                isPaused = isPaused,
                titleInput = titleInput,
                autoSave = autoSave,
                onSave = onSave,
            )
            Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

            IndelibleButton(
                text = "Delete Subscription",
                onClick = onDelete,
                style = IndelibleButtonStyle.Destructive,
            )
        }
    }
}
