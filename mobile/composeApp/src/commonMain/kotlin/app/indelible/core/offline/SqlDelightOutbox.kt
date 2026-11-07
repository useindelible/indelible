package app.indelible.core.offline

import app.cash.sqldelight.coroutines.asFlow
import app.cash.sqldelight.coroutines.mapToList
import app.indelible.core.util.uuidV7
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.datetime.Clock
import kotlinx.serialization.encodeToString

internal class SqlDelightOutbox(
    private val context: SqlDelightStoreContext,
    private val failCreateHook: () -> Unit,
) : OutboxStore {
    override suspend fun <T> enqueue(
        session: Session,
        kind: OutboxKind,
        entityId: String,
        documentId: String,
        buildPayload: EnqueueTx.() -> Pair<OutboxPayload, T>,
    ): T =
        context.write {
            val scope = session.scope
            val tx = EnqueueTxImpl(scope, context.queries)
            try {
                context.database.transactionWithResult {
                    context.requireLive(session)
                    val (payload, result) = tx.buildPayload()
                    context.queries.insertOutbox(
                        scope = scope,
                        id = uuidV7(),
                        kind = kind.wireName(),
                        entity_id = entityId,
                        document_id = documentId,
                        payload_json = offlineJson.encodeToString(payload),
                        created_at = Clock.System.now().toEpochMilliseconds(),
                        state = initialState(scope, kind, entityId).wireName(),
                    )
                    context.queries.moveRevision(scope, documentId, payload)
                    result
                }
            } finally {
                tx.close()
            }
        }

    private fun initialState(
        scope: String,
        kind: OutboxKind,
        entityId: String,
    ): OutboxState =
        if (kind.dependsOnCreate() && context.queries.hasFailedCreate(scope, entityId).executeAsOne()) {
            OutboxState.BLOCKED
        } else {
            OutboxState.PENDING
        }

    override suspend fun pendingOrdered(scope: String): List<OutboxRow> =
        context.read {
            context.queries
                .pendingOrdered(scope)
                .executeAsList()
                .map(::outboxRowFrom)
        }

    override suspend fun rowsByState(
        scope: String,
        state: OutboxState,
    ): List<OutboxRow> =
        context.read {
            context.queries
                .outboxByState(scope, state.wireName())
                .executeAsList()
                .map(::outboxRowFrom)
        }

    override suspend fun remove(
        scope: String,
        id: String,
    ) {
        context.write {
            context.database.transaction {
                val row = context.queries.outboxRow(scope, id).executeAsOneOrNull()
                if (row != null) context.queries.acknowledge(outboxRowFrom(row))
            }
        }
    }

    override suspend fun markAttempt(
        scope: String,
        id: String,
        now: Long,
        nextAttemptAt: Long,
        error: String?,
    ) {
        context.write {
            context.queries.markAttempt(
                last_attempt_at = now,
                next_attempt_at = nextAttemptAt,
                last_error = error,
                scope = scope,
                id = id,
            )
        }
    }

    override suspend fun markFailed(
        scope: String,
        id: String,
        error: String?,
    ) {
        context.write {
            context.queries.setState(OutboxState.FAILED.wireName(), error, scope, id)
        }
    }

    override suspend fun failCreateAndBlockDependants(
        scope: String,
        id: String,
        entityId: String,
        error: String?,
    ) {
        context.write {
            context.database.transaction {
                context.queries.setState(OutboxState.FAILED.wireName(), error, scope, id)
                failCreateHook()
                context.queries.setStateForEntity(
                    OutboxState.BLOCKED.wireName(),
                    scope,
                    entityId,
                    OutboxState.PENDING.wireName(),
                )
            }
        }
    }

    override suspend fun retryRow(
        scope: String,
        id: String,
    ): Boolean =
        context.write {
            context.database.transactionWithResult {
                val row =
                    context.queries
                        .outboxRow(scope, id)
                        .executeAsOneOrNull()
                        ?.takeIf { it.state == OutboxState.FAILED.wireName() && it.superseded == 0L }
                if (row != null) {
                    context.queries.resetForRetry(scope, id)
                    context.queries.setStateForEntity(
                        OutboxState.PENDING.wireName(),
                        scope,
                        row.entity_id,
                        OutboxState.BLOCKED.wireName(),
                    )
                }
                row != null
            }
        }
}

internal class SqlDelightOutboxObservation(
    private val context: SqlDelightStoreContext,
) : OutboxObservation {
    override fun observeOutbox(scope: String): Flow<List<OutboxRow>> =
        context.queries
            .outboxForScope(scope)
            .asFlow()
            .mapToList(context.dispatcher)
            .map { rows -> rows.map(::outboxRowFrom) }

    override fun observeOutboxForDocument(
        scope: String,
        documentId: String,
    ): Flow<List<OutboxRow>> =
        context.queries
            .outboxForDocument(scope, documentId)
            .asFlow()
            .mapToList(context.dispatcher)
            .map { rows -> rows.map(::outboxRowFrom) }

    override fun observeDocumentSyncCounts(scope: String): Flow<Map<String, DocumentSyncCounts>> =
        context.queries
            .documentSyncCounts(scope)
            .asFlow()
            .mapToList(context.dispatcher)
            .map { rows ->
                rows.associate { row ->
                    row.document_id to
                        DocumentSyncCounts(
                            pending = row.pending.orZero(),
                            retrying = row.retrying.orZero(),
                            failed = row.failed.orZero(),
                            blocked = row.blocked.orZero(),
                        )
                }
            }
}

private fun Long?.orZero(): Int = this?.toInt() ?: 0

private fun OutboxKind.dependsOnCreate(): Boolean =
    when (this) {
        OutboxKind.HIGHLIGHT_COLOR,
        OutboxKind.HIGHLIGHT_NOTE,
        OutboxKind.HIGHLIGHT_TAGS,
        OutboxKind.HIGHLIGHT_DELETE,
        -> true
        OutboxKind.READING_EVENT,
        OutboxKind.HIGHLIGHT_CREATE,
        OutboxKind.DOCUMENT_NOTE,
        -> false
    }
