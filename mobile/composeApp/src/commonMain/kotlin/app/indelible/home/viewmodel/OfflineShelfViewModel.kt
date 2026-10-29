package app.indelible.home.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.core.offline.CachedDocumentRow
import app.indelible.core.offline.CatalogEntry
import app.indelible.core.offline.KeptList
import app.indelible.core.offline.OfflineStore
import app.indelible.core.offline.OutboxRow
import app.indelible.core.offline.SessionRegistry
import app.indelible.core.offline.SnapshotRow
import app.indelible.core.offline.offlineJson
import app.indelible.home.model.HomeItem
import app.indelible.home.repository.HOME_CONTINUE_KEY
import app.indelible.home.repository.HOME_RECENT_KEY
import app.indelible.offline.viewmodel.ChangeStatus
import app.indelible.offline.viewmodel.pendingChanges
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

/** How Home stands with the network: the shelf shows for every state but [ONLINE]. */
enum class ShelfNet { OFFLINE, SENDING, SYNCED, ONLINE }

/**
 * [progressPercent] is 0..100, null when the item was never read; [imageUrl] is the item's own artwork.
 * [onPhone] is false for an item the server last listed that has no copy here, so it cannot open offline.
 */
data class ShelfItem(
    val documentId: String,
    val title: String,
    val documentType: String,
    val pinned: Boolean,
    val progressPercent: Int?,
    val imageUrl: String? = null,
    val domain: String? = null,
    val onPhone: Boolean = true,
)

/** [queued] changes still to send; [sendTotal] is the queue when the network came back, for "n of m". */
data class OfflineShelfState(
    val net: ShelfNet = ShelfNet.ONLINE,
    val items: List<ShelfItem> = emptyList(),
    val queued: Int = 0,
    val sendTotal: Int = 0,
    val continueReading: List<ShelfItem> = emptyList(),
    val recentlySaved: List<ShelfItem> = emptyList(),
    val loadedAt: Long? = null,
) {
    val sent: Int get() = (sendTotal - queued).coerceAtLeast(0)
}

internal const val SYNCED_HOLD_MS = 4_000L
internal const val SEND_STALL_MS = 20_000L
internal const val PROBE_FIRST_MS = 2_000L
internal const val PROBE_EVERY_MS = 30_000L

/**
 * What Home shows without a network: the copies on this phone, most recently opened first, and the
 * changes waiting to go. Once the network is back it follows the queue until it is empty, holds
 * "synced" for [SYNCED_HOLD_MS], then hands Home back to the server's dashboard. A queue that does not
 * move for [SEND_STALL_MS] (server down, rows retrying) hands Home back too; Storage keeps the rows.
 * Home is away once a request to the server got no answer ([reachable] is false), and the server is
 * asked again through [probe] until it answers. It works only while something watches [state].
 */
@OptIn(ExperimentalCoroutinesApi::class)
class OfflineShelfViewModel(
    private val registry: SessionRegistry,
    private val store: OfflineStore,
    reachable: Flow<Boolean?>,
    private val probe: suspend () -> Unit,
) : ViewModel() {
    private val _state = MutableStateFlow(OfflineShelfState())
    val state: StateFlow<OfflineShelfState> = _state.asStateFlow()

    private var timer: Job? = null
    private var scope: String? = null

    init {
        val online = reachable.map { it != false }.distinctUntilChanged()
        val watched = _state.subscriptionCount.map { it > 0 }.distinctUntilChanged()
        viewModelScope.launch {
            watched
                .flatMapLatest { if (it) combine(online, shelf()) { on, shelf -> on to shelf } else emptyFlow() }
                .collect { (isOnline, shelf) -> update(isOnline, shelf) }
        }
        viewModelScope.launch {
            watched.flatMapLatest { if (it) online else flowOf(true) }.collectLatest { if (!it) probeUntilAnswered() }
        }
    }

    private suspend fun probeUntilAnswered() {
        delay(PROBE_FIRST_MS)
        while (true) {
            probe()
            delay(PROBE_EVERY_MS)
        }
    }

    private fun update(
        isOnline: Boolean,
        shelf: Shelf,
    ) {
        val previous = _state.value
        val switched = shelf.scope != scope
        scope = shelf.scope
        val next =
            when {
                !isOnline -> OfflineShelfState(ShelfNet.OFFLINE, shelf.items, shelf.queued)
                switched -> OfflineShelfState(ShelfNet.ONLINE, shelf.items, shelf.queued)
                else -> previous.reconnected(shelf)
            }
        _state.value = next.copy(
            continueReading = shelf.continueReading,
            recentlySaved = shelf.recentlySaved,
            loadedAt = shelf.loadedAt,
        )
        when {
            next.net == ShelfNet.SYNCED && previous.net != ShelfNet.SYNCED -> leaveAfter(SYNCED_HOLD_MS)
            next.net == ShelfNet.SENDING && (previous.net != next.net || next.queued != previous.queued) ->
                leaveAfter(SEND_STALL_MS)
            next.net == ShelfNet.OFFLINE || next.net == ShelfNet.ONLINE -> timer?.cancel()
        }
    }

    private fun OfflineShelfState.reconnected(shelf: Shelf): OfflineShelfState =
        when {
            net == ShelfNet.ONLINE -> OfflineShelfState(ShelfNet.ONLINE, shelf.items, shelf.queued)
            net == ShelfNet.OFFLINE && shelf.queued > 0 ->
                OfflineShelfState(ShelfNet.SENDING, shelf.items, shelf.queued, shelf.queued)
            net == ShelfNet.OFFLINE -> OfflineShelfState(ShelfNet.ONLINE, shelf.items, shelf.queued)
            net == ShelfNet.SENDING && shelf.queued == 0 ->
                OfflineShelfState(ShelfNet.SYNCED, shelf.items, 0, sendTotal)
            net == ShelfNet.SENDING ->
                copy(
                    items = shelf.items,
                    queued = shelf.queued,
                    sendTotal = sendTotal + (shelf.queued - queued).coerceAtLeast(0),
                )
            else -> copy(items = shelf.items, queued = shelf.queued)
        }

    private fun leaveAfter(millis: Long) {
        timer?.cancel()
        timer =
            viewModelScope.launch {
                delay(millis)
                _state.update { if (it.net == ShelfNet.OFFLINE) it else it.copy(net = ShelfNet.ONLINE, sendTotal = 0) }
            }
    }

    private fun shelf(): Flow<Shelf> =
        registry.current
            .map { it.session?.scope }
            .distinctUntilChanged()
            .flatMapLatest { scope -> scope?.let(::shelfOf) ?: flowOf(Shelf()) }

    private fun shelfOf(scope: String): Flow<Shelf> =
        combine(
            store.observeCatalog(scope),
            store.observeOutbox(scope),
            store.observeList(scope, HOME_CONTINUE_KEY),
            store.observeList(scope, HOME_RECENT_KEY),
            ::ShelfInputs,
        ).mapLatest { inputs ->
            val catalog = inputs.catalog
            val copies = store.cachedDocuments(scope).associateBy { it.documentId }
            val entries = catalog.associateBy { it.documentId }
            Shelf(
                scope = scope,
                items = catalog.toItems(copies),
                queued = queued(inputs.rows, catalog),
                continueReading = inputs.continued.rows.toLastLoaded(entries, copies),
                recentlySaved = inputs.recent.rows.toLastLoaded(entries, copies),
                loadedAt = listOfNotNull(inputs.continued.seenAt, inputs.recent.seenAt).maxOrNull(),
            )
        }

    private data class ShelfInputs(
        val catalog: List<CatalogEntry>,
        val rows: List<OutboxRow>,
        val continued: KeptList,
        val recent: KeptList,
    )

    private data class Shelf(
        val scope: String? = null,
        val items: List<ShelfItem> = emptyList(),
        val queued: Int = 0,
        val continueReading: List<ShelfItem> = emptyList(),
        val recentlySaved: List<ShelfItem> = emptyList(),
        val loadedAt: Long? = null,
    )
}

private fun List<CatalogEntry>.toItems(copies: Map<String, CachedDocumentRow>): List<ShelfItem> =
    sortedByDescending { it.lastOpenedAt }.map {
        val copy = copies[it.documentId]
        val image = copy?.readerJson?.let(::shelfImageOf)
        ShelfItem(it.documentId, it.title, it.documentType, it.pinned, copy?.progressPercent, image)
    }

private fun List<SnapshotRow>.toLastLoaded(
    entries: Map<String, CatalogEntry>,
    copies: Map<String, CachedDocumentRow>,
): List<ShelfItem> =
    mapNotNull { row ->
        runCatching { offlineJson.decodeFromString(HomeItem.serializer(), row.rowJson) }.getOrNull()
    }.map { item ->
        val entry = entries[item.id]
        ShelfItem(
            documentId = item.id,
            title = item.title,
            documentType = item.itemType,
            pinned = entry?.pinned == true,
            progressPercent = copies[item.id]?.progressPercent ?: item.progressPercent?.toInt(),
            imageUrl = item.thumbnailUrl ?: item.leadImageUrl,
            domain = item.domain,
            onPhone = entry != null,
        )
    }

/** The thumbnail, else the lead image, of a copy's stored reader response; null when it has neither. */
internal fun shelfImageOf(readerJson: String): String? {
    val fields = runCatching { Json.parseToJsonElement(readerJson).jsonObject }.getOrNull() ?: return null
    return listOf("thumbnail_url", "lead_image_url")
        .firstNotNullOfOrNull { fields[it]?.jsonPrimitive?.contentOrNull }
}

/** Counted the way Storage lists them, so a long read is one change, not thousands. */
private fun queued(
    rows: List<OutboxRow>,
    catalog: List<CatalogEntry>,
): Int {
    val changes = pendingChanges(rows, catalog)
    return changes.count { it.status != ChangeStatus.SUPERSEDED && it.status != ChangeStatus.FAILED }
}
