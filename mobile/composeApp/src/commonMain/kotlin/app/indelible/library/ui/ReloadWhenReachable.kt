package app.indelible.library.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberUpdatedState
import app.indelible.core.offline.reachableAgain
import kotlinx.coroutines.flow.Flow

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
