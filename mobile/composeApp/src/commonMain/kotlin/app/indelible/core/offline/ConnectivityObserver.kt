package app.indelible.core.offline

import kotlinx.coroutines.flow.Flow

/** Emits the current reachability first, then every change to it. */
expect class ConnectivityObserver {
    val online: Flow<Boolean>
}
