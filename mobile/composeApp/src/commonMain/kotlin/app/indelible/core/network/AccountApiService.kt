package app.indelible.core.network

import app.indelible.api.generated.client.ApiV1MeClient
import app.indelible.api.generated.client.ApiV1MeEmailClient
import app.indelible.api.generated.client.ApiV1MePasswordClient
import app.indelible.api.generated.client.ApiV1TokensClient
import app.indelible.api.generated.models.ApiTokenResponse
import app.indelible.api.generated.models.ChangeEmailRequest
import app.indelible.api.generated.models.ChangePasswordRequest
import app.indelible.api.generated.models.CreateApiTokenRequest
import app.indelible.api.generated.models.CreateApiTokenResponse
import app.indelible.api.generated.models.DeleteAccountRequest
import app.indelible.api.generated.models.UpdateProfileRequest
import app.indelible.core.model.AuthUser
import app.indelible.core.model.toAuthUser
import io.ktor.client.request.get
import io.ktor.client.request.header
import io.ktor.client.statement.bodyAsBytes
import io.ktor.client.statement.bodyAsText
import io.ktor.http.isSuccess

class AccountApiService(
    private val transport: AuthenticatedApiTransport,
) {
    suspend fun getSession(): Result<AuthUser> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1MeClient(client).getProfile(configuration)
            }.map { it.toAuthUser() }

    suspend fun updateProfile(displayName: String): Result<AuthUser> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1MeClient(client).updateProfile(UpdateProfileRequest(displayName), configuration)
            }.map { it.toAuthUser() }

    suspend fun deleteAccount(confirmation: String): Result<Unit> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1MeClient(client).deleteAccount(DeleteAccountRequest(confirmation), configuration)
        }

    suspend fun changeEmail(
        newEmail: String,
        password: String,
    ): Result<Unit> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1MeEmailClient(client).changeEmail(ChangeEmailRequest(newEmail, password), configuration)
        }

    suspend fun changePassword(
        currentPassword: String,
        newPassword: String,
    ): Result<Unit> =
        transport.authenticatedRequest(retryOn401 = false) { client, configuration ->
            ApiV1MePasswordClient(client).changePassword(
                ChangePasswordRequest(currentPassword, newPassword),
                configuration,
            )
        }

    suspend fun fetchAvatarBytes(avatarUrl: String): Result<ByteArray> =
        transport.directAuthenticatedRequest { client, _, token ->
            val response =
                client.get(transport.rewriteBackendOrigin(avatarUrl)) {
                    header("Authorization", "Bearer $token")
                }
            if (!response.status.isSuccess()) {
                throw ApiException(response.status.value, response.bodyAsText())
            }
            response.bodyAsBytes()
        }

    suspend fun listApiTokens(): Result<List<ApiTokenResponse>> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1TokensClient(client).listTokens(configuration)
            }.map { it.data }

    suspend fun createApiToken(body: CreateApiTokenRequest): Result<CreateApiTokenResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1TokensClient(client).createToken(body, configuration)
        }

    suspend fun deleteApiToken(tokenId: String): Result<Unit> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1TokensClient(client).revokeToken(tokenId, configuration)
        }
}
