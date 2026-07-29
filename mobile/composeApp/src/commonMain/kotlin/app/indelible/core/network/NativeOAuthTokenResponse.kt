package app.indelible.core.network

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class NativeOAuthTokenResponse(
    @SerialName("access_token")
    val accessToken: String,
    @SerialName("refresh_token")
    val refreshToken: String,
    @SerialName("token_type")
    val tokenType: String,
    @SerialName("expires_at")
    val expiresAt: Long,
    @SerialName("refresh_token_expires_at")
    val refreshTokenExpiresAt: Long,
)
