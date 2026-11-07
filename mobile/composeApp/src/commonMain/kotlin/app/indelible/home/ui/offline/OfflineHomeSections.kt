package app.indelible.home.ui.offline

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.unit.dp
import app.indelible.core.i18n.relativeTimeText
import app.indelible.home.viewmodel.OfflineShelfState
import app.indelible.home.viewmodel.ShelfItem
import app.indelible.home.viewmodel.ShelfNet
import app.indelible.ui.theme.MonochromeColors
import kotlinx.datetime.Clock
import kotlinx.datetime.Instant

internal enum class OfflineSection { LEDE, HERO, PICK_UP, CONTINUE, RECENT, SHELF, HINT, EMPTY_SHELF }

/** Top to bottom: what the server sent last sits between the hero and the shelf of copies. */
internal fun offlineSections(shelf: OfflineShelfState): List<OfflineSection> =
    buildList {
        add(OfflineSection.LEDE)
        add(OfflineSection.HERO)
        val hasCopies = shelf.items.isNotEmpty()
        if (hasCopies) add(OfflineSection.PICK_UP)
        if (shelf.continueReading.isNotEmpty()) add(OfflineSection.CONTINUE)
        if (shelf.recentlySaved.isNotEmpty()) add(OfflineSection.RECENT)
        if (hasCopies) {
            add(OfflineSection.SHELF)
            if (shelf.net == ShelfNet.OFFLINE) add(OfflineSection.HINT)
        } else if (shelf.net == ShelfNet.OFFLINE) {
            add(OfflineSection.EMPTY_SHELF)
        }
    }

/** A list as the server last sent it, on a ledge, with how long ago it arrived. */
@Composable
internal fun LastLoadedShelf(
    title: String,
    items: List<ShelfItem>,
    loadedAt: Long?,
    mono: MonochromeColors,
    onOpenItem: (String) -> Unit,
) {
    val stamp = loadedAt?.let { relativeTimeText(Instant.fromEpochMilliseconds(it), Clock.System.now()) }
    Column {
        SectionHead(title, stamp, mono)
        LazyRow(
            horizontalArrangement = Arrangement.spacedBy(18.dp),
            contentPadding = PaddingValues(start = 22.dp, end = 22.dp, top = LEDGE_TOP, bottom = 6.dp),
            modifier =
                Modifier.drawBehind {
                    val y = (LEDGE_TOP + STAND_HEIGHT).toPx()
                    val inset = 14.dp.toPx()
                    drawLine(mono.line2, Offset(inset, y), Offset(size.width - inset, y), 1.dp.toPx())
                },
        ) {
            items(items, key = { it.documentId }) { item ->
                LedgeObject(item, mono, onOpen = { onOpenItem(item.documentId) })
            }
        }
    }
}

private val LEDGE_TOP = 10.dp
