package app.indelible.core.offline

class InjectedStoreFailure : Exception("injected store failure")

/** Throws on the first call of the named operation, then behaves like the delegate. */
private class OneShotFault {
    private var armed = true

    fun fire() {
        if (armed) {
            armed = false
            throw InjectedStoreFailure()
        }
    }
}

class ThrowingPendingReadStore(
    private val delegate: OfflineStore,
) : OfflineStore by delegate {
    private val fault = OneShotFault()

    override suspend fun pendingOrdered(scope: String): List<OutboxRow> {
        fault.fire()
        return delegate.pendingOrdered(scope)
    }
}

class ThrowingMarkAttemptStore(
    private val delegate: OfflineStore,
) : OfflineStore by delegate {
    private val fault = OneShotFault()

    override suspend fun markAttempt(
        scope: String,
        id: String,
        now: Long,
        nextAttemptAt: Long,
        error: String?,
    ) {
        fault.fire()
        delegate.markAttempt(scope, id, now, nextAttemptAt, error)
    }
}

class ThrowingClientIdentityStore(
    private val delegate: OfflineStore,
) : OfflineStore by delegate {
    private val fault = OneShotFault()

    override suspend fun clientIdentity(scope: String): ClientIdentity {
        fault.fire()
        return delegate.clientIdentity(scope)
    }
}
