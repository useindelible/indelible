package app.indelible.library.ui

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
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Bookmark
import androidx.compose.material.icons.filled.BookmarkBorder
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.OpenInBrowser
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Star
import androidx.compose.material.icons.filled.StarBorder
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.core.i18n.LocaleFormatters
import app.indelible.core.i18n.LocalizedDateStyle
import app.indelible.core.i18n.resolve
import app.indelible.core.i18n.resolveString
import app.indelible.core.model.ItemDetail
import app.indelible.library.ui.components.TriageSegmentedControl
import app.indelible.library.viewmodel.ItemDetailEffect
import app.indelible.library.viewmodel.ItemDetailUiState
import app.indelible.library.viewmodel.ItemDetailViewModel
import app.indelible.library.viewmodel.TriageFilter
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_back
import indelible.composeapp.generated.resources.common_coming_soon
import indelible.composeapp.generated.resources.common_delete
import indelible.composeapp.generated.resources.library_action_add_favorite
import indelible.composeapp.generated.resources.library_action_add_shortlist
import indelible.composeapp.generated.resources.library_action_open_original
import indelible.composeapp.generated.resources.library_action_open_reader
import indelible.composeapp.generated.resources.library_action_rearchive
import indelible.composeapp.generated.resources.library_action_remove_favorite
import indelible.composeapp.generated.resources.library_action_remove_shortlist
import indelible.composeapp.generated.resources.library_more_options_cd
import indelible.composeapp.generated.resources.library_pipeline_failed
import indelible.composeapp.generated.resources.library_pipeline_processing
import indelible.composeapp.generated.resources.library_pipeline_ready
import indelible.composeapp.generated.resources.library_pipeline_title
import indelible.composeapp.generated.resources.library_pipeline_unknown
import indelible.composeapp.generated.resources.library_reading_time
import indelible.composeapp.generated.resources.library_triage
import indelible.composeapp.generated.resources.library_word_count
import kotlinx.coroutines.launch
import kotlinx.datetime.Instant
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ItemDetailScreen(
    viewModel: ItemDetailViewModel,
    onNavigateBack: () -> Unit,
    onOpenInReader: (itemId: String) -> Unit = {},
    modifier: Modifier = Modifier,
) {
    val uiState by viewModel.uiState.collectAsState()
    val snackbarHostState = remember { SnackbarHostState() }
    val coroutineScope = rememberCoroutineScope()
    val uriHandler = LocalUriHandler.current
    val comingSoonMessage = stringResource(Res.string.common_coming_soon)

    LaunchedEffect(viewModel) {
        viewModel.effects.collect { effect ->
            when (effect) {
                is ItemDetailEffect.NavigateBack -> onNavigateBack()
                is ItemDetailEffect.ShowSnackbar -> snackbarHostState.showSnackbar(effect.message.resolveString())
            }
        }
    }

    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(
                title = {
                    val title = (uiState as? ItemDetailUiState.Success)?.item?.title ?: ""
                    Text(
                        text = title,
                        style = MaterialTheme.typography.titleSmall,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = stringResource(Res.string.common_back),
                        )
                    }
                },
                actions = {
                    IconButton(
                        onClick = { coroutineScope.launch { snackbarHostState.showSnackbar(comingSoonMessage) } },
                    ) {
                        Icon(
                            imageVector = Icons.Filled.MoreVert,
                            contentDescription = stringResource(Res.string.library_more_options_cd),
                        )
                    }
                },
            )
        },
        snackbarHost = { SnackbarHost(snackbarHostState) },
    ) { paddingValues ->
        when (val state = uiState) {
            is ItemDetailUiState.Loading -> {
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

            is ItemDetailUiState.Error -> {
                Box(
                    modifier =
                        Modifier
                            .fillMaxSize()
                            .padding(paddingValues),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        text = state.message.resolve(),
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.error,
                    )
                }
            }

            is ItemDetailUiState.Success -> {
                ItemDetailContent(
                    item = state.item,
                    onTriage = { triageState -> viewModel.triage(triageState) },
                    onToggleFavorite = { viewModel.toggleFavorite() },
                    onToggleShortlist = { viewModel.toggleShortlist() },
                    onRearchive = { viewModel.rearchive() },
                    onDelete = { viewModel.deleteItem() },
                    onOpenInBrowser = { url -> uriHandler.openUri(url) },
                    onOpenInReader = { onOpenInReader(state.item.id) },
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
private fun ItemDetailContent(
    item: ItemDetail,
    onTriage: (String) -> Unit,
    onToggleFavorite: () -> Unit,
    onToggleShortlist: () -> Unit,
    onRearchive: () -> Unit,
    onDelete: () -> Unit,
    onOpenInBrowser: (String) -> Unit,
    onOpenInReader: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val scrollState = rememberScrollState()

    Column(
        modifier =
            modifier
                .verticalScroll(scrollState)
                .padding(
                    horizontal = IndelibleSpacing.screenPaddingH,
                    vertical = IndelibleSpacing.step16,
                ),
    ) {
        ItemMetaHeader(item = item)

        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))
        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

        PipelineStatusSection(item = item)

        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))
        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

        ItemActionsSection(
            item = item,
            onTriage = onTriage,
            onToggleFavorite = onToggleFavorite,
            onToggleShortlist = onToggleShortlist,
            onRearchive = onRearchive,
            onDelete = onDelete,
            onOpenInBrowser = onOpenInBrowser,
            onOpenInReader = onOpenInReader,
        )
    }
}

@Composable
private fun ItemMetaHeader(item: ItemDetail) {
    Text(
        text = item.title,
        style = MaterialTheme.typography.headlineSmall,
        color = MaterialTheme.colorScheme.onSurface,
    )
    Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

    val publishedDate = item.publishedAt?.let { LocaleFormatters.date(it, LocalizedDateStyle.MEDIUM) }
    val metaLine = listOfNotNull(item.domain, item.author, publishedDate).joinToString(" · ")
    if (metaLine.isNotBlank()) {
        Text(
            text = metaLine,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }

    val statsLine =
        buildString {
            item.wordCount?.let {
                append(pluralStringResource(Res.plurals.library_word_count, it, it))
            }
            item.readingTimeMinutes?.let {
                if (isNotEmpty()) append(" · ")
                append(pluralStringResource(Res.plurals.library_reading_time, it, it))
            }
            item.language?.let {
                if (isNotEmpty()) append(" · ")
                append(it.uppercase())
            }
        }
    if (statsLine.isNotBlank()) {
        Spacer(modifier = Modifier.height(IndelibleSpacing.step4))
        Text(
            text = statsLine,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun ItemActionsSection(
    item: ItemDetail,
    onTriage: (String) -> Unit,
    onToggleFavorite: () -> Unit,
    onToggleShortlist: () -> Unit,
    onRearchive: () -> Unit,
    onDelete: () -> Unit,
    onOpenInBrowser: (String) -> Unit,
    onOpenInReader: () -> Unit,
) {
    val currentTriageFilter =
        when (item.triageState) {
            "inbox" -> TriageFilter.INBOX
            "later" -> TriageFilter.LATER
            "archive" -> TriageFilter.ARCHIVE
            else -> TriageFilter.INBOX
        }

    Text(
        text = stringResource(Res.string.library_triage),
        style = MaterialTheme.typography.bodySmall,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
    )
    Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
    TriageSegmentedControl(
        selected = currentTriageFilter,
        onSelect = { onTriage(it.apiValue) },
    )

    Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
    ) {
        IconButton(onClick = onToggleFavorite) {
            Icon(
                imageVector = if (item.isFavorite) Icons.Filled.Star else Icons.Filled.StarBorder,
                contentDescription =
                    stringResource(
                        if (item.isFavorite) {
                            Res.string.library_action_remove_favorite
                        } else {
                            Res.string.library_action_add_favorite
                        },
                    ),
                tint =
                    if (item.isFavorite) {
                        MaterialTheme.colorScheme.primary
                    } else {
                        MaterialTheme.colorScheme.onSurfaceVariant
                    },
            )
        }
        IconButton(onClick = onToggleShortlist) {
            Icon(
                imageVector = if (item.isShortlisted) Icons.Filled.Bookmark else Icons.Filled.BookmarkBorder,
                contentDescription =
                    stringResource(
                        if (item.isShortlisted) {
                            Res.string.library_action_remove_shortlist
                        } else {
                            Res.string.library_action_add_shortlist
                        },
                    ),
                tint =
                    if (item.isShortlisted) {
                        MaterialTheme.colorScheme.primary
                    } else {
                        MaterialTheme.colorScheme.onSurfaceVariant
                    },
            )
        }
        val openUrl = item.canonicalUrl ?: item.url
        if (openUrl != null) {
            IconButton(onClick = { onOpenInBrowser(openUrl) }) {
                Icon(
                    imageVector = Icons.Filled.OpenInBrowser,
                    contentDescription = stringResource(Res.string.library_action_open_original),
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
        IconButton(onClick = onRearchive) {
            Icon(
                imageVector = Icons.Filled.Refresh,
                contentDescription = stringResource(Res.string.library_action_rearchive),
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }

    Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

    IndelibleButton(
        text = stringResource(Res.string.common_delete),
        onClick = onDelete,
        style = IndelibleButtonStyle.Destructive,
        modifier = Modifier.fillMaxWidth(),
    )

    Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

    IndelibleButton(
        text = stringResource(Res.string.library_action_open_reader),
        onClick = onOpenInReader,
        modifier = Modifier.fillMaxWidth(),
    )
}

@Composable
private fun PipelineStatusSection(
    item: ItemDetail,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier) {
        Text(
            text = stringResource(Res.string.library_pipeline_title),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(modifier = Modifier.height(IndelibleSpacing.step4))
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
        ) {
            val (statusLabel, statusColor) =
                when (item.pipelineStatus) {
                    "processing" ->
                        stringResource(Res.string.library_pipeline_processing) to MaterialTheme.colorScheme.primary
                    "ready" -> stringResource(Res.string.library_pipeline_ready) to MaterialTheme.colorScheme.primary
                    "failed" -> stringResource(Res.string.library_pipeline_failed) to MaterialTheme.colorScheme.error
                    else ->
                        stringResource(Res.string.library_pipeline_unknown) to
                            MaterialTheme.colorScheme.onSurfaceVariant
                }
            Text(
                text = statusLabel,
                style = MaterialTheme.typography.bodyMedium,
                color = statusColor,
            )
            if (item.pipelineStatus == "processing") {
                CircularProgressIndicator(
                    modifier = Modifier.size(IndelibleSpacing.step16),
                    strokeWidth = IndelibleSpacing.step2,
                )
            }
        }
        val pipelineError = item.pipelineError
        if (!pipelineError.isNullOrBlank()) {
            Spacer(modifier = Modifier.height(IndelibleSpacing.step4))
            Text(
                text = pipelineError,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.error,
            )
        }
    }
}

@Preview(showBackground = true)
@Composable
private fun PipelineStatusSectionPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            PipelineStatusSection(
                item = previewItemDetail(),
                modifier = Modifier.padding(IndelibleSpacing.step16),
            )
        }
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun PipelineStatusSectionPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            PipelineStatusSection(
                item = previewItemDetail(),
                modifier = Modifier.padding(IndelibleSpacing.step16),
            )
        }
    }
}

private fun previewItemDetail() =
    ItemDetail(
        id = "lib_preview1",
        documentId = "doc_preview1",
        itemType = "article",
        triageState = "inbox",
        isFavorite = true,
        isShortlisted = false,
        title = "The Future of Open-Source AI Models",
        excerpt = "A deep dive into what the next generation of open models will look like",
        url = "https://techcrunch.com/article",
        canonicalUrl = "https://techcrunch.com/article",
        domain = "techcrunch.com",
        author = "Sarah Chen",
        publishedAt = Instant.parse("2024-01-15T00:00:00Z"),
        language = "en",
        source = "url",
        savedAt = Instant.parse("2024-01-15T12:00:00Z"),
        createdAt = Instant.parse("2024-01-15T12:00:00Z"),
        updatedAt = Instant.parse("2024-01-15T12:00:00Z"),
    )
