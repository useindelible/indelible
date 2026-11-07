package app.indelible.core.offline

data class OutboxPlan(
    val admitted: List<OutboxRow>,
    val earliestWaitingDeadline: Long?,
)

/** A not-due row closes its entity for the rest of the pass, since a later row for it can't run first. */
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
