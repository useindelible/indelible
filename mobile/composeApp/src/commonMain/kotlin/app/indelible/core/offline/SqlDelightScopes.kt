package app.indelible.core.offline

import app.indelible.core.util.clientId

internal class SqlDelightScopes(
    private val context: SqlDelightStoreContext,
) : ScopeStore {
    override suspend fun quiesce() = context.quiesce()

    override suspend fun clientIdentity(scope: String): ClientIdentity =
        context.write {
            context.database.transactionWithResult {
                val existing = context.queries.getClientState(scope).executeAsOneOrNull()
                if (existing != null) {
                    ClientIdentity(clientId = existing.client_id, purgePending = existing.purge_pending != 0L)
                } else {
                    val newClientId = clientId()
                    context.queries.insertClientState(scope, newClientId, 0, 0)
                    ClientIdentity(clientId = newClientId, purgePending = false)
                }
            }
        }

    override suspend fun scopesWithState(): List<Pair<String, Boolean>> =
        context.read {
            context.queries
                .scopesWithState()
                .executeAsList()
                .map { it.scope to (it.purge_pending != 0L) }
        }

    override suspend fun setPurgePending(
        scope: String,
        pending: Boolean,
    ) {
        context.write {
            context.queries.setPurgePending(pending.toLong(), scope)
        }
    }

    override suspend fun purgeRows(scope: String) {
        context.write {
            context.queries.purgeRows(scope)
        }
    }

    override suspend fun finishPurge(scope: String) {
        context.write {
            context.queries.deleteClientState(scope)
        }
    }
}
