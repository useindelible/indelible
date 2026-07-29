package app.indelible.feed.ui.components

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.Menu
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import app.indelible.profile.ui.components.UserAvatar
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.geistMonoFontFamily

/**
 * Feed header (prototype FRAME 3 `.appbar` + `.head`): the same slim action bar
 * the library uses, with a manage-sources gear in place of sort, above an eyebrow
 * and a tappable scope chip. The eyebrow reports the subscription count
 * ("Your feed · N sources"); the chip shows the active scope (Unseen / Seen) and a
 * chevron that fills with accent while [popoverOpen]. The prototype's unseen-count
 * pill and "updated / next refresh" timing line are intentionally dropped — the
 * client has no unseen-count or refresh-schedule data to back them.
 */
@Composable
fun FeedTopBar(
    scopeTitle: String,
    sourceCount: Int?,
    sourceCountExact: Boolean,
    popoverOpen: Boolean,
    userDisplayName: String?,
    avatarUrl: String?,
    avatarBytes: ByteArray?,
    onMenuClick: () -> Unit,
    onScopeClick: () -> Unit,
    onManageSources: () -> Unit,
    onProfileClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier =
            modifier
                .fillMaxWidth()
                .statusBarsPadding(),
    ) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = IndelibleSpacing.step8),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconButton(
                onClick = onMenuClick,
                modifier = Modifier.size(IndelibleSpacing.step48),
            ) {
                Icon(
                    imageVector = Icons.Filled.Menu,
                    contentDescription = "Open navigation",
                    tint = MaterialTheme.colorScheme.onSurface,
                )
            }
            Spacer(modifier = Modifier.weight(1f))
            IconButton(
                onClick = onManageSources,
                modifier = Modifier.size(IndelibleSpacing.step48),
            ) {
                Icon(
                    imageVector = IndelibleIcons.Settings,
                    contentDescription = "Manage sources",
                    tint = MaterialTheme.colorScheme.onSurface,
                )
            }
            Box(
                modifier =
                    Modifier
                        .size(IndelibleSpacing.step48)
                        .clip(CircleShape)
                        .clickable(onClick = onProfileClick),
                contentAlignment = Alignment.Center,
            ) {
                UserAvatar(
                    displayName = userDisplayName ?: "?",
                    size = IndelibleSpacing.step32,
                    textStyle = MaterialTheme.typography.titleSmall,
                    avatarUrl = avatarUrl,
                    avatarBytes = avatarBytes,
                )
            }
        }

        Column(
            modifier =
                Modifier.padding(
                    start = IndelibleSpacing.rowPaddingH,
                    end = IndelibleSpacing.rowPaddingH,
                    top = IndelibleSpacing.step8,
                    bottom = IndelibleSpacing.step16,
                ),
        ) {
            Text(
                text = feedEyebrow(sourceCount, sourceCountExact),
                style =
                    MaterialTheme.typography.labelSmall.copy(
                        fontFamily = geistMonoFontFamily(),
                        fontWeight = FontWeight.Medium,
                        letterSpacing = 0.16.em,
                    ),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
            Row(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .clip(IndelibleShape.md)
                        .clickable(onClick = onScopeClick),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = scopeTitle,
                    style = MaterialTheme.typography.headlineLarge,
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.weight(1f, fill = false),
                )
                Spacer(modifier = Modifier.width(IndelibleSpacing.step12))
                FeedScopeChevron(open = popoverOpen)
            }
        }
    }
}

private fun feedEyebrow(
    sourceCount: Int?,
    exact: Boolean,
): String {
    if (sourceCount == null) return "Your feed".uppercase()
    val suffix = if (exact) "" else "+"
    val noun = if (sourceCount == 1 && exact) "source" else "sources"
    return "Your feed · $sourceCount$suffix $noun".uppercase()
}

@Composable
private fun FeedScopeChevron(open: Boolean) {
    val rotation by animateFloatAsState(
        targetValue = if (open) 180f else 0f,
        label = "feedScopeChevron",
    )
    val borderColor = MaterialTheme.colorScheme.outline
    val background = if (open) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surfaceContainerHigh
    val tint = if (open) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurfaceVariant
    Box(
        modifier =
            Modifier
                .size(IndelibleSpacing.step28)
                .drawBehind {
                    // Closed-state hairline ring — a draw primitive, not a layout token.
                    if (!open) {
                        val sw = 1.dp.toPx()
                        drawCircle(
                            color = borderColor,
                            radius = (size.minDimension - sw) / 2f,
                            style = Stroke(width = sw),
                        )
                    }
                }
                .clip(CircleShape)
                .background(background),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            imageVector = Icons.Filled.KeyboardArrowDown,
            contentDescription = null,
            tint = tint,
            modifier =
                Modifier
                    .size(IndelibleSpacing.step20)
                    .rotate(rotation),
        )
    }
}

@Preview
@Composable
private fun FeedTopBarPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            FeedTopBar(
                scopeTitle = "Unseen",
                sourceCount = 8,
                sourceCountExact = true,
                popoverOpen = false,
                userDisplayName = "Sam",
                avatarUrl = null,
                avatarBytes = null,
                onMenuClick = {},
                onScopeClick = {},
                onManageSources = {},
                onProfileClick = {},
            )
        }
    }
}

@Preview
@Composable
private fun FeedTopBarPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            FeedTopBar(
                scopeTitle = "Seen",
                sourceCount = 50,
                sourceCountExact = false,
                popoverOpen = true,
                userDisplayName = "Sam",
                avatarUrl = null,
                avatarBytes = null,
                onMenuClick = {},
                onScopeClick = {},
                onManageSources = {},
                onProfileClick = {},
            )
        }
    }
}
