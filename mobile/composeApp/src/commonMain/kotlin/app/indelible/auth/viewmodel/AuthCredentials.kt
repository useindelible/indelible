package app.indelible.auth.viewmodel

import app.indelible.api.generated.models.AuthResponse
import app.indelible.core.storage.TokenStorage

internal suspend fun TokenStorage.saveCredentials(response: AuthResponse) {
    response.accessToken?.let { saveToken(it) }
    response.refreshToken?.let { saveRefreshToken(it) }
    response.expiresAt?.let { saveExpiresAt(it) }
}
