package app.indelible.feed.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import app.indelible.feed.viewmodel.FeedFilter
import app.indelible.ui.platform.rememberHapticTick
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.geistMonoFontFamily
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.feed_filter_seen
import indelible.composeapp.generated.resources.feed_filter_show
import indelible.composeapp.generated.resources.feed_filter_unseen
import org.jetbrains.compose.resources.stringResource

/**
 * Feed scope-switcher popover: the same left-anchored "popIn" sheet the library
 * uses, narrowed to the feed's two scopes. It drops under the feed scope chip and
 * lists Unseen / Seen, rendering the row matching [currentFilter] with the accent
 * container. Selecting a row delegates to [onSelectFilter]; the scrim and row taps
 * dismiss via the callbacks, leaving open/close state owned by the caller through
 * [visible]. Mirrors [app.indelible.library.ui.components.ScopeSwitcherPopover] so
 * the two switchers read identically.
 */
private const val SLIDE_OFFSET_DIVISOR = 10

@Composable
fun FeedScopePopover(
    visible: Boolean,
    currentFilter: FeedFilter,
    onSelectFilter: (FeedFilter) -> Unit,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val scrimInteraction = remember { MutableInteractionSource() }
    Box(modifier = modifier.fillMaxSize()) {
        AnimatedVisibility(
            visible = visible,
            enter = fadeIn(),
            exit = fadeOut(),
            modifier = Modifier.fillMaxSize(),
        ) {
            Box(
                modifier =
                    Modifier
                        .fillMaxSize()
                        .background(MaterialTheme.colorScheme.scrim.copy(alpha = 0.5f))
                        .clickable(
                            interactionSource = scrimInteraction,
                            indication = null,
                            onClick = onDismiss,
                        ),
            )
        }

        AnimatedVisibility(
            visible = visible,
            enter =
                fadeIn() +
                    scaleIn(initialScale = 0.96f, transformOrigin = TransformOrigin(0f, 0f)) +
                    slideInVertically { full -> -full / SLIDE_OFFSET_DIVISOR },
            exit =
                fadeOut() +
                    scaleOut(targetScale = 0.96f, transformOrigin = TransformOrigin(0f, 0f)) +
                    slideOutVertically { full -> -full / SLIDE_OFFSET_DIVISOR },
            modifier =
                Modifier
                    .align(Alignment.TopStart)
                    .statusBarsPadding()
                    .padding(
                        start = IndelibleSpacing.step16,
                        top = IndelibleSpacing.step96 + IndelibleSpacing.step16,
                        bottom = IndelibleSpacing.step16,
                    ).fillMaxWidth(POPOVER_WIDTH_FRACTION),
        ) {
            Surface(
                shape = IndelibleShape.xxl,
                color = MaterialTheme.colorScheme.surfaceContainer,
                shadowElevation = IndelibleSpacing.step12,
            ) {
                Column(
                    modifier =
                        Modifier
                            .verticalScroll(rememberScrollState())
                            .padding(IndelibleSpacing.step10),
                ) {
                    FeedScopeLabel(stringResource(Res.string.feed_filter_show))
                    FeedScopeItem(
                        icon = IndelibleIcons.Inbox,
                        name = stringResource(Res.string.feed_filter_unseen),
                        active = currentFilter == FeedFilter.UNSEEN,
                        onClick = { onSelectFilter(FeedFilter.UNSEEN) },
                    )
                    FeedScopeItem(
                        icon = Icons.Filled.Check,
                        name = stringResource(Res.string.feed_filter_seen),
                        active = currentFilter == FeedFilter.SEEN,
                        onClick = { onSelectFilter(FeedFilter.SEEN) },
                    )
                }
            }
        }
    }
}

@Composable
private fun FeedScopeLabel(text: String) {
    Text(
        text = text,
        style =
            MaterialTheme.typography.labelSmall.copy(
                fontFamily = geistMonoFontFamily(),
                fontWeight = FontWeight.SemiBold,
                letterSpacing = 0.15.em,
            ),
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier =
            Modifier.padding(
                start = IndelibleSpacing.step12,
                end = IndelibleSpacing.step12,
                top = IndelibleSpacing.step10,
                bottom = IndelibleSpacing.step6,
            ),
    )
}

@Composable
private fun FeedScopeItem(
    icon: ImageVector,
    name: String,
    active: Boolean,
    onClick: () -> Unit,
) {
    val hapticTick = rememberHapticTick()
    val rowBackground = if (active) MaterialTheme.colorScheme.primaryContainer else Color.Transparent
    val nameColor = if (active) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onSurface
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .clip(IndelibleShape.xl)
                .background(rowBackground)
                .clickable(onClick = {
                    hapticTick()
                    onClick()
                })
                .padding(horizontal = IndelibleSpacing.step12, vertical = IndelibleSpacing.step10),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        FeedScopeIcon(icon = icon, active = active)
        Spacer(modifier = Modifier.width(IndelibleSpacing.step12))
        Text(
            text = name,
            style = MaterialTheme.typography.bodyLarge,
            fontWeight = if (active) FontWeight.SemiBold else FontWeight.Medium,
            color = nameColor,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.weight(1f),
        )
    }
}

@Composable
private fun FeedScopeIcon(
    icon: ImageVector,
    active: Boolean,
) {
    val borderColor = MaterialTheme.colorScheme.outline
    val boxBackground = if (active) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surfaceContainerHigh
    val iconTint = if (active) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurfaceVariant
    Box(
        modifier =
            Modifier
                .size(IndelibleSpacing.step32)
                .drawBehind {
                    // Neutral-state hairline ring — a draw primitive, not a layout token.
                    if (!active) {
                        val sw = 1.dp.toPx()
                        drawRoundRect(
                            color = borderColor,
                            topLeft = Offset(sw / 2f, sw / 2f),
                            size = Size(size.width - sw, size.height - sw),
                            cornerRadius = CornerRadius(IndelibleSpacing.step10.toPx()),
                            style = Stroke(width = sw),
                        )
                    }
                }.clip(IndelibleShape.md)
                .background(boxBackground),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = iconTint,
            modifier = Modifier.size(IndelibleSpacing.step16),
        )
    }
}

private const val POPOVER_WIDTH_FRACTION = 0.86f

@Preview
@Composable
private fun FeedScopePopoverPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface(modifier = Modifier.fillMaxSize()) {
            FeedScopePopover(
                visible = true,
                currentFilter = FeedFilter.UNSEEN,
                onSelectFilter = {},
                onDismiss = {},
            )
        }
    }
}

@Preview
@Composable
private fun FeedScopePopoverPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface(modifier = Modifier.fillMaxSize()) {
            FeedScopePopover(
                visible = true,
                currentFilter = FeedFilter.SEEN,
                onSelectFilter = {},
                onDismiss = {},
            )
        }
    }
}
