package app.indelible.home.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.BookmarkBorder
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.ui.components.dashedZeroBorder
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing

@Composable
internal fun EmptyContinueReadingHero(modifier: Modifier = Modifier) {
    val borderColor = MaterialTheme.colorScheme.outline
    Column(
        modifier =
            modifier
                .fillMaxWidth()
                .clip(IndelibleShape.xxl)
                .background(MaterialTheme.colorScheme.surfaceContainerHigh)
                .dashedZeroBorder(borderColor)
                .padding(IndelibleSpacing.step20),
    ) {
        Text(
            text = "Nothing in progress",
            style = homeEyebrowStyle(),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(modifier = Modifier.height(IndelibleSpacing.step14))
        Row(
            horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step14),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Surface(
                shape = IndelibleShape.lg,
                color = MaterialTheme.colorScheme.surfaceContainerHighest,
                modifier = Modifier.size(IndelibleSpacing.step56),
            ) {
                Icon(
                    imageVector = Icons.Filled.BookmarkBorder,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(IndelibleSpacing.step16),
                )
            }
            Text(
                text = "Whatever you start reading waits for you here",
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onSurface,
                modifier = Modifier.weight(1f),
            )
        }
        Spacer(modifier = Modifier.height(IndelibleSpacing.step14))
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = "Resumes at the exact paragraph",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.weight(1f),
            )
            Spacer(
                modifier =
                    Modifier
                        .padding(start = IndelibleSpacing.step16)
                        .weight(1f)
                        .height(IndelibleSpacing.step6)
                        .clip(IndelibleShape.full)
                        .background(MaterialTheme.colorScheme.outlineVariant),
            )
        }
    }
}

@Composable
internal fun HomeZeroedSection(
    message: String,
    modifier: Modifier = Modifier,
) {
    Text(
        text = message,
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier =
            modifier
                .fillMaxWidth()
                .dashedZeroBorder(MaterialTheme.colorScheme.outline)
                .padding(IndelibleSpacing.step16),
    )
}

@Preview
@Composable
private fun EmptyContinueReadingHeroPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            Column(modifier = Modifier.padding(IndelibleSpacing.step16)) {
                EmptyContinueReadingHero()
                Spacer(modifier = Modifier.height(IndelibleSpacing.step16))
                HomeZeroedSection(message = "Nothing started yet. Half-read items show up here.")
            }
        }
    }
}

@Preview
@Composable
private fun EmptyContinueReadingHeroPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            Column(modifier = Modifier.padding(IndelibleSpacing.step16)) {
                EmptyContinueReadingHero()
                Spacer(modifier = Modifier.height(IndelibleSpacing.step16))
                HomeZeroedSection(message = "Nothing started yet. Half-read items show up here.")
            }
        }
    }
}
