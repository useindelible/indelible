package app.indelible.core.offline

import app.indelible.share.isNetworkException
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.consumeEach
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/**
 * Owns the one coroutine that ever drains the outbox. Every trigger is [requestDrain]; requests
 * are conflated, so a burst costs one pass and a request that lands during a pass runs another.
 * A pass runs only while startup housekeeping is done, no session transition is open and the
 * session is not auth-paused; each of those has one owner, and clearing one asks for a drain
 * without touching the others.
 */
class OutboxWorker(
    store: OfflineStore,
    private val registry: SessionRegistry,
    sender: OutboxSender,
    private val clock: () -> Long,
    isNetwork: (Throwable) -> Boolean = ::isNetworkException,
    private val dispatcher: CoroutineDispatcher = Dispatchers.Default,
) {
    private val pass = OutboxPass(store, registry, sender, clock, isNetwork)
    private val requests = Channel<Unit>(Channel.CONFLATED)
    private val passGate = Mutex()
    private val workerScope = MutableStateFlow<CoroutineScope?>(null)
    private val passJob = MutableStateFlow<Job?>(null)
    private val timerJob = MutableStateFlow<Job?>(null)
    private val startupReady = MutableStateFlow(false)
    private val transitionCount = MutableStateFlow(0)
    private val authPausedState = MutableStateFlow(false)
    private var consecutiveFailures = 0
    val authPaused: StateFlow<Boolean> = authPausedState.asStateFlow()

    fun start() {
        check(workerScope.value == null) { "OutboxWorker is already started" }
        val owner = CoroutineScope(SupervisorJob() + dispatcher)
        workerScope.value = owner
        owner.launch { requests.consumeEach { serve(owner) } }
    }

    fun stop() {
        timerJob.value?.cancel()
        workerScope.value?.cancel()
        workerScope.value = null
    }

    fun requestDrain() {
        requests.trySend(Unit)
    }

    fun markStartupReady() {
        startupReady.value = true
        requestDrain()
    }

    /** Returns once no pass is running; passes stay blocked until [endTransition]. */
    suspend fun beginTransition() {
        transitionCount.update { it + 1 }
        passGate.withLock { passJob.value }?.join()
    }

    fun endTransition() {
        transitionCount.update { it - 1 }
        requestDrain()
    }

    fun resumeAuth() {
        authPausedState.value = false
        requestDrain()
    }

    private val runnable: Boolean
        get() = startupReady.value && transitionCount.value == 0 && !authPausedState.value

    private suspend fun serve(owner: CoroutineScope) {
        val job =
            passGate.withLock {
                if (!runnable) return
                owner.launch { runPass(owner) }.also { passJob.value = it }
            }
        job.join()
        passJob.compareAndSet(job, null)
    }

    private suspend fun runPass(owner: CoroutineScope) {
        val session = registry.current.value.session
        val outcome =
            if (session == null) {
                PassOutcome(PassResult.NoSession, null)
            } else {
                runCatching { pass.run(session) }.getOrElse { failure ->
                    if (failure is CancellationException) throw failure
                    PassOutcome(PassResult.Failed, null)
                }
            }
        consecutiveFailures = if (outcome.result == PassResult.Failed) consecutiveFailures + 1 else 0
        if (outcome.result == PassResult.AuthPaused) authPausedState.value = true
        // Rows passed over behind a terminal failure are still due; nothing else would re-run them.
        if (outcome.skippedDue) requestDrain()
        schedule(owner, outcome)
    }

    private fun schedule(
        owner: CoroutineScope,
        outcome: PassOutcome,
    ) {
        timerJob.value?.cancel()
        val deadline =
            when (val result = outcome.result) {
                is PassResult.RetryableStop -> listOfNotNull(result.nextAttemptAt, outcome.waitingDeadline).min()
                PassResult.Completed -> outcome.waitingDeadline
                PassResult.Failed -> clock() + backoffMs(consecutiveFailures)
                PassResult.AuthPaused, PassResult.Stale, PassResult.NoSession -> null
            } ?: return
        timerJob.value =
            owner.launch {
                delay((deadline - clock()).coerceAtLeast(0))
                requestDrain()
            }
    }
}
