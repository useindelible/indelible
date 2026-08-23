package app.indelible.onboarding.ui

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Checkbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import app.indelible.onboarding.ui.components.StepCard
import app.indelible.onboarding.viewmodel.DEFAULT_SUGGESTED_FEEDS
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleTextField
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_continue
import indelible.composeapp.generated.resources.onboarding_feeds_custom
import indelible.composeapp.generated.resources.onboarding_feeds_subtitle
import indelible.composeapp.generated.resources.onboarding_feeds_suggested
import indelible.composeapp.generated.resources.onboarding_feeds_title
import indelible.composeapp.generated.resources.onboarding_feeds_url_label
import indelible.composeapp.generated.resources.onboarding_skip
import org.jetbrains.compose.resources.stringResource

@Composable
fun FeedsStep(
    selectedFeeds: Set<String>,
    rssUrlInput: String,
    onToggleFeed: (String) -> Unit,
    onRssUrlChange: (String) -> Unit,
    onContinue: () -> Unit,
    onSkip: () -> Unit,
    modifier: Modifier = Modifier,
) {
    StepCard(
        title = stringResource(Res.string.onboarding_feeds_title),
        subtitle = stringResource(Res.string.onboarding_feeds_subtitle),
        modifier = modifier,
    ) {
        Text(
            text = stringResource(Res.string.onboarding_feeds_suggested),
            style = MaterialTheme.typography.titleSmall,
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

        DEFAULT_SUGGESTED_FEEDS.forEach { feed ->
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Checkbox(
                    checked = selectedFeeds.contains(feed.url),
                    onCheckedChange = { onToggleFeed(feed.url) },
                )
                Spacer(modifier = Modifier.width(IndelibleSpacing.step8))
                Text(
                    text = feed.title,
                    style = MaterialTheme.typography.bodyLarge, // body: 15sp/400
                    modifier = Modifier.weight(1f),
                )
            }
            Text(
                text = stringResource(feed.descriptionRes),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.fillMaxWidth(),
            )
            Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
        }

        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

        Text(
            text = stringResource(Res.string.onboarding_feeds_custom),
            style = MaterialTheme.typography.titleSmall, // callout: 14sp/600
            color = MaterialTheme.colorScheme.onBackground,
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

        IndelibleTextField(
            value = rssUrlInput,
            onValueChange = onRssUrlChange,
            label = stringResource(Res.string.onboarding_feeds_url_label),
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step32))

        IndelibleButton(text = stringResource(Res.string.common_continue), onClick = onContinue)

        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

        TextButton(
            onClick = onSkip,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text(
                text = stringResource(Res.string.onboarding_skip),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}
