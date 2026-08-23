package app.indelible.feed.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.MoreHoriz
import androidx.compose.material.icons.filled.Pause
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.feed.model.FeedSource
import app.indelible.feed.model.FeedSubscription
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.feed_action_edit
import indelible.composeapp.generated.resources.feed_action_more_cd
import indelible.composeapp.generated.resources.feed_action_pause_cd
import indelible.composeapp.generated.resources.feed_action_resume_cd
import indelible.composeapp.generated.resources.feed_action_unsubscribe
import indelible.composeapp.generated.resources.feed_status_active
import indelible.composeapp.generated.resources.feed_status_error
import indelible.composeapp.generated.resources.feed_status_paused
import kotlinx.datetime.Instant
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource

@Composable
fun SubscriptionRow(
    subscription: FeedSubscription,
    onToggleStatus: () -> Unit,
    onToggleAutoSave: () -> Unit,
    onEdit: () -> Unit,
    onDelete: () -> Unit,
    modifier: Modifier = Modifier,
) {
    var menuExpanded by remember { mutableStateOf(false) }
    val isActive = subscription.status == "active"
    val isError = subscription.status == "error"

    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .padding(
                    horizontal = IndelibleSpacing.step16,
                    vertical = IndelibleSpacing.step12,
                ),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
    ) {
        FeedBubble(
            name = subscription.titleOverride ?: subscription.source.name,
            status = subscription.status,
        )

        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = subscription.titleOverride ?: subscription.source.name,
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.onSurface,
            )
            subscription.source.domain?.let { domain ->
                Text(
                    text = domain,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }

        StatusBadge(status = subscription.status)

        IconButton(
            onClick = onToggleStatus,
            enabled = !isError,
            modifier = Modifier.size(IndelibleSpacing.step40),
        ) {
            Icon(
                imageVector = if (isActive) Icons.Filled.Pause else Icons.Filled.PlayArrow,
                contentDescription =
                    stringResource(
                        if (isActive) Res.string.feed_action_pause_cd else Res.string.feed_action_resume_cd,
                    ),
                tint =
                    when {
                        isError -> MaterialTheme.colorScheme.error
                        else -> MaterialTheme.colorScheme.onSurfaceVariant
                    },
                modifier = Modifier.size(IndelibleSpacing.step20),
            )
        }

        Switch(
            checked = subscription.autoSave,
            onCheckedChange = { onToggleAutoSave() },
        )

        Box {
            IconButton(onClick = { menuExpanded = true }) {
                Icon(
                    imageVector = Icons.Filled.MoreHoriz,
                    contentDescription = stringResource(Res.string.feed_action_more_cd),
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            DropdownMenu(
                expanded = menuExpanded,
                onDismissRequest = { menuExpanded = false },
            ) {
                DropdownMenuItem(
                    text = { Text(stringResource(Res.string.feed_action_edit)) },
                    onClick = {
                        menuExpanded = false
                        onEdit()
                    },
                )
                DropdownMenuItem(
                    text = {
                        Text(
                            text = stringResource(Res.string.feed_action_unsubscribe),
                            color = MaterialTheme.colorScheme.error,
                        )
                    },
                    onClick = {
                        menuExpanded = false
                        onDelete()
                    },
                )
            }
        }
    }
}

@Composable
private fun FeedBubble(
    name: String,
    status: String,
    modifier: Modifier = Modifier,
) {
    val initials = feedInitials(name)
    val isError = status == "error"
    val bgColor =
        if (isError) {
            MaterialTheme.colorScheme.error.copy(alpha = 0.10f)
        } else {
            MaterialTheme.colorScheme.primaryContainer
        }
    val textColor =
        if (isError) {
            MaterialTheme.colorScheme.error
        } else {
            MaterialTheme.colorScheme.onPrimaryContainer
        }

    Box(
        modifier =
            modifier
                .size(IndelibleSpacing.step40)
                .background(bgColor, MaterialTheme.shapes.medium),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = initials,
            style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Bold),
            color = textColor,
        )
    }
}

private fun feedInitials(name: String): String {
    if (name.isBlank()) return "?"
    val stopwords = setOf("the", "a", "an")
    val words = name.trim().split("\\s+".toRegex()).filter { it.isNotBlank() }
    val meaningful = words.filter { it.lowercase() !in stopwords }
    return when {
        meaningful.isEmpty() -> name.take(2).uppercase()
        meaningful.size == 1 -> meaningful[0].take(2).uppercase()
        else -> (meaningful[0].take(1) + meaningful[1].take(1)).uppercase()
    }
}

@Composable
private fun StatusBadge(
    status: String,
    modifier: Modifier = Modifier,
) {
    val bgColor =
        when (status) {
            "active" -> IndelibleTheme.colors.success.copy(alpha = 0.12f)
            "error" -> MaterialTheme.colorScheme.error.copy(alpha = 0.12f)
            else -> MaterialTheme.colorScheme.surfaceContainerHigh
        }
    val textColor =
        when (status) {
            "active" -> IndelibleTheme.colors.success
            "error" -> MaterialTheme.colorScheme.error
            else -> MaterialTheme.colorScheme.onSurfaceVariant
        }
    val label = stringResource(statusLabelRes(status))

    Box(
        modifier =
            modifier
                .background(bgColor, IndelibleShape.full)
                .padding(
                    horizontal = IndelibleSpacing.step8,
                    vertical = IndelibleSpacing.step2,
                ),
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.labelSmall,
            color = textColor,
        )
    }
}

private fun statusLabelRes(status: String): StringResource =
    when (status) {
        "active" -> Res.string.feed_status_active
        "error" -> Res.string.feed_status_error
        else -> Res.string.feed_status_paused
    }

@Preview(showBackground = true)
@Composable
private fun SubscriptionRowPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            Column {
                SubscriptionRow(
                    subscription = previewSubscription(status = "active"),
                    onToggleStatus = {},
                    onToggleAutoSave = {},
                    onEdit = {},
                    onDelete = {},
                )
                SubscriptionRow(
                    subscription = previewSubscription(status = "paused"),
                    onToggleStatus = {},
                    onToggleAutoSave = {},
                    onEdit = {},
                    onDelete = {},
                )
                SubscriptionRow(
                    subscription = previewSubscription(status = "error"),
                    onToggleStatus = {},
                    onToggleAutoSave = {},
                    onEdit = {},
                    onDelete = {},
                )
            }
        }
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun SubscriptionRowPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            Column {
                SubscriptionRow(
                    subscription = previewSubscription(status = "active"),
                    onToggleStatus = {},
                    onToggleAutoSave = {},
                    onEdit = {},
                    onDelete = {},
                )
                SubscriptionRow(
                    subscription = previewSubscription(status = "error"),
                    onToggleStatus = {},
                    onToggleAutoSave = {},
                    onEdit = {},
                    onDelete = {},
                )
            }
        }
    }
}

private fun previewSubscription(status: String = "active") =
    FeedSubscription(
        id = "sub-1",
        inputUrl = "https://blog.jetbrains.com/kotlin/feed",
        titleOverride = null,
        autoSave = false,
        status = status,
        source =
            FeedSource(
                id = "src-1",
                name = "Kotlin Blog",
                url = "https://blog.jetbrains.com/kotlin/feed",
                pollUrl = "https://blog.jetbrains.com/kotlin/feed",
                domain = "blog.jetbrains.com",
                imageUrl = null,
                consecutiveFailures = 0,
                isResolvable = true,
                popularity = 0,
                sourceKind = "rss",
                visibility = "public",
            ),
        createdAt = Instant.parse("2026-03-20T10:00:00Z"),
        updatedAt = Instant.parse("2026-03-20T10:00:00Z"),
    )
