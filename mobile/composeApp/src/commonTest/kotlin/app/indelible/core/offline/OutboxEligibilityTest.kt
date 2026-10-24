package app.indelible.core.offline

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class OutboxEligibilityTest {
    private fun row(
        seq: Long,
        entityId: String,
        nextAttemptAt: Long,
    ): OutboxRow =
        OutboxRow(
            seq = seq,
            scope = "scope",
            id = "row_$seq",
            kind = OutboxKind.DOCUMENT_NOTE,
            entityId = entityId,
            documentId = entityId,
            payload = OutboxPayload.DocumentNote("body", null),
            createdAt = 0L,
            attempts = 0,
            lastAttemptAt = null,
            nextAttemptAt = nextAttemptAt,
            state = OutboxState.PENDING,
            lastError = null,
        )

    @Test
    fun dueHeadAdmitsContiguousDuePrefixAndFirstNotDueRowFeedsDeadline() {
        val rows = listOf(row(1, "a", 0), row(2, "a", 0), row(3, "a", 500), row(4, "a", 0))

        val plan = eligibleRows(rows, now = 100)

        assertEquals(listOf(1L, 2L), plan.admitted.map { it.seq })
        assertEquals(500L, plan.earliestWaitingDeadline)
    }

    @Test
    fun notDueHeadShieldsEveryLaterRowForTheEntityEvenWhenTheyAreDue() {
        val rows = listOf(row(1, "a", 900), row(2, "a", 0))

        val plan = eligibleRows(rows, now = 100)

        assertTrue(plan.admitted.isEmpty())
        assertEquals(900L, plan.earliestWaitingDeadline)
    }

    @Test
    fun entitiesAreIndependentAndDeadlineIsTheMinimumAcrossThem() {
        val rows = listOf(row(1, "a", 900), row(2, "b", 0), row(3, "b", 300), row(4, "c", 0))

        val plan = eligibleRows(rows, now = 100)

        assertEquals(listOf(2L, 4L), plan.admitted.map { it.seq })
        assertEquals(300L, plan.earliestWaitingDeadline)
    }

    @Test
    fun laterRowsOfAClosedEntityDoNotLowerTheDeadline() {
        val rows = listOf(row(1, "a", 900), row(2, "a", 200))

        val plan = eligibleRows(rows, now = 100)

        assertEquals(900L, plan.earliestWaitingDeadline)
    }

    @Test
    fun nothingWaitingGivesNullDeadline() {
        val plan = eligibleRows(listOf(row(1, "a", 0)), now = 100)

        assertEquals(listOf(1L), plan.admitted.map { it.seq })
        assertNull(plan.earliestWaitingDeadline)
    }

    @Test
    fun rowDueExactlyNowIsAdmitted() {
        val plan = eligibleRows(listOf(row(1, "a", 100)), now = 100)

        assertEquals(listOf(1L), plan.admitted.map { it.seq })
        assertNull(plan.earliestWaitingDeadline)
    }
}
