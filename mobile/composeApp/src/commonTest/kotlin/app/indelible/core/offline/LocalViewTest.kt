package app.indelible.core.offline

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

private const val DOC = "doc_1"

internal fun outboxRow(
    seq: Long,
    payload: OutboxPayload,
    state: OutboxState = OutboxState.PENDING,
    superseded: Boolean = false,
): OutboxRow {
    val kind =
        when (payload) {
            is OutboxPayload.ReadingEvent -> OutboxKind.READING_EVENT
            is OutboxPayload.HighlightCreate -> OutboxKind.HIGHLIGHT_CREATE
            is OutboxPayload.HighlightColor -> OutboxKind.HIGHLIGHT_COLOR
            is OutboxPayload.HighlightNote -> OutboxKind.HIGHLIGHT_NOTE
            is OutboxPayload.HighlightTags -> OutboxKind.HIGHLIGHT_TAGS
            is OutboxPayload.HighlightDelete -> OutboxKind.HIGHLIGHT_DELETE
            is OutboxPayload.DocumentNote -> OutboxKind.DOCUMENT_NOTE
        }
    val entityId =
        when (payload) {
            is OutboxPayload.HighlightCreate -> payload.highlightId
            is OutboxPayload.HighlightColor -> payload.highlightId
            is OutboxPayload.HighlightNote -> payload.highlightId
            is OutboxPayload.HighlightTags -> payload.highlightId
            is OutboxPayload.HighlightDelete -> payload.highlightId
            is OutboxPayload.ReadingEvent, is OutboxPayload.DocumentNote -> DOC
        }
    return OutboxRow(
        seq = seq,
        scope = "scope",
        id = "row_$seq",
        kind = kind,
        entityId = entityId,
        documentId = DOC,
        payload = payload,
        createdAt = seq,
        attempts = 0,
        lastAttemptAt = null,
        nextAttemptAt = 0,
        state = state,
        lastError = null,
        superseded = superseded,
    )
}

internal fun progressEvent(
    originSeq: Long,
    basisPoints: Int?,
): OutboxPayload.ReadingEvent =
    OutboxPayload.ReadingEvent(
        eventId = "evt_$originSeq",
        originSeq = originSeq,
        kind = if (basisPoints == null) "opened" else "progress",
        progressBasisPoints = basisPoints,
        cause = "reader",
        sessionId = null,
        attempt = null,
        positionJson = null,
        assetKind = null,
        activeMs = null,
        recordedAtEpochMs = originSeq,
    )

internal fun serverHighlight(
    id: String,
    color: String = "yellow",
    tags: List<String> = emptyList(),
): CachedHighlight = CachedHighlight(id = id, documentId = DOC, color = color, textContent = "text $id", tags = tags)

class LocalViewTest {
    private fun changes(
        vararg rows: OutboxRow,
        ackedProgressSeq: Long = -1,
    ): LocalChanges = LocalChanges(DocumentRevision(0, 0, ackedProgressSeq), rows.toList())

    @Test
    fun liveRowsReplayInSeqOrderOverTheServerList() {
        val view =
            changes(
                outboxRow(1, OutboxPayload.HighlightCreate("hlt_new", "green", "fresh", null, null)),
                outboxRow(2, OutboxPayload.HighlightColor("hlt_1", "blue"), state = OutboxState.FAILED),
                outboxRow(3, OutboxPayload.HighlightNote("hlt_1", "mine")),
                outboxRow(4, OutboxPayload.HighlightTags("hlt_new", listOf("a")), state = OutboxState.BLOCKED),
                outboxRow(5, OutboxPayload.HighlightColor("hlt_1", "red"), superseded = true),
            ).highlights(listOf(serverHighlight("hlt_1")))

        assertEquals(listOf("hlt_1", "hlt_new"), view.map { it.id })
        assertEquals("blue", view[0].color)
        assertEquals("mine", view[0].note?.body)
        assertEquals(listOf("a"), view[1].tags)
        assertEquals(DOC, view[1].documentId)
    }

    @Test
    fun replayIsIdempotentForAnAlreadyAppliedRow() {
        val server = listOf(serverHighlight("hlt_1", color = "blue", tags = listOf("kept")))

        val view =
            changes(
                outboxRow(1, OutboxPayload.HighlightCreate("hlt_1", "yellow", "text hlt_1", null, null)),
                outboxRow(2, OutboxPayload.HighlightColor("hlt_1", "blue")),
            ).highlights(server)

        assertEquals(server, view)
    }

    @Test
    fun queuedDeleteIsNotResurrected() {
        val view =
            changes(outboxRow(1, OutboxPayload.HighlightDelete("hlt_1")))
                .highlights(listOf(serverHighlight("hlt_1"), serverHighlight("hlt_2")))

        assertEquals(listOf("hlt_2"), view.map { it.id })
    }

    @Test
    fun latestLiveNoteWinsOverTheServer() {
        assertEquals(
            "second",
            changes(
                outboxRow(1, OutboxPayload.DocumentNote("first", null), state = OutboxState.FAILED),
                outboxRow(2, OutboxPayload.DocumentNote("second", null)),
            ).note("server"),
        )
        assertEquals("server", changes().note("server"))
        assertNull(changes().note(null))
    }

    @Test
    fun eventsWithoutProgressAreIgnored() {
        val view =
            changes(
                outboxRow(1, progressEvent(originSeq = 4, basisPoints = 6_000)),
                outboxRow(2, progressEvent(originSeq = 5, basisPoints = null)),
            ).progress(Progress(percent = 20, maxPercent = 70))

        assertEquals(Progress(percent = 60, maxPercent = 70), view)
    }

    @Test
    fun progressAtOrBelowTheAcknowledgedSeqFallsBackToTheServer() {
        val server = Progress(percent = 20, maxPercent = 20)

        val event = outboxRow(1, progressEvent(originSeq = 3, basisPoints = 9_000))

        val view = changes(event, ackedProgressSeq = 3).progress(server)

        assertEquals(server, view)
        assertEquals(Progress(percent = 90, maxPercent = 90), changes(event).progress(server))
    }
}
