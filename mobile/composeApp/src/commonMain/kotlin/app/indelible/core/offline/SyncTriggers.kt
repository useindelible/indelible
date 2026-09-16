package app.indelible.core.offline

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import app.indelible.core.network.AuthenticatedApiTransport
import io.ktor.util.date.getTimeMillis
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.conflate
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

/**
 * Wires every event that can make the outbox drainable — app start, foreground, connectivity
 * returning, a successful request proving the server is reachable, and the earliest scheduled
 * retry coming due — to [OutboxWorker.drain], which is single-flight and so tolerates the overlap.
 */
@Composable
fun SyncDrainEffect(
    worker: OutboxWorker,
    connectivity: ConnectivityObserver,
    transport: AuthenticatedApiTransport,
) {
    val coroutineScope = rememberCoroutineScope()
    val retryJob = remember { mutableStateOf<Job?>(null) }
    val drainNow =
        remember(worker, coroutineScope) {
            {
                coroutineScope.launch {
                    worker.drain()
                    retryJob.value?.cancel()
                    retryJob.value = coroutineScope.launchRetryLoop(worker)
                }
                Unit
            }
        }

    LaunchedEffect(drainNow) { drainNow() }

    val lifecycleOwner = LocalLifecycleOwner.current
    DisposableEffect(lifecycleOwner, drainNow) {
        val observer =
            LifecycleEventObserver { _, event ->
                if (event == Lifecycle.Event.ON_START) drainNow()
            }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose { lifecycleOwner.lifecycle.removeObserver(observer) }
    }

    LaunchedEffect(connectivity, drainNow) {
        connectivity.online.distinctUntilChanged().collect { online ->
            if (online) drainNow()
        }
    }

    LaunchedEffect(transport, drainNow) {
        transport.requestSucceeded.conflate().collect { drainNow() }
    }
}

private fun CoroutineScope.launchRetryLoop(worker: OutboxWorker): Job =
    launch {
        while (isActive) {
            val retryAt = worker.nextRetryAt() ?: return@launch
            val waitMs = retryAt - getTimeMillis()
            if (waitMs > 0) delay(waitMs)
            worker.drain()
        }
    }
