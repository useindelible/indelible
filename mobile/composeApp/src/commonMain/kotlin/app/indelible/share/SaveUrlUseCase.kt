package app.indelible.share

import app.indelible.core.model.SaveItemRequest
import app.indelible.core.model.SaveLibraryEntryResponse
import app.indelible.core.network.ApiException
import app.indelible.core.network.LibraryApiService
import app.indelible.core.storage.TokenStorage
import app.indelible.share.model.PendingItem
import app.indelible.share.repository.PendingSaveRepository

sealed class SaveResult {
    data class Success(
        val response: SaveLibraryEntryResponse,
    ) : SaveResult()

    data object AlreadySaved : SaveResult()

    data object Queued : SaveResult()

    data object AuthRequired : SaveResult()

    data object InvalidUrl : SaveResult()

    data class Error(
        val message: String,
    ) : SaveResult()
}

class SaveUrlUseCase(
    private val libraryApiService: LibraryApiService,
    private val tokenStorage: TokenStorage,
    private val pendingSaveRepository: PendingSaveRepository,
    private val networkExceptionDetector: (Throwable) -> Boolean = ::isNetworkException,
) {
    suspend fun save(url: String): SaveResult {
        if (!isValidUrl(url)) {
            return SaveResult.InvalidUrl
        }

        val token = tokenStorage.getToken()
        val refreshToken = tokenStorage.getRefreshToken()
        if (token == null && refreshToken == null) {
            return SaveResult.AuthRequired
        }

        drainQueue()

        return try {
            val result = libraryApiService.saveItem(SaveItemRequest(url = url))
            result.fold(
                onSuccess = { response -> SaveResult.Success(response) },
                onFailure = { throwable ->
                    when {
                        throwable is ApiException && throwable.statusCode == STATUS_UNAUTHORIZED ->
                            SaveResult.AuthRequired
                        throwable is ApiException && throwable.statusCode == STATUS_CONFLICT ->
                            SaveResult.AlreadySaved
                        networkExceptionDetector(throwable) -> {
                            enqueueForLater(url)
                            SaveResult.Queued
                        }
                        else -> SaveResult.Error(throwable.message ?: "Unknown error")
                    }
                },
            )
        } catch (e: Exception) {
            if (networkExceptionDetector(e)) {
                enqueueForLater(url)
                SaveResult.Queued
            } else {
                SaveResult.Error(e.message ?: "Unknown error")
            }
        }
    }

    private suspend fun drainQueue() {
        val pending = pendingSaveRepository.drainAll()
        for ((index, item) in pending.withIndex()) {
            try {
                val result = libraryApiService.saveItem(SaveItemRequest(url = item.url))
                result.onFailure { throwable ->
                    if (networkExceptionDetector(throwable)) {
                        pending.drop(index).forEach { pendingSaveRepository.enqueue(it) }
                        return
                    }
                }
            } catch (e: Exception) {
                if (networkExceptionDetector(e)) {
                    pending.drop(index).forEach { pendingSaveRepository.enqueue(it) }
                    return
                }
            }
        }
    }

    private suspend fun enqueueForLater(url: String) {
        val now = currentEpochMillis()
        val item =
            PendingItem(
                id = "$now-${(0..999999).random()}",
                url = url,
                enqueuedAt = now,
            )
        pendingSaveRepository.enqueue(item)
    }

    companion object {
        private const val STATUS_UNAUTHORIZED = 401
        private const val STATUS_CONFLICT = 409
    }
}

internal expect fun isNetworkException(throwable: Throwable): Boolean

internal expect fun currentEpochMillis(): Long
