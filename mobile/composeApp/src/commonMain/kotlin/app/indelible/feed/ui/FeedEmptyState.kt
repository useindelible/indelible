package app.indelible.feed.ui

import androidx.compose.foundation.background
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
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.RssFeed
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.feed.viewmodel.FeedFilter
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.ZeroedGhostRows
import app.indelible.ui.components.dashedZeroBorder
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.feed_action_add
import indelible.composeapp.generated.resources.feed_empty_first_body
import indelible.composeapp.generated.resources.feed_empty_first_caption
import indelible.composeapp.generated.resources.feed_empty_first_kicker
import indelible.composeapp.generated.resources.feed_empty_first_title
import indelible.composeapp.generated.resources.feed_empty_seen_body
import indelible.composeapp.generated.resources.feed_empty_seen_caption
import indelible.composeapp.generated.resources.feed_empty_seen_kicker
import indelible.composeapp.generated.resources.feed_empty_seen_title
import indelible.composeapp.generated.resources.feed_empty_unseen_body
import indelible.composeapp.generated.resources.feed_empty_unseen_caption
import indelible.composeapp.generated.resources.feed_empty_unseen_kicker
import indelible.composeapp.generated.resources.feed_empty_unseen_title
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource

private data class EmptyFeedCopy(
    val kicker: StringResource,
    val title: StringResource,
    val body: StringResource,
    val caption: StringResource,
)

@Composable
internal fun FeedEmptyState(
    filter: FeedFilter,
    hasSubscriptions: Boolean,
    onAddFeed: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val copy = emptyFeedCopy(filter, hasSubscriptions)
    Column(
        modifier =
            modifier
                .fillMaxSize()
                .padding(
                    horizontal = IndelibleSpacing.rowPaddingH,
                    vertical = IndelibleSpacing.step8,
                ),
    ) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .dashedZeroBorder(MaterialTheme.colorScheme.outline)
                    .padding(IndelibleSpacing.step12),
            verticalAlignment = Alignment.Top,
        ) {
            Box(
                modifier =
                    Modifier
                        .size(IndelibleSpacing.step56)
                        .clip(IndelibleShape.lg)
                        .background(MaterialTheme.colorScheme.surfaceContainerHighest),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    imageVector = Icons.Filled.RssFeed,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Spacer(modifier = Modifier.width(IndelibleSpacing.step14))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = stringResource(copy.kicker),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Spacer(modifier = Modifier.height(IndelibleSpacing.step4))
                Text(
                    text = stringResource(copy.title),
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Spacer(modifier = Modifier.height(IndelibleSpacing.step6))
                Text(
                    text = stringResource(copy.body),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                if (!hasSubscriptions) {
                    Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
                    IndelibleButton(
                        text = stringResource(Res.string.feed_action_add),
                        onClick = onAddFeed,
                        compact = true,
                    )
                }
            }
        }
        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
        ZeroedGhostRows(
            borderColor = MaterialTheme.colorScheme.outline,
            lineColor = MaterialTheme.colorScheme.outlineVariant,
        )
        Spacer(modifier = Modifier.height(IndelibleSpacing.step12))
        Text(
            text = stringResource(copy.caption),
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.fillMaxWidth(),
        )
    }
}

private fun emptyFeedCopy(
    filter: FeedFilter,
    hasSubscriptions: Boolean,
): EmptyFeedCopy =
    when {
        !hasSubscriptions ->
            EmptyFeedCopy(
                kicker = Res.string.feed_empty_first_kicker,
                title = Res.string.feed_empty_first_title,
                body = Res.string.feed_empty_first_body,
                caption = Res.string.feed_empty_first_caption,
            )
        filter == FeedFilter.UNSEEN ->
            EmptyFeedCopy(
                kicker = Res.string.feed_empty_unseen_kicker,
                title = Res.string.feed_empty_unseen_title,
                body = Res.string.feed_empty_unseen_body,
                caption = Res.string.feed_empty_unseen_caption,
            )
        else ->
            EmptyFeedCopy(
                kicker = Res.string.feed_empty_seen_kicker,
                title = Res.string.feed_empty_seen_title,
                body = Res.string.feed_empty_seen_body,
                caption = Res.string.feed_empty_seen_caption,
            )
    }

@Preview
@Composable
private fun FeedEmptyStatePreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            FeedEmptyState(
                filter = FeedFilter.UNSEEN,
                hasSubscriptions = false,
                onAddFeed = {},
            )
        }
    }
}

@Preview
@Composable
private fun FeedEmptyStatePreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            FeedEmptyState(
                filter = FeedFilter.UNSEEN,
                hasSubscriptions = true,
                onAddFeed = {},
            )
        }
    }
}
