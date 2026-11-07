package app.indelible.core.offline

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flowOf

actual class ConnectivityObserver {
    actual val online: Flow<Boolean> = flowOf(true)
}
