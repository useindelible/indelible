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

/**
 * The OS reporting the network back, or any request getting through to the server; the first values are a baseline.
 * A screen that shows [stale] content takes a server that already answers as a reason to reload at once.
 */
internal fun reachableAgain(
    online: Flow<Boolean>,
    reachable: Flow<Boolean?>,
    stale: Boolean = false,
): Flow<Unit> = merge(online.risingEdges(), reachable.map { it == true }.risingEdges(withCurrent = stale))

private fun Flow<Boolean>.risingEdges(withCurrent: Boolean = false): Flow<Unit> =
    distinctUntilChanged()
        .drop(if (withCurrent) 0 else 1)
        .filter { it }
        .map { }

@Composable
internal fun ReloadWhenReachable(
    online: Flow<Boolean>,
    reachable: Flow<Boolean?>,
    stale: Boolean = false,
    onBack: () -> Unit,
) {
    val latest by rememberUpdatedState(onBack)
    LaunchedEffect(online, reachable) {
        reachableAgain(online, reachable, stale).collect { latest() }
    }
}
