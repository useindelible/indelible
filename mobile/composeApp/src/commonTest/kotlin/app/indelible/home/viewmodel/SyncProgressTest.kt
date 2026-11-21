package app.indelible.home.viewmodel

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class SyncProgressTest {
    private fun inputs(
        isOnline: Boolean = true,
        serverUp: Boolean = true,
        queued: Int = 0,
        attention: Int = 0,
        switchedScope: Boolean = false,
    ) = SyncInputs(isOnline, serverUp, queued, attention, switchedScope)

    private fun offlineWith(queued: Int): SyncProgress = SyncProgress().step(inputs(isOnline = false, queued = queued))

    @Test
    fun reconnectingSendsThenSyncsWhenEveryChangeLands() {
        val sending = offlineWith(2).step(inputs(queued = 2))
        assertEquals(ShelfNet.SENDING, sending.net)
        assertEquals(2, sending.sendTotal)

        val halfway = sending.step(inputs(queued = 1))
        assertEquals(1, halfway.sent)

        val done = halfway.step(inputs(queued = 0))
        assertEquals(ShelfNet.SYNCED, done.net)
        assertEquals(2, done.sendTotal)
        assertEquals(SYNCED_HOLD_MS, done.holdMillisAfter(halfway))
        assertEquals(ShelfNet.ONLINE, done.leave().net)
    }

    @Test
    fun newWorkDuringTheHoldEndsIt() {
        val synced = offlineWith(1).step(inputs(queued = 1)).step(inputs(queued = 0))
        assertEquals(ShelfNet.SYNCED, synced.net)

        assertEquals(ShelfNet.ONLINE, synced.step(inputs(queued = 1)).net)
        assertEquals(ShelfNet.ONLINE, synced.step(inputs(attention = 1)).net)
    }

    @Test
    fun reconnectingWithFailuresEndsOnlineNotSynced() {
        val sending = offlineWith(2).step(inputs(queued = 2))
        val oneSent = sending.step(inputs(queued = 1))

        val oneFailed = oneSent.step(inputs(queued = 0, attention = 1))

        assertEquals(ShelfNet.ONLINE, oneFailed.net)
        assertEquals(1, oneFailed.lastSent)
        assertEquals(1, oneFailed.attention)
    }

    @Test
    fun onlineWithoutAnAnswerIsUnreachableNotOffline() {
        val unreachable = SyncProgress().step(inputs(isOnline = true, serverUp = false, queued = 3))
        assertEquals(ShelfNet.OFFLINE, unreachable.net)
        assertTrue(unreachable.unreachable)
        assertEquals(3, unreachable.queued)

        val offline = SyncProgress().step(inputs(isOnline = false, serverUp = false, queued = 3))
        assertEquals(ShelfNet.OFFLINE, offline.net)
        assertFalse(offline.unreachable)
    }

    @Test
    fun lastSentSurvivesTheHoldWhileAttentionRemains() {
        val run =
            offlineWith(2)
                .step(inputs(queued = 2))
                .step(inputs(queued = 1))
                .step(inputs(queued = 0, attention = 1))

        val later = run.step(inputs(queued = 0, attention = 1)).leave()

        assertEquals(1, later.lastSent)
        assertEquals(1, later.attention)
    }

    @Test
    fun attentionClearedResetsLastSent() {
        val run = offlineWith(2).step(inputs(queued = 2)).step(inputs(queued = 0, attention = 1))

        val resolved = run.step(inputs(queued = 0, attention = 0))

        assertEquals(0, resolved.lastSent)
        assertEquals(0, resolved.attention)
    }

    @Test
    fun aScopeSwitchResetsTheRun() {
        val sending = offlineWith(2).step(inputs(queued = 2))

        val switched = sending.step(inputs(queued = 5, switchedScope = true))

        assertEquals(ShelfNet.ONLINE, switched.net)
        assertEquals(0, switched.sendTotal)
        assertEquals(0, switched.lastSent)
        assertEquals(5, switched.queued)
    }

    @Test
    fun aStalledSendHandsHomeBack() {
        val offline = offlineWith(1)
        val sending = offline.step(inputs(queued = 1))

        assertEquals(SEND_STALL_MS, sending.holdMillisAfter(offline))
        assertEquals(SEND_STALL_MS, sending.step(inputs(queued = 3)).holdMillisAfter(sending))
        assertNull(sending.step(inputs(queued = 1)).holdMillisAfter(sending))

        val handedBack = sending.leave()
        assertEquals(ShelfNet.ONLINE, handedBack.net)
        assertEquals(0, handedBack.sendTotal)
    }

    @Test
    fun aChangeMadeWhileSendingGrowsTheTotal() {
        val sending = offlineWith(2).step(inputs(queued = 2)).step(inputs(queued = 1))

        val grown = sending.step(inputs(queued = 2))

        assertEquals(3, grown.sendTotal)
        assertEquals(1, grown.sent)
    }

    @Test
    fun leavingWhileOfflineChangesNothing() {
        val offline = offlineWith(1)

        assertEquals(offline, offline.leave())
    }

    @Test
    fun aRunOverOldFailuresEndsOnlineNotSynced() {
        val offline = SyncProgress().step(inputs(isOnline = false, queued = 2, attention = 1))
        val sending = offline.step(inputs(queued = 2, attention = 1))
        assertEquals(ShelfNet.SENDING, sending.net)

        val done = sending.step(inputs(queued = 0, attention = 1))

        assertEquals(ShelfNet.ONLINE, done.net)
        assertEquals(2, done.lastSent)
        assertNull(done.holdMillisAfter(sending))
    }

    @Test
    fun aRowThatFailedMidRunIsNotCountedAsSent() {
        val sending = offlineWith(3).step(inputs(queued = 3))

        val oneFailed = sending.step(inputs(queued = 2, attention = 1))

        assertEquals(0, oneFailed.sent)
        assertEquals(1, oneFailed.step(inputs(queued = 1, attention = 1)).sent)
    }
}
