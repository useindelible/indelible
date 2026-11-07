package app.indelible.core.offline

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import app.indelible.core.network.AuthenticatedApiTransport
import kotlinx.coroutines.flow.distinctUntilChanged

/** Every event that can make the outbox drainable asks for a drain; retry timing stays the worker's. */
@Composable
fun SyncDrainEffect(
    worker: OutboxWorker,
    connectivity: ConnectivityObserver,
    transport: AuthenticatedApiTransport,
) {
    LaunchedEffect(worker) { worker.requestDrain() }

    val lifecycleOwner = LocalLifecycleOwner.current
    DisposableEffect(lifecycleOwner, worker) {
        val observer =
            LifecycleEventObserver { _, event ->
                if (event == Lifecycle.Event.ON_START) worker.requestDrain()
            }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose { lifecycleOwner.lifecycle.removeObserver(observer) }
    }

    LaunchedEffect(connectivity, worker) {
        connectivity.online.distinctUntilChanged().collect(worker::setNetwork)
    }

    LaunchedEffect(transport, worker) {
        transport.requestSucceeded.collect { worker.requestDrain() }
    }
}
