package app.indelible.share

import app.indelible.core.network.AuthenticatedApiTransport
import app.indelible.core.network.LibraryApiService
import app.indelible.core.storage.IosTokenStorage
import app.indelible.share.repository.IosPendingSaveRepository
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.IO
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlin.native.ObjCName

// @ObjCName exposes this class to Swift under the name ShareExtensionBridge without a package prefix.
// Callback-based API avoids Swift concurrency bridging complexity with KMP suspend functions.
@OptIn(kotlin.experimental.ExperimentalObjCName::class)
@ObjCName("ShareExtensionBridge")
class ShareExtensionBridge {
    private val scopeJob = SupervisorJob()
    private val scope = CoroutineScope(scopeJob + Dispatchers.IO)

    private val tokenStorage = IosTokenStorage()
    private val transport = AuthenticatedApiTransport(tokenStorage)
    private val libraryApiService = LibraryApiService(transport)
    private val pendingSaveRepository = IosPendingSaveRepository()
    private val saveUrlUseCase = SaveUrlUseCase(libraryApiService, tokenStorage, pendingSaveRepository)

    fun save(
        url: String,
        completion: (Boolean, String?) -> Unit,
    ) {
        scope.launch {
            val result = saveUrlUseCase.save(url)
            val (success, message) =
                when (result) {
                    is SaveResult.Success -> true to null
                    is SaveResult.AlreadySaved -> true to "already_saved"
                    is SaveResult.Queued -> true to "queued"
                    is SaveResult.AuthRequired -> false to "auth_required"
                    is SaveResult.InvalidUrl -> false to "invalid_url"
                    is SaveResult.Error -> false to result.message
                }
            // KMP-to-Swift interop requires callbacks to be delivered on the main thread
            withContext(Dispatchers.Main) {
                completion(success, message)
            }
        }
    }

    fun close() {
        scopeJob.cancel()
        transport.close()
    }
}
