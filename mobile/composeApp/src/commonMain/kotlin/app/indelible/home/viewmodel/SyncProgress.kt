package app.indelible.home.viewmodel

/** How Home stands with the network: the shelf shows for every state but [ONLINE]. */
enum class ShelfNet { OFFLINE, SENDING, SYNCED, ONLINE }

internal const val SYNCED_HOLD_MS = 3_000L
internal const val SEND_STALL_MS = 20_000L

/**
 * One catch-up as the Home hero and the Library banner both see it. [unreachable] is a phone online while
 * the server gives no answer; [lastSent] is what the last run sent, kept while [attention] (failures) remains.
 */
data class SyncProgress(
    val net: ShelfNet = ShelfNet.ONLINE,
    val queued: Int = 0,
    val sendTotal: Int = 0,
    val attention: Int = 0,
    val unreachable: Boolean = false,
    val lastSent: Int = 0,
    internal val attentionAtStart: Int = 0,
) {
    val sent: Int get() = (sendTotal - queued).coerceAtLeast(0)
}

data class SyncInputs(
    val isOnline: Boolean,
    val serverUp: Boolean,
    val queued: Int,
    val attention: Int,
    val switchedScope: Boolean,
)

fun SyncProgress.step(inputs: SyncInputs): SyncProgress {
    val next =
        when {
            !inputs.isOnline || !inputs.serverUp ->
                SyncProgress(
                    net = ShelfNet.OFFLINE,
                    queued = inputs.queued,
                    attention = inputs.attention,
                    unreachable = inputs.isOnline,
                    lastSent = lastSent,
                )
            inputs.switchedScope -> SyncProgress(ShelfNet.ONLINE, inputs.queued, attention = inputs.attention)
            else -> reconnected(inputs)
        }
    return if (next.attention == 0) next.copy(lastSent = 0) else next
}

private fun SyncProgress.reconnected(inputs: SyncInputs): SyncProgress =
    when {
        net == ShelfNet.ONLINE -> online(inputs)
        net == ShelfNet.OFFLINE && inputs.queued > 0 ->
            SyncProgress(
                net = ShelfNet.SENDING,
                queued = inputs.queued,
                sendTotal = inputs.queued,
                attention = inputs.attention,
                lastSent = lastSent,
                attentionAtStart = inputs.attention,
            )
        net == ShelfNet.OFFLINE -> online(inputs)
        net == ShelfNet.SENDING && inputs.queued == 0 -> finished(inputs.attention)
        net == ShelfNet.SENDING ->
            copy(
                queued = inputs.queued,
                sendTotal = sendTotal + (inputs.queued - queued).coerceAtLeast(0),
                attention = inputs.attention,
            )
        else -> copy(queued = inputs.queued, attention = inputs.attention)
    }

private fun SyncProgress.online(inputs: SyncInputs): SyncProgress =
    SyncProgress(ShelfNet.ONLINE, inputs.queued, attention = inputs.attention, lastSent = lastSent)

// A run that failed anything never reads "all caught up": it ends online, and the failures carry the count.
private fun SyncProgress.finished(attention: Int): SyncProgress {
    val failed = (attention - attentionAtStart).coerceAtLeast(0)
    return if (failed > 0) {
        SyncProgress(ShelfNet.ONLINE, attention = attention, lastSent = sendTotal - failed)
    } else {
        SyncProgress(ShelfNet.SYNCED, sendTotal = sendTotal, attention = attention, lastSent = sendTotal)
    }
}

/** How long the new state is held before [leave], or null when nothing new is to be armed. */
fun SyncProgress.holdMillisAfter(previous: SyncProgress): Long? =
    when {
        net == ShelfNet.SYNCED && previous.net != ShelfNet.SYNCED -> SYNCED_HOLD_MS
        net == ShelfNet.SENDING && (previous.net != ShelfNet.SENDING || queued != previous.queued) -> SEND_STALL_MS
        else -> null
    }

fun SyncProgress.leave(): SyncProgress =
    if (net == ShelfNet.OFFLINE) this else copy(net = ShelfNet.ONLINE, sendTotal = 0)
