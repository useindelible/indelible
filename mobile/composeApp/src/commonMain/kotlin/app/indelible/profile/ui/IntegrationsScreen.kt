package app.indelible.profile.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import app.indelible.profile.ui.components.SettingsRow
import app.indelible.profile.ui.components.SettingsSection
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

private const val COPY_FEEDBACK_DELAY_MS = 2000L

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun IntegrationsScreen(
    ingestEmail: String?,
    ingestLibraryEmail: String?,
    onNavigateBack: () -> Unit,
    onNavigateToAddLibrary: () -> Unit,
    onNavigateToAddFeed: () -> Unit,
    onNavigateToFeeds: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = "Content",
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
    ) { paddingValues ->
        Column(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(top = paddingValues.calculateTopPadding())
                    .verticalScroll(rememberScrollState()),
        ) {
            SettingsSection(title = "Content") {
                SettingsRow(
                    label = "Add to Library",
                    sublabel = "Save a URL or paste article text",
                    onClick = onNavigateToAddLibrary,
                )
                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                SettingsRow(
                    label = "Add RSS Feed",
                    sublabel = "Subscribe to blogs and newsletters",
                    onClick = onNavigateToAddFeed,
                )
                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                SettingsRow(
                    label = "Manage Feeds",
                    sublabel = "Edit or remove feed subscriptions",
                    onClick = onNavigateToFeeds,
                )
            }

            SettingsSection(title = "Email Ingest") {
                IngestCard(
                    libraryEmail = ingestLibraryEmail,
                    feedEmail = ingestEmail,
                    modifier = Modifier.padding(
                        horizontal = IndelibleSpacing.step16,
                        vertical = IndelibleSpacing.step8,
                    ),
                )
            }

            Spacer(modifier = Modifier.height(IndelibleSpacing.step32))
        }
    }
}

@Composable
private fun IngestCard(
    libraryEmail: String?,
    feedEmail: String?,
    modifier: Modifier = Modifier,
) {
    val hasAnyEmail = libraryEmail != null || feedEmail != null

    Card(
        modifier = modifier.fillMaxWidth(),
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
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = "Ingest Addresses",
                    style = MaterialTheme.typography.titleLarge,
                )
                if (hasAnyEmail) {
                    SuccessBadge(label = "Active")
                }
            }

            Text(
                text = "Forward emails to save to Library, or route to Feed to subscribe as an entry.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )

            Column(verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step0)) {
                if (libraryEmail != null) {
                    IngestRow(label = "Library", address = libraryEmail)
                    HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                }
                if (feedEmail != null) {
                    IngestRow(label = "Feed", address = feedEmail)
                }
            }
        }
    }
}

@Composable
private fun IngestRow(
    label: String,
    address: String,
    modifier: Modifier = Modifier,
) {
    val clipboard = LocalClipboardManager.current
    val scope = rememberCoroutineScope()
    var copied by remember { mutableStateOf(false) }

    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .padding(vertical = IndelibleSpacing.step10),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step10),
    ) {
        Text(
            text = label.uppercase(),
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
            overflow = androidx.compose.ui.text.style.TextOverflow.Ellipsis,
        )
        CopyPillButton(
            label = if (copied) "Copied" else "Copy",
            onClick = {
                clipboard.setText(AnnotatedString(address))
                scope.launch {
                    copied = true
                    delay(COPY_FEEDBACK_DELAY_MS)
                    copied = false
                }
            },
        )
    }
}

@Composable
private fun CopyPillButton(
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        onClick = onClick,
        modifier = modifier.height(IndelibleSpacing.step28),
        shape = IndelibleShape.full,
        color = MaterialTheme.colorScheme.surface,
        border = BorderStroke(0.5.dp, MaterialTheme.colorScheme.outline),
    ) {
        Row(
            modifier = Modifier.padding(horizontal = IndelibleSpacing.step10),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = label,
                style = MaterialTheme.typography.bodySmall.copy(fontWeight = FontWeight.SemiBold),
            )
        }
    }
}

@Composable
private fun SuccessBadge(
    label: String,
    modifier: Modifier = Modifier,
) {
    val successColor = IndelibleTheme.colors.success
    Surface(
        modifier = modifier,
        shape = IndelibleShape.full,
        color = successColor.copy(alpha = 0.12f),
    ) {
        Text(
            text = label.uppercase(),
            style = MaterialTheme.typography.bodySmall.copy(fontWeight = FontWeight.SemiBold),
            color = successColor,
            modifier = Modifier.padding(horizontal = IndelibleSpacing.step10, vertical = IndelibleSpacing.step4),
        )
    }
}
