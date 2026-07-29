package app.indelible.auth.oauth

import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow

object OAuthCallbackBus {
    private val _callbacks = MutableSharedFlow<String>(extraBufferCapacity = 1)
    val callbacks: SharedFlow<String> = _callbacks

    fun emit(url: String) {
        _callbacks.tryEmit(url)
    }
}
