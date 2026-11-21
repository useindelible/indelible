package app.indelible.core.offline

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.drop
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge

/** The OS reporting the network back, or any request getting through to the server; the first values are a baseline. */
fun reachableAgain(
    online: Flow<Boolean>,
    reachable: Flow<Boolean?>,
): Flow<Unit> = merge(online.risingEdges(), reachable.map { it == true }.risingEdges())

/** The phone has a network but the last request to the server got no answer. */
fun serverUnreachable(
    online: Flow<Boolean>,
    reachable: Flow<Boolean?>,
): Flow<Boolean> = combine(online, reachable) { isOnline, up -> isOnline && up == false }

private fun Flow<Boolean>.risingEdges(): Flow<Unit> =
    distinctUntilChanged()
        .drop(1)
        .filter { it }
        .map { }
