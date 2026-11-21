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
import io.ktor.http.parseUrl
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
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
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

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

/**
 * [queued] changes still to send; [sendTotal] is the queue when the network came back, for "n of m".
 * [unreachable] is a phone online while the server at [host] gives no answer, which reads differently from offline.
 */
data class OfflineShelfState(
    val net: ShelfNet = ShelfNet.ONLINE,
    val items: List<ShelfItem> = emptyList(),
    val queued: Int = 0,
    val sendTotal: Int = 0,
    val continueReading: List<ShelfItem> = emptyList(),
    val recentlySaved: List<ShelfItem> = emptyList(),
    val loadedAt: Long? = null,
    val unreachable: Boolean = false,
    val host: String? = null,
) {
    val sent: Int get() = (sendTotal - queued).coerceAtLeast(0)
}

internal const val PROBE_FIRST_MS = 2_000L
internal const val PROBE_EVERY_MS = 30_000L

/**
 * What Home shows without a network: the copies on this phone, most recently opened first, and the
 * changes waiting to go. Once the network is back it follows the queue until it is empty, holds
 * "synced" for [SYNCED_HOLD_MS], then hands Home back to the server's dashboard. A queue that does not
 * move for [SEND_STALL_MS] (server down, rows retrying) hands Home back too; Storage keeps the rows.
 * Home is away once a request to the server got no answer ([reachable] is false), and the server is
 * asked again through [probe] until it answers. The same run is published as [progress] for the Library
 * banner. It works only while something watches [state] or [progress].
 */
@OptIn(ExperimentalCoroutinesApi::class)
class OfflineShelfViewModel(
    private val registry: SessionRegistry,
    private val store: OfflineStore,
    reachable: Flow<Boolean?>,
    private val probe: suspend () -> Unit,
    online: Flow<Boolean> = flowOf(true),
    private val requestDrain: () -> Unit = {},
) : ViewModel() {
    private val _state = MutableStateFlow(OfflineShelfState())
    val state: StateFlow<OfflineShelfState> = _state.asStateFlow()

    private val _progress = MutableStateFlow(SyncProgress())
    val progress: StateFlow<SyncProgress> = _progress.asStateFlow()

    /** The signed-in server's host name, for "Can't reach <host>"; null when signed out. */
    val host: StateFlow<String?> =
        registry.current
            .map { it.session?.origin?.let(::hostOf) }
            .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private var timer: Job? = null
    private var retry: Job? = null
    private var scope: String? = null

    init {
        val serverUp = reachable.map { it != false }.distinctUntilChanged()
        val withItems = _state.subscriptionCount.map { it > 0 }.distinctUntilChanged()
        val watched =
            combine(_state.subscriptionCount, _progress.subscriptionCount) { a, b -> a + b > 0 }.distinctUntilChanged()
        viewModelScope.launch {
            watched
                .flatMapLatest { if (it) combine(online, serverUp, shelf(withItems), ::Triple) else emptyFlow() }
                .collect { (isOnline, up, shelf) -> update(isOnline, up, shelf) }
        }
        viewModelScope.launch {
            watched
                .flatMapLatest { if (it) combine(online, serverUp, ::Pair) else flowOf(false to true) }
                .collectLatest { (isOnline, up) -> if (isOnline && !up) probeUntilAnswered() }
        }
    }

    /** Clears the backoff of every queued row, asks the server again, then sends what is queued. */
    fun retryNow() {
        if (retry?.isActive == true) return
        retry =
            viewModelScope.launch {
                scope?.let { store.resetBackoff(it) }
                requestDrain()
                probe()
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
        serverUp: Boolean,
        shelf: Shelf,
    ) {
        val previous = _progress.value
        val switched = shelf.scope != scope
        scope = shelf.scope
        val next = previous.step(SyncInputs(isOnline, serverUp, shelf.queued, shelf.attention, switched))
        publish(next, shelf)
        when (val hold = next.holdMillisAfter(previous)) {
            null -> if (next.net == ShelfNet.OFFLINE || next.net == ShelfNet.ONLINE) timer?.cancel()
            else -> leaveAfter(hold)
        }
    }

    private fun publish(
        progress: SyncProgress,
        shelf: Shelf,
    ) {
        _progress.value = progress
        _state.value =
            OfflineShelfState(
                net = progress.net,
                items = shelf.items,
                queued = progress.queued,
                sendTotal = progress.sendTotal,
                continueReading = shelf.continueReading,
                recentlySaved = shelf.recentlySaved,
                loadedAt = shelf.loadedAt,
                unreachable = progress.unreachable,
                host = host.value,
            )
    }

    private fun leaveAfter(millis: Long) {
        timer?.cancel()
        timer =
            viewModelScope.launch {
                delay(millis)
                val left = _progress.value.leave()
                _progress.value = left
                _state.value = _state.value.copy(net = left.net, sendTotal = left.sendTotal)
            }
    }

    private fun shelf(withItems: Flow<Boolean>): Flow<Shelf> =
        registry.current
            .map { it.session?.scope }
            .distinctUntilChanged()
            .flatMapLatest { scope -> scope?.let { shelfOf(it, withItems) } ?: flowOf(Shelf()) }

    // The banner only needs the counts; the copies' reader JSON is parsed only while Home shows the items.
    private fun shelfOf(
        scope: String,
        withItems: Flow<Boolean>,
    ): Flow<Shelf> =
        combine(
            store.observeCatalog(scope),
            store.observeOutbox(scope),
            store.observeList(scope, HOME_CONTINUE_KEY),
            store.observeList(scope, HOME_RECENT_KEY),
            withItems,
            ::ShelfInputs,
        ).mapLatest { inputs ->
            val catalog = inputs.catalog
            val counts = queueCounts(inputs.rows, catalog)
            if (!inputs.withItems) return@mapLatest Shelf(scope, queued = counts.queued, attention = counts.attention)
            val copies = store.cachedDocuments(scope).associateBy { it.documentId }
            val entries = catalog.associateBy { it.documentId }
            Shelf(
                scope = scope,
                items = catalog.toItems(copies),
                queued = counts.queued,
                attention = counts.attention,
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
        val withItems: Boolean,
    )

    private data class Shelf(
        val scope: String? = null,
        val items: List<ShelfItem> = emptyList(),
        val queued: Int = 0,
        val attention: Int = 0,
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

/** The server as the user named it: the host, plus the port when it is not the scheme's own. */
internal fun hostOf(origin: String): String? {
    val url = parseUrl(origin) ?: return null
    val port = url.specifiedPort
    return if (port == 0 || port == url.protocol.defaultPort) url.host else "${url.host}:$port"
}

/** The thumbnail, else the lead image, of a copy's stored reader response; null when it has neither. */
internal fun shelfImageOf(readerJson: String): String? {
    val fields = runCatching { Json.parseToJsonElement(readerJson).jsonObject }.getOrNull() ?: return null
    return listOf("thumbnail_url", "lead_image_url")
        .firstNotNullOfOrNull { fields[it]?.jsonPrimitive?.contentOrNull }
}

private data class QueueCounts(
    val queued: Int,
    val attention: Int,
)

/** Counted the way Storage lists them, so a long read is one change, not thousands. */
private fun queueCounts(
    rows: List<OutboxRow>,
    catalog: List<CatalogEntry>,
): QueueCounts {
    val changes = pendingChanges(rows, catalog)
    val every = changes.flatMap { listOf(it) + it.blocked }
    return QueueCounts(
        queued = every.count { it.status != ChangeStatus.SUPERSEDED && it.status != ChangeStatus.FAILED },
        attention = every.count { it.status == ChangeStatus.FAILED },
    )
}
