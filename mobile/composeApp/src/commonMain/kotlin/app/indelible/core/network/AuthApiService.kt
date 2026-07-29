package app.indelible.core.network

import app.indelible.api.generated.client.ApiV1AuthEmailResendClient
import app.indelible.api.generated.client.ApiV1AuthLoginClient
import app.indelible.api.generated.client.ApiV1AuthLogoutClient
import app.indelible.api.generated.client.ApiV1AuthPasswordForgotClient
import app.indelible.api.generated.client.ApiV1AuthPasswordResetClient
import app.indelible.api.generated.client.ApiV1AuthProvidersClient
import app.indelible.api.generated.client.ApiV1AuthRegisterClient
import app.indelible.api.generated.models.ForgotPasswordRequest
import app.indelible.api.generated.models.LoginRequest
import app.indelible.api.generated.models.OAuthProvidersResponse
import app.indelible.api.generated.models.RefreshTokenRequest
import app.indelible.api.generated.models.RegisterRequest
import app.indelible.api.generated.models.ResetPasswordRequest
import app.indelible.auth.oauth.NativeOAuthRedirectUri
import app.indelible.core.model.AuthResponse
import app.indelible.core.platform.platformClientType
import io.ktor.client.call.body
import io.ktor.client.request.forms.FormDataContent
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsText
import io.ktor.http.Parameters
import io.ktor.http.isSuccess

class AuthApiService(
    private val transport: AuthenticatedApiTransport,
) {
    suspend fun login(
        email: String,
        password: String,
    ): Result<AuthResponse> =
        transport.publicRequest { client, configuration ->
            ApiV1AuthLoginClient(client).login(LoginRequest(email, password), configuration)
        }

    suspend fun register(
        name: String,
        email: String,
        password: String,
    ): Result<AuthResponse> =
        transport.publicRequest { client, configuration ->
            ApiV1AuthRegisterClient(client).register(
                RegisterRequest(email = email, password = password, displayName = name),
                configuration,
            )
        }

    suspend fun forgotPassword(email: String): Result<Unit> =
        transport
            .publicRequest { client, configuration ->
                ApiV1AuthPasswordForgotClient(client).forgotPassword(ForgotPasswordRequest(email), configuration)
            }.map { Unit }

    suspend fun resetPassword(
        token: String,
        newPassword: String,
    ): Result<Unit> =
        transport
            .publicRequest { client, configuration ->
                ApiV1AuthPasswordResetClient(client).resetPassword(
                    ResetPasswordRequest(token = token, newPassword = newPassword),
                    configuration,
                )
            }.map { Unit }

    suspend fun logout(): Result<Unit> {
        val refreshToken = transport.refreshToken() ?: return Result.success(Unit)
        return transport.publicRequest { client, configuration ->
            ApiV1AuthLogoutClient(client).logout(RefreshTokenRequest(refreshToken), configuration)
        }
    }

    suspend fun resendVerification(): Result<Unit> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1AuthEmailResendClient(client).resendVerification(configuration)
            }.map { Unit }

    suspend fun getOAuthProviders(): Result<OAuthProvidersResponse> =
        transport.publicRequest { client, configuration ->
            ApiV1AuthProvidersClient(client).listProviders(configuration)
        }

    suspend fun nativeOAuthStartUrl(
        providerId: String,
        codeChallenge: String,
        appState: String,
    ): String =
        "${transport.baseUrl()}/api/v1/auth/oauth/${encodeURIComponent(providerId)}/native/start" +
            "?platform=${platformClientType()}" +
            "&code_challenge=${encodeURIComponent(codeChallenge)}" +
            "&code_challenge_method=S256" +
            "&app_state=${encodeURIComponent(appState)}"

    suspend fun exchangeNativeOAuthCode(
        code: String,
        codeVerifier: String,
    ): Result<NativeOAuthTokenResponse> =
        runCatching {
            val response =
                transport.httpClient.post("${transport.baseUrl()}/api/v1/auth/oauth/native/token") {
                    header("X-Client-Type", platformClientType())
                    setBody(
                        FormDataContent(
                            Parameters.build {
                                append("grant_type", "authorization_code")
                                append("code", code)
                                append("code_verifier", codeVerifier)
                                append("redirect_uri", NativeOAuthRedirectUri)
                            },
                        ),
                    )
                }
            if (!response.status.isSuccess()) {
                throw ApiException(response.status.value, response.bodyAsText())
            }
            response.body<NativeOAuthTokenResponse>()
        }

    private fun encodeURIComponent(value: String): String =
        value.encodeToByteArray().joinToString("") { byte ->
            val intValue = byte.toInt() and BYTE_MASK
            val char = intValue.toChar()
            if (char.isLetterOrDigit() || char == '-' || char == '_' || char == '.' || char == '~') {
                char.toString()
            } else {
                "%${intValue.toString(HEX_RADIX).uppercase().padStart(2, '0')}"
            }
        }

    private companion object {
        const val BYTE_MASK = 0xff
        const val HEX_RADIX = 16
    }
}
