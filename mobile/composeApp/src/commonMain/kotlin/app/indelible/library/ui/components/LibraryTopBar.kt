package app.indelible.library.ui.components

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
import androidx.compose.material.icons.filled.FilterList
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
import app.indelible.core.model.LibraryCounts
import app.indelible.profile.ui.components.UserAvatar
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.library_item_count
import indelible.composeapp.generated.resources.library_open_navigation_cd
import indelible.composeapp.generated.resources.library_sort_filter_cd
import indelible.composeapp.generated.resources.library_your_library
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource
import app.indelible.ui.theme.geistMonoFontFamily

/**
 * Library header (prototype `.appbar` + `.head`): a slim action bar (menu, sort,
 * avatar) above an eyebrow + tappable scope chip. The chip shows the active scope
 * title, an optional count pill, and a chevron that rotates and fills with accent
 * while the scope-switcher popover is [popoverOpen]. Tapping the chip row calls
 * [onScopeClick] to toggle that popover.
 */
@Composable
fun LibraryTopBar(
    scopeTitle: String,
    scopeCount: Int?,
    counts: LibraryCounts?,
    popoverOpen: Boolean,
    userDisplayName: String?,
    avatarUrl: String?,
    avatarBytes: ByteArray?,
    onMenuClick: () -> Unit,
    onScopeClick: () -> Unit,
    onSortClick: () -> Unit,
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
                    contentDescription = stringResource(Res.string.library_open_navigation_cd),
                    tint = MaterialTheme.colorScheme.onSurface,
                )
            }
            Spacer(modifier = Modifier.weight(1f))
            IconButton(
                onClick = onSortClick,
                modifier = Modifier.size(IndelibleSpacing.step48),
            ) {
                Icon(
                    imageVector = Icons.Filled.FilterList,
                    contentDescription = stringResource(Res.string.library_sort_filter_cd),
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
                text = stringResource(Res.string.library_your_library),
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
                if (scopeCount != null) {
                    Spacer(modifier = Modifier.width(IndelibleSpacing.step12))
                    ScopeCountPill(count = scopeCount)
                }
                Spacer(modifier = Modifier.width(IndelibleSpacing.step12))
                ScopeChevron(open = popoverOpen)
            }
            if (counts != null) {
                Spacer(modifier = Modifier.height(IndelibleSpacing.step20))
                LibraryClearanceMeter(counts = counts)
            }
        }
    }
}

@Composable
private fun ScopeCountPill(count: Int) {
    Box(
        modifier =
            Modifier
                .clip(IndelibleShape.full)
                .background(MaterialTheme.colorScheme.primaryContainer)
                .padding(horizontal = IndelibleSpacing.step10, vertical = IndelibleSpacing.step4),
    ) {
        Text(
            text = pluralStringResource(Res.plurals.library_item_count, count, count),
            style = MaterialTheme.typography.bodyMedium.copy(fontFamily = geistMonoFontFamily()),
            color = MaterialTheme.colorScheme.primary,
        )
    }
}

@Composable
private fun ScopeChevron(open: Boolean) {
    val rotation by animateFloatAsState(
        targetValue = if (open) 180f else 0f,
        label = "scopeChevron",
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
private fun LibraryTopBarPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            LibraryTopBar(
                scopeTitle = "Inbox",
                scopeCount = 42,
                counts =
                    LibraryCounts(
                        total = 66,
                        unread = 42,
                        reading = 5,
                        done = 19,
                        byItemType = mapOf("article" to 38, "video" to 11),
                    ),
                popoverOpen = false,
                userDisplayName = "Sam",
                avatarUrl = null,
                avatarBytes = null,
                onMenuClick = {},
                onScopeClick = {},
                onSortClick = {},
                onProfileClick = {},
            )
        }
    }
}

@Preview
@Composable
private fun LibraryTopBarPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            LibraryTopBar(
                scopeTitle = "AI Research",
                scopeCount = 32,
                counts = null,
                popoverOpen = true,
                userDisplayName = "Sam",
                avatarUrl = null,
                avatarBytes = null,
                onMenuClick = {},
                onScopeClick = {},
                onSortClick = {},
                onProfileClick = {},
            )
        }
    }
}
