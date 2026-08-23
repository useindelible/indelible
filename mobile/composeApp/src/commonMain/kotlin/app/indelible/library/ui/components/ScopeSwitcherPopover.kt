package app.indelible.library.ui.components

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
import androidx.compose.material3.HorizontalDivider
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
import app.indelible.library.viewmodel.LibraryScope
import app.indelible.library.viewmodel.TriageFilter
import app.indelible.sidebar.model.Collection
import app.indelible.sidebar.model.SmartList
import app.indelible.ui.platform.rememberHapticTick
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import app.indelible.ui.theme.geistMonoFontFamily
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.library_scope_archive
import indelible.composeapp.generated.resources.library_scope_inbox
import indelible.composeapp.generated.resources.library_scope_later
import indelible.composeapp.generated.resources.library_views
import indelible.composeapp.generated.resources.sidebar_collections
import indelible.composeapp.generated.resources.sidebar_smart_lists
import org.jetbrains.compose.resources.stringResource

/**
 * Scope-switcher popover (prototype `.scope-pop`): a left-anchored sheet that drops
 * under the library scope chip with a scale + fade "popIn". It lists the triage
 * Views (Inbox / Later / Archive) and, when present, the user's Collections and
 * Smart Lists. The row matching [currentScope] (plus [currentTriage] for the Views
 * section) renders active with the accent container. Selecting a row delegates to
 * the matching `onSelect*` callback; the scrim and any row tap dismiss via the
 * callbacks, leaving open/close state owned by the caller through [visible].
 */
private const val SLIDE_OFFSET_DIVISOR = 10

@Composable
fun ScopeSwitcherPopover(
    visible: Boolean,
    currentScope: LibraryScope,
    currentTriage: TriageFilter,
    collections: List<Collection>,
    smartLists: List<SmartList>,
    onSelectTriage: (TriageFilter) -> Unit,
    onSelectCollection: (Collection) -> Unit,
    onSelectSmartList: (SmartList) -> Unit,
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
            ScopePopoverCard(
                currentScope = currentScope,
                currentTriage = currentTriage,
                collections = collections,
                smartLists = smartLists,
                onSelectTriage = onSelectTriage,
                onSelectCollection = onSelectCollection,
                onSelectSmartList = onSelectSmartList,
            )
        }
    }
}

@Composable
private fun ScopePopoverCard(
    currentScope: LibraryScope,
    currentTriage: TriageFilter,
    collections: List<Collection>,
    smartLists: List<SmartList>,
    onSelectTriage: (TriageFilter) -> Unit,
    onSelectCollection: (Collection) -> Unit,
    onSelectSmartList: (SmartList) -> Unit,
) {
    val banners = IndelibleTheme.colors.collectionBanners
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
            ScopePopLabel(stringResource(Res.string.library_views))
            ScopePopItem(
                icon = IndelibleIcons.Inbox,
                name = stringResource(Res.string.library_scope_inbox),
                active = currentScope is LibraryScope.Triage && currentTriage == TriageFilter.INBOX,
                onClick = { onSelectTriage(TriageFilter.INBOX) },
            )
            ScopePopItem(
                icon = IndelibleIcons.Clock,
                name = stringResource(Res.string.library_scope_later),
                active = currentScope is LibraryScope.Triage && currentTriage == TriageFilter.LATER,
                onClick = { onSelectTriage(TriageFilter.LATER) },
            )
            ScopePopItem(
                icon = IndelibleIcons.Archive,
                name = stringResource(Res.string.library_scope_archive),
                active = currentScope is LibraryScope.Triage && currentTriage == TriageFilter.ARCHIVE,
                onClick = { onSelectTriage(TriageFilter.ARCHIVE) },
            )

            if (collections.isNotEmpty()) {
                ScopePopSeparator()
                ScopePopLabel(stringResource(Res.string.sidebar_collections))
                collections.forEachIndexed { index, collection ->
                    ScopePopItem(
                        icon = IndelibleIcons.Folder,
                        name = collection.name,
                        active = currentScope is LibraryScope.Collection && currentScope.id == collection.id,
                        onClick = { onSelectCollection(collection) },
                        count = collection.itemCount.toInt(),
                        iconTint = banners[index % banners.size],
                    )
                }
            }

            if (smartLists.isNotEmpty()) {
                ScopePopSeparator()
                ScopePopLabel(stringResource(Res.string.sidebar_smart_lists))
                smartLists.forEach { smartList ->
                    ScopePopItem(
                        icon = IndelibleIcons.SmartList,
                        name = smartList.name,
                        active = currentScope is LibraryScope.SmartList && currentScope.id == smartList.id,
                        onClick = { onSelectSmartList(smartList) },
                    )
                }
            }
        }
    }
}

@Composable
private fun ScopePopLabel(text: String) {
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
private fun ScopePopItem(
    icon: ImageVector,
    name: String,
    active: Boolean,
    onClick: () -> Unit,
    count: Int? = null,
    iconTint: Color? = null,
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
        ScopePopIcon(icon = icon, active = active, tint = iconTint)
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
        if (count != null) {
            Spacer(modifier = Modifier.width(IndelibleSpacing.step8))
            ScopePopCount(count = count, active = active)
        }
    }
}

@Composable
private fun ScopePopIcon(
    icon: ImageVector,
    active: Boolean,
    tint: Color?,
) {
    val borderColor = MaterialTheme.colorScheme.outline
    val drawBorder = !active && tint == null
    val boxBackground =
        when {
            active -> MaterialTheme.colorScheme.primary
            tint != null -> tint.copy(alpha = 0.18f)
            else -> MaterialTheme.colorScheme.surfaceContainerHigh
        }
    val iconTint =
        when {
            active -> MaterialTheme.colorScheme.onPrimary
            tint != null -> tint
            else -> MaterialTheme.colorScheme.onSurfaceVariant
        }
    Box(
        modifier =
            Modifier
                .size(IndelibleSpacing.step32)
                .drawBehind {
                    // Neutral-state hairline ring — a draw primitive, not a layout token.
                    if (drawBorder) {
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

@Composable
private fun ScopePopCount(
    count: Int,
    active: Boolean,
) {
    val background =
        if (active) {
            MaterialTheme.colorScheme.primaryContainer
        } else {
            MaterialTheme.colorScheme.surfaceContainerHigh
        }
    val foreground = if (active) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
    Box(
        modifier =
            Modifier
                .clip(IndelibleShape.full)
                .background(background)
                .padding(horizontal = IndelibleSpacing.step8, vertical = IndelibleSpacing.step2),
    ) {
        Text(
            text = count.toString(),
            style = MaterialTheme.typography.labelSmall.copy(fontFamily = geistMonoFontFamily()),
            color = foreground,
        )
    }
}

@Composable
private fun ScopePopSeparator() {
    HorizontalDivider(
        modifier = Modifier.padding(horizontal = IndelibleSpacing.step10, vertical = IndelibleSpacing.step8),
        color = MaterialTheme.colorScheme.outlineVariant,
    )
}

private const val POPOVER_WIDTH_FRACTION = 0.86f

private fun previewCollection(
    id: String,
    name: String,
    itemCount: Long,
) = Collection(
    id = id,
    name = name,
    itemCount = itemCount,
    sortOrder = 0,
    `object` = "collection",
    createdAt = kotlinx.datetime.Instant.parse("2024-01-01T00:00:00Z"),
    updatedAt = kotlinx.datetime.Instant.parse("2024-01-01T00:00:00Z"),
)

private fun previewSmartList(
    id: String,
    name: String,
) = SmartList(
    id = id,
    name = name,
    filterExpression = kotlinx.serialization.json.JsonObject(emptyMap()),
    isPinned = false,
    `object` = "smart_list",
    createdAt = kotlinx.datetime.Instant.parse("2024-01-01T00:00:00Z"),
    updatedAt = kotlinx.datetime.Instant.parse("2024-01-01T00:00:00Z"),
)

@Preview
@Composable
private fun ScopeSwitcherPopoverPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface(modifier = Modifier.fillMaxSize()) {
            ScopeSwitcherPopover(
                visible = true,
                currentScope = LibraryScope.Triage,
                currentTriage = TriageFilter.INBOX,
                collections =
                    listOf(
                        previewCollection("col_1", "Design", 14),
                        previewCollection("col_2", "AI Research", 32),
                    ),
                smartLists = listOf(previewSmartList("sl_1", "Unread")),
                onSelectTriage = {},
                onSelectCollection = {},
                onSelectSmartList = {},
                onDismiss = {},
            )
        }
    }
}

@Preview
@Composable
private fun ScopeSwitcherPopoverPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface(modifier = Modifier.fillMaxSize()) {
            ScopeSwitcherPopover(
                visible = true,
                currentScope = LibraryScope.Collection("col_2", "AI Research"),
                currentTriage = TriageFilter.INBOX,
                collections =
                    listOf(
                        previewCollection("col_1", "Design", 14),
                        previewCollection("col_2", "AI Research", 32),
                    ),
                smartLists = listOf(previewSmartList("sl_1", "Unread")),
                onSelectTriage = {},
                onSelectCollection = {},
                onSelectSmartList = {},
                onDismiss = {},
            )
        }
    }
}
