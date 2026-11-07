package app.indelible.library.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberUpdatedState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.drop
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge

/** The OS reporting the network back, or any request getting through to the server; the first values are a baseline. */
internal fun reachableAgain(
    online: Flow<Boolean>,
    reachable: Flow<Boolean?>,
): Flow<Unit> = merge(online.risingEdges(), reachable.map { it == true }.risingEdges())

private fun Flow<Boolean>.risingEdges(): Flow<Unit> =
    distinctUntilChanged()
        .drop(1)
        .filter { it }
        .map { }

@Composable
internal fun ReloadWhenReachable(
    online: Flow<Boolean>,
    reachable: Flow<Boolean?>,
    onBack: () -> Unit,
) {
    val latest by rememberUpdatedState(onBack)
    LaunchedEffect(online, reachable) {
        reachableAgain(online, reachable).collect { latest() }
    }
}
