package app.indelible.feed.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.outlined.Upload
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
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
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import app.indelible.core.platform.rememberFilePicker
import app.indelible.feed.viewmodel.AddFeedEffect
import app.indelible.feed.viewmodel.AddFeedUiState
import app.indelible.feed.viewmodel.AddFeedViewModel
import app.indelible.profile.ui.components.SettingsSection
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleTextField
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import kotlinx.coroutines.launch

private const val DASHED_BORDER_DASH = 10f
private const val DASHED_BORDER_GAP = 7f
private const val COPY_FEEDBACK_DELAY_MS = 2000L

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AddFeedScreen(
    viewModel: AddFeedViewModel,
    ingestEmail: String?,
    onNavigateBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val uiState by viewModel.uiState.collectAsState()
    var rssUrl by remember { mutableStateOf("") }
    val snackbarHostState = remember { SnackbarHostState() }
    val coroutineScope = rememberCoroutineScope()
    val clipboardManager = LocalClipboardManager.current

    LaunchedEffect(viewModel) {
        viewModel.effects.collect { effect ->
            when (effect) {
                is AddFeedEffect.ShowSnackbar -> snackbarHostState.showSnackbar(effect.message)
                is AddFeedEffect.NavigateBack -> onNavigateBack()
            }
        }
    }

    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = "Add RSS Feed",
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
        Column(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(top = paddingValues.calculateTopPadding())
                    .verticalScroll(rememberScrollState()),
        ) {
            SettingsSection(title = "Subscribe to Feed") {
                Card(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .padding(horizontal = IndelibleSpacing.step16, vertical = IndelibleSpacing.step8),
                    shape = MaterialTheme.shapes.extraLarge,
                    colors =
                        CardDefaults.cardColors(
                            containerColor = MaterialTheme.colorScheme.surfaceVariant,
                        ),
                    border = BorderStroke(0.5.dp, MaterialTheme.colorScheme.outlineVariant),
                    elevation = CardDefaults.cardElevation(defaultElevation = 0.dp),
                ) {
                    Column(
                        modifier =
                            Modifier
                                .fillMaxWidth()
                                .padding(IndelibleSpacing.step16),
                        verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
                    ) {
                        IndelibleTextField(
                            value = rssUrl,
                            onValueChange = { rssUrl = it },
                            label = "Feed URL",
                            modifier = Modifier.fillMaxWidth(),
                        )
                        IndelibleButton(
                            text = "Subscribe",
                            onClick = { viewModel.subscribe(rssUrl) },
                            isLoading = uiState is AddFeedUiState.Loading,
                            enabled = rssUrl.isNotBlank(),
                        )
                    }
                }
            }

            SettingsSection(title = "Import OPML") {
                val opmlPicker =
                    rememberFilePicker(
                        mimeTypes = listOf("text/xml", "application/xml", "*/*"),
                    ) { bytes, name -> viewModel.importOpml(bytes, name) }
                OpmlDropZone(
                    isLoading = uiState is AddFeedUiState.OpmlLoading,
                    onClick = opmlPicker,
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .padding(horizontal = IndelibleSpacing.step16, vertical = IndelibleSpacing.step8),
                )
            }

            SettingsSection(title = "Newsletter Email") {
                Card(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .padding(horizontal = IndelibleSpacing.step16, vertical = IndelibleSpacing.step8),
                    shape = MaterialTheme.shapes.extraLarge,
                    colors =
                        CardDefaults.cardColors(
                            containerColor = MaterialTheme.colorScheme.surfaceVariant,
                        ),
                    border = BorderStroke(0.5.dp, MaterialTheme.colorScheme.outlineVariant),
                    elevation = CardDefaults.cardElevation(defaultElevation = 0.dp),
                ) {
                    Column(
                        modifier =
                            Modifier
                                .fillMaxWidth()
                                .padding(IndelibleSpacing.step16),
                        verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step10),
                    ) {
                        Text(
                            text = "Subscribe via email \u2014 newsletters sent here are saved as feed entries.",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        if (ingestEmail != null) {
                            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                            NewsletterIngestRow(
                                address = ingestEmail,
                                onCopy = {
                                    clipboardManager.setText(AnnotatedString(ingestEmail))
                                    coroutineScope.launch {
                                        snackbarHostState.showSnackbar("Email copied to clipboard")
                                    }
                                },
                            )
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.height(IndelibleSpacing.step32))
        }
    }
}

@Composable
private fun OpmlDropZone(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    isLoading: Boolean = false,
) {
    val borderColor = MaterialTheme.colorScheme.outlineVariant
    val shape = MaterialTheme.shapes.large

    Box(
        modifier =
            modifier
                .clip(shape)
                .clickable(enabled = !isLoading, onClick = onClick)
                .drawBehind {
                    val strokeWidth = 1.5.dp.toPx()
                    val dash = DASHED_BORDER_DASH
                    val gap = DASHED_BORDER_GAP
                    val stroke =
                        Stroke(
                            width = strokeWidth,
                            pathEffect = PathEffect.dashPathEffect(floatArrayOf(dash, gap), 0f),
                        )
                    drawRoundRect(
                        color = borderColor,
                        style = stroke,
                        cornerRadius = CornerRadius(12.dp.toPx()),
                    )
                }.padding(horizontal = IndelibleSpacing.step16, vertical = IndelibleSpacing.step24),
        contentAlignment = Alignment.Center,
    ) {
        if (isLoading) {
            CircularProgressIndicator(
                modifier = Modifier.size(IndelibleSpacing.step28),
                strokeWidth = IndelibleSpacing.step2,
            )
        } else {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step4),
            ) {
                Icon(
                    imageVector = Icons.Outlined.Upload,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
                    modifier = Modifier.size(IndelibleSpacing.step28),
                )
                Text(
                    text = "Select OPML file",
                    style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.Medium),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Text(
                    text = ".opml or .xml \u00b7 imports all feeds at once",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
                )
            }
        }
    }
}

@Composable
private fun NewsletterIngestRow(
    address: String,
    onCopy: () -> Unit,
    modifier: Modifier = Modifier,
) {
    var copied by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    Row(
        modifier = modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step10),
    ) {
        Text(
            text = "FEED",
            style = MaterialTheme.typography.bodySmall.copy(fontWeight = FontWeight.SemiBold),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.width(IndelibleSpacing.step48),
        )
        Text(
            text = address,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.weight(1f),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Surface(
            onClick = {
                onCopy()
                scope.launch {
                    copied = true
                    kotlinx.coroutines.delay(COPY_FEEDBACK_DELAY_MS)
                    copied = false
                }
            },
            modifier = Modifier.height(IndelibleSpacing.step28),
            shape = IndelibleShape.full,
            color = MaterialTheme.colorScheme.surface,
            border = BorderStroke(0.5.dp, MaterialTheme.colorScheme.outline),
        ) {
            Row(
                modifier = Modifier.padding(horizontal = IndelibleSpacing.step10),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = if (copied) "Copied" else "Copy",
                    style = MaterialTheme.typography.bodySmall.copy(fontWeight = FontWeight.SemiBold),
                )
            }
        }
    }
}
