package app.indelible.auth.viewmodel

import app.indelible.core.offline.OfflineStore
import app.indelible.core.offline.OutboxWorker
import app.indelible.core.offline.ScopePurger
import app.indelible.core.offline.SqlDelightOfflineStore
import app.indelible.core.offline.testOutboxWorker
import app.indelible.core.offline.testScopePurger
import app.indelible.core.storage.TokenStorage
import app.indelible.db.testOfflineDatabase
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flowOf

/** An offline account whose store keeps no profile until a test signs in through it. */
fun testOfflineAccount(
    tokenStorage: TokenStorage,
    store: OfflineStore = SqlDelightOfflineStore(testOfflineDatabase()),
    worker: OutboxWorker = testOutboxWorker(),
    purger: ScopePurger = testScopePurger(),
    online: Flow<Boolean> = flowOf(true),
): OfflineAccount = OfflineAccount(store, tokenStorage, worker, purger, online)
