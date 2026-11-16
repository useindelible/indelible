package app.indelible.core.offline

import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow

/** Tells open screens that a document's local annotations changed outside them, after the change committed. */
class LocalChangeEvents {
    private val _documentChanged =
        MutableSharedFlow<String>(extraBufferCapacity = EVENT_BUFFER, onBufferOverflow = BufferOverflow.DROP_OLDEST)
    val documentChanged: SharedFlow<String> = _documentChanged.asSharedFlow()

    fun emit(documentId: String) {
        _documentChanged.tryEmit(documentId)
    }

    private companion object {
        const val EVENT_BUFFER = 16
    }
}
