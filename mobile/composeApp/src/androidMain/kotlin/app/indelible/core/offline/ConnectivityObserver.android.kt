package app.indelible.core.offline

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.distinctUntilChanged

actual class ConnectivityObserver(
    private val context: Context,
) {
    actual val online: Flow<Boolean> =
        callbackFlow {
            val manager = context.getSystemService(ConnectivityManager::class.java)
            if (manager == null) {
                trySend(true)
                awaitClose { }
                return@callbackFlow
            }
            val callback =
                object : ConnectivityManager.NetworkCallback() {
                    override fun onAvailable(network: Network) {
                        trySend(true)
                    }

                    override fun onLost(network: Network) {
                        trySend(false)
                    }
                }
            trySend(manager.activeNetwork != null)
            manager.registerDefaultNetworkCallback(callback)
            awaitClose { manager.unregisterNetworkCallback(callback) }
        }.distinctUntilChanged()
}
