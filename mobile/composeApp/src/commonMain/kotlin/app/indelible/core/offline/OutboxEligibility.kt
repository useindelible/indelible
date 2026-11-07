package app.indelible.core.offline

data class OutboxPlan(
    val admitted: List<OutboxRow>,
    val earliestWaitingDeadline: Long?,
)

/**
 * Rows for one entity run strictly in seq order, so a not-due row closes its entity for the
 * pass: later rows for it are neither admitted nor counted toward the deadline, because none of
 * them can run before the one that closed it.
 */
fun eligibleRows(
    rows: List<OutboxRow>,
    now: Long,
): OutboxPlan {
    val admitted = mutableListOf<OutboxRow>()
    val closed = mutableSetOf<String>()
    var deadline: Long? = null
    for (row in rows) {
        if (row.entityId in closed) continue
        if (row.nextAttemptAt <= now) {
            admitted += row
        } else {
            closed += row.entityId
            deadline = minOf(deadline ?: row.nextAttemptAt, row.nextAttemptAt)
        }
    }
    return OutboxPlan(admitted, deadline)
}
