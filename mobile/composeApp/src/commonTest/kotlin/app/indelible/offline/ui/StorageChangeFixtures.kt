package app.indelible.offline.ui

import app.indelible.core.offline.OutboxKind
import app.indelible.core.offline.SyncFailure
import app.indelible.offline.viewmodel.ChangeStatus
import app.indelible.offline.viewmodel.PendingChange

internal fun failed(
    kind: OutboxKind,
    failure: SyncFailure,
    detail: String? = null,
    blocked: List<PendingChange> = emptyList(),
) = PendingChange(
    id = "row_1",
    kind = kind,
    documentTitle = "The Patient Mind",
    status = ChangeStatus.FAILED,
    error = null,
    createdAt = 1L,
    detail = detail,
    blocked = blocked,
    failure = failure,
)

internal fun blockedColour(id: String) =
    PendingChange(id, OutboxKind.HIGHLIGHT_COLOR, "The Patient Mind", ChangeStatus.BLOCKED, null, 2L)
