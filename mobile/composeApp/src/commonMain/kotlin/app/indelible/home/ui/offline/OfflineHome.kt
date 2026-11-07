package app.indelible.home.ui.offline

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import app.indelible.home.ui.components.greetingText
import app.indelible.home.ui.components.todayLabel
import app.indelible.home.viewmodel.Greeting
import app.indelible.home.viewmodel.OfflineShelfState
import app.indelible.home.viewmodel.ShelfItem
import app.indelible.home.viewmodel.ShelfNet
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import app.indelible.ui.theme.MonochromeColors
import app.indelible.ui.theme.OfflineIcons
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.home_offline_cached
import indelible.composeapp.generated.resources.home_offline_empty_next
import indelible.composeapp.generated.resources.home_offline_empty_shelf
import indelible.composeapp.generated.resources.home_offline_empty_step_keep
import indelible.composeapp.generated.resources.home_offline_empty_step_open
import indelible.composeapp.generated.resources.home_offline_empty_step_space
import indelible.composeapp.generated.resources.home_offline_hint
import indelible.composeapp.generated.resources.home_offline_kept
import indelible.composeapp.generated.resources.home_offline_shelf
import indelible.composeapp.generated.resources.home_offline_shelf_count
import indelible.composeapp.generated.resources.home_pick_up
import indelible.composeapp.generated.resources.home_progress_percent
import indelible.composeapp.generated.resources.home_resume
import indelible.composeapp.generated.resources.reader_type_article
import indelible.composeapp.generated.resources.reader_type_book
import indelible.composeapp.generated.resources.reader_type_pdf
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource

/**
 * Home without a network: the window-seat card, the item to pick up, and the shelf of copies on
 * this phone. With nothing on the phone it says what to do next time instead.
 */
@Composable
fun OfflineHome(
    shelf: OfflineShelfState,
    greeting: Greeting,
    firstName: String?,
    contentPadding: PaddingValues,
    onOpenItem: (String) -> Unit,
) {
    val mono = IndelibleTheme.monochrome
    val resume = shelf.items.firstOrNull()
    LazyColumn(
        modifier = Modifier.fillMaxSize().background(mono.canvas),
        contentPadding =
            PaddingValues(
                top = contentPadding.calculateTopPadding(),
                bottom = contentPadding.calculateBottomPadding() + IndelibleSpacing.step40,
            ),
    ) {
        item { Lede(greeting, firstName, mono) }
        item { OfflineHero(shelf, Modifier.padding(horizontal = IndelibleSpacing.step14)) }
        if (resume != null) {
            item { SectionHead(stringResource(Res.string.home_pick_up), null, mono) }
            item { ResumeCard(resume, mono, onOpen = { onOpenItem(resume.documentId) }) }
            item {
                val count = shelf.items.size
                val total = pluralStringResource(Res.plurals.home_offline_shelf_count, count, count)
                SectionHead(stringResource(Res.string.home_offline_shelf), total, mono)
            }
            item { Shelf(shelf.items, mono, onOpenItem) }
            if (shelf.net == ShelfNet.OFFLINE) item { Hint(mono) }
        } else if (shelf.net == ShelfNet.OFFLINE) {
            item { SectionHead(stringResource(Res.string.home_offline_empty_shelf), null, mono) }
            item { GhostShelf(mono) }
            item { SectionHead(stringResource(Res.string.home_offline_empty_next), null, mono) }
            item { Steps(mono) }
        }
    }
}

@Composable
private fun Lede(
    greeting: Greeting,
    firstName: String?,
    mono: MonochromeColors,
) {
    val title =
        buildAnnotatedString {
            append(greetingText(greeting))
            if (!firstName.isNullOrBlank()) {
                append(", ")
                val quiet = SpanStyle(fontStyle = FontStyle.Italic, fontWeight = FontWeight.Normal, color = mono.ink2)
                withStyle(quiet) { append(firstName) }
            }
        }
    Column(Modifier.padding(start = 22.dp, end = 22.dp, top = IndelibleSpacing.step8, bottom = 18.dp)) {
        Text(todayLabel(), style = mono(), color = mono.ink3)
        Text(
            title,
            style = serif(MaterialTheme.typography.displaySmall),
            color = mono.ink,
            modifier = Modifier.padding(top = IndelibleSpacing.step10),
        )
    }
}

@Composable
private fun SectionHead(
    title: String,
    count: String?,
    mono: MonochromeColors,
) {
    Row(
        verticalAlignment = Alignment.Bottom,
        horizontalArrangement = Arrangement.SpaceBetween,
        modifier =
            Modifier
                .fillMaxWidth()
                .padding(start = 22.dp, end = 22.dp, top = 30.dp, bottom = IndelibleSpacing.step12),
    ) {
        Text(title, style = serif(MaterialTheme.typography.titleLarge, FontWeight.Normal), color = mono.ink)
        if (count != null) Text(count, style = mono(), color = mono.ink3)
    }
}

@Composable
private fun kindLabel(documentType: String): String =
    when (documentType) {
        "book" -> stringResource(Res.string.reader_type_book)
        "pdf" -> stringResource(Res.string.reader_type_pdf)
        else -> stringResource(Res.string.reader_type_article)
    }

@Composable
private fun ResumeCard(
    item: ShelfItem,
    mono: MonochromeColors,
    onOpen: () -> Unit,
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step16),
        modifier =
            Modifier
                .padding(horizontal = IndelibleSpacing.step14)
                .fillMaxWidth()
                .polished(mono, IndelibleShape.xl)
                .clickable(onClick = onOpen)
                .padding(
                    start = IndelibleSpacing.step14,
                    end = IndelibleSpacing.step16,
                    top = IndelibleSpacing.step14,
                    bottom = IndelibleSpacing.step14,
                ),
    ) {
        ShelfCover(
            item.documentId,
            kindLabel(item.documentType),
            item.title,
            kept = false,
            width = 76.dp,
            height = 106.dp,
            imageUrl = item.imageUrl,
            tilt = RESUME_TILT,
        )
        ResumeDetails(item, mono, onOpen, Modifier.weight(1f))
    }
}

@Composable
private fun ResumeDetails(
    item: ShelfItem,
    mono: MonochromeColors,
    onOpen: () -> Unit,
    modifier: Modifier,
) {
    Column(modifier) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
            if (item.pinned) {
                Icon(OfflineIcons.Pin, null, tint = mono.success, modifier = Modifier.size(11.dp))
            }
            Text(
                stringResource(if (item.pinned) Res.string.home_offline_kept else Res.string.home_offline_cached),
                style = mono(),
                color = if (item.pinned) mono.success else mono.ink3,
            )
        }
        Text(
            item.title,
            style = serif(MaterialTheme.typography.titleLarge, FontWeight.Medium),
            color = mono.ink,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.padding(top = IndelibleSpacing.step6),
        )
        Text(
            kindLabel(item.documentType),
            style = MaterialTheme.typography.bodyMedium,
            color = mono.ink2,
            modifier = Modifier.padding(top = 3.dp),
        )
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.padding(top = IndelibleSpacing.step10),
        ) {
            val percent = item.progressPercent ?: 0
            ProgressTrack(percent / PERCENT, mono, Modifier.weight(1f))
            Text(
                stringResource(Res.string.home_progress_percent, percent),
                style = mono(),
                color = mono.ink2,
                maxLines = 1,
                softWrap = false,
                modifier = Modifier.padding(start = IndelibleSpacing.step10),
            )
            GoButton(mono, onOpen)
        }
    }
}

/** A 4dp track with the read part in ink and a bone thumb centred on its end. */
@Composable
private fun ProgressTrack(
    fraction: Float,
    mono: MonochromeColors,
    modifier: Modifier,
) {
    val thumb = bone(mono)
    Canvas(modifier.height(THUMB)) {
        val y = size.height / 2
        val bar = 4.dp.toPx()
        val r = THUMB.toPx() / 2
        val end = r + (size.width - 2 * r) * fraction.coerceIn(0f, 1f)
        drawLine(mono.track, Offset(0f, y), Offset(size.width, y), bar, StrokeCap.Round)
        drawLine(mono.ink, Offset(0f, y), Offset(end, y), bar, StrokeCap.Round)
        drawCircle(mono.track, r + 3.dp.toPx(), Offset(end, y))
        drawCircle(thumb, r, Offset(end, y))
    }
}

@Composable
private fun GoButton(
    mono: MonochromeColors,
    onOpen: () -> Unit,
) {
    Box(
        contentAlignment = Alignment.Center,
        modifier =
            Modifier
                .padding(start = IndelibleSpacing.step8)
                .size(40.dp)
                .clip(IndelibleShape.full)
                .background(bone(mono))
                .clickable(onClick = onOpen),
    ) {
        Icon(
            OfflineIcons.Play,
            contentDescription = stringResource(Res.string.home_resume),
            tint = mono.onAccent,
            modifier = Modifier.size(15.dp).padding(start = 2.dp),
        )
    }
}

@Composable
private fun Shelf(
    items: List<ShelfItem>,
    mono: MonochromeColors,
    onOpenItem: (String) -> Unit,
) {
    LazyRow(
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step14),
        contentPadding = PaddingValues(horizontal = 22.dp, vertical = IndelibleSpacing.step4),
    ) {
        items(items, key = { it.documentId }) { item ->
            Column(Modifier.width(BOOK_WIDTH).clickable { onOpenItem(item.documentId) }) {
                ShelfCover(
                    item.documentId,
                    kindLabel(item.documentType),
                    item.title,
                    kept = item.pinned,
                    width = BOOK_WIDTH,
                    height = BOOK_HEIGHT,
                    imageUrl = item.imageUrl,
                )
                Text(
                    item.title,
                    style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.Medium),
                    color = mono.ink,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(top = IndelibleSpacing.step10),
                )
                Text(
                    stringResource(if (item.pinned) Res.string.home_offline_kept else Res.string.home_offline_cached),
                    style = mono(),
                    color = if (item.pinned) mono.success else mono.ink3,
                    modifier = Modifier.padding(top = IndelibleSpacing.step4),
                )
            }
        }
    }
}

@Composable
private fun Hint(mono: MonochromeColors) {
    Row(
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
        modifier = Modifier.padding(start = 22.dp, end = 22.dp, top = IndelibleSpacing.step16),
    ) {
        Icon(
            OfflineIcons.Download,
            contentDescription = null,
            tint = mono.ink2,
            modifier = Modifier.size(17.dp).padding(top = 1.dp),
        )
        Text(
            stringResource(Res.string.home_offline_hint),
            style = MaterialTheme.typography.bodyMedium,
            color = mono.ink3,
        )
    }
}

@Composable
private fun GhostShelf(mono: MonochromeColors) {
    Row(
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step14),
        modifier = Modifier.padding(horizontal = 22.dp),
    ) {
        GHOST_FADES.forEachIndexed { index, alpha ->
            Box(
                contentAlignment = Alignment.Center,
                modifier =
                    Modifier.size(BOOK_WIDTH, BOOK_HEIGHT).drawBehind {
                        drawRoundRect(
                            mono.line2.copy(alpha = mono.line2.alpha * alpha),
                            cornerRadius = CornerRadius(6.dp.toPx()),
                            style = Stroke(1.dp.toPx(), pathEffect = dashes(4.dp.toPx(), 3.dp.toPx())),
                        )
                    },
            ) {
                if (index == 0) {
                    Icon(IndelibleIcons.Plus, null, tint = mono.ink3, modifier = Modifier.size(18.dp))
                }
            }
        }
    }
}

@Composable
private fun Steps(mono: MonochromeColors) {
    val steps =
        listOf(
            stringResource(Res.string.home_offline_empty_step_open),
            stringResource(Res.string.home_offline_empty_step_keep),
            stringResource(Res.string.home_offline_empty_step_space),
        )
    Column(
        Modifier
            .padding(horizontal = IndelibleSpacing.step14)
            .fillMaxWidth()
            .polished(mono, IndelibleShape.xl)
            .padding(horizontal = IndelibleSpacing.step16),
    ) {
        steps.forEachIndexed { index, step ->
            if (index > 0) Box(Modifier.fillMaxWidth().height(1.dp).background(mono.line))
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
                modifier = Modifier.padding(vertical = 13.dp),
            ) {
                Box(
                    Modifier.size(28.dp).polished(mono, IndelibleShape.full, mono.raised2),
                    contentAlignment = Alignment.Center,
                ) {
                    val numeral =
                        serif(MaterialTheme.typography.bodyMedium, FontWeight.Normal).copy(fontStyle = FontStyle.Italic)
                    Text((index + 1).toString(), style = numeral, color = mono.ink)
                }
                Text(step, style = MaterialTheme.typography.bodyLarge, color = mono.ink2)
            }
        }
    }
}

private val BOOK_WIDTH = 98.dp
private val BOOK_HEIGHT = 138.dp
private const val PERCENT = 100f
private const val GHOST_MID = 0.6f
private const val GHOST_FAR = 0.3f
private val GHOST_FADES = listOf(1f, GHOST_MID, GHOST_FAR)
private val THUMB = 11.dp
private const val RESUME_TILT = -3f

private fun dashes(
    on: Float,
    off: Float,
): PathEffect = PathEffect.dashPathEffect(floatArrayOf(on, off))
