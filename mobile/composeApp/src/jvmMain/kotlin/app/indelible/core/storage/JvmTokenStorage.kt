package app.indelible.core.storage

import app.indelible.core.network.resolvedServerUrl
import app.indelible.share.repository.JvmPendingSaveRepository
import app.indelible.share.repository.PendingSaveQueueOwner
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import java.util.Base64

private val jwtPayloadJson = Json { ignoreUnknownKeys = true }

class JvmTokenStorage(
    private val delegate: InMemoryTokenStorage,
    private val pendingSaveRepository: JvmPendingSaveRepository,
) : TokenStorage by delegate {
    override suspend fun clearAll() {
        try {
            pendingSaveRepository.clearAll()
        } finally {
            delegate.clearAll()
        }
    }
}

suspend fun InMemoryTokenStorage.pendingQueueOwner(): PendingSaveQueueOwner? {
    val serverUrl = resolvedServerUrl()
    val token = getToken() ?: return null
    val payload = token.split('.').takeIf { it.size == 3 }?.get(1) ?: return null

    val userId =
        try {
            jwtPayloadJson.decodeFromString<JwtPayload>(String(Base64.getUrlDecoder().decode(payload)))
                .sub
                ?.takeIf { it.isNotBlank() }
        } catch (_: Exception) {
            null
        }
    return userId?.let { PendingSaveQueueOwner(serverUrl, it) }
}

@Serializable
private data class JwtPayload(
    val sub: String? = null,
)
