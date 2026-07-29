package app.indelible.core.network

import app.indelible.api.generated.client.ApiV1ImportsClient
import app.indelible.api.generated.client.ApiV1ImportsRollbackClient
import app.indelible.api.generated.models.ImportJobStatusResponse
import app.indelible.api.generated.models.ImportUploadResponse
import io.ktor.client.call.body
import io.ktor.client.request.forms.MultiPartFormDataContent
import io.ktor.client.request.forms.formData
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsText
import io.ktor.http.Headers
import io.ktor.http.HttpHeaders
import io.ktor.http.isSuccess

class ImportApiService(
    private val transport: AuthenticatedApiTransport,
) {
    suspend fun uploadImport(
        sourceSlug: String,
        fileBytes: ByteArray,
        fileName: String,
        contentType: String,
    ): Result<ImportUploadResponse> =
        transport.directAuthenticatedRequest { client, baseUrl, token ->
            val response =
                client.post("$baseUrl/api/v1/imports/$sourceSlug") {
                    header("Authorization", "Bearer $token")
                    setBody(
                        MultiPartFormDataContent(
                            formData {
                                append(
                                    "file",
                                    fileBytes,
                                    Headers.build {
                                        append(HttpHeaders.ContentDisposition, "filename=\"$fileName\"")
                                        append(HttpHeaders.ContentType, contentType)
                                    },
                                )
                            },
                        ),
                    )
                }
            if (!response.status.isSuccess()) {
                throw ApiException(response.status.value, response.bodyAsText())
            }
            response.body<ImportUploadResponse>()
        }

    suspend fun getImport(importJobId: String): Result<ImportJobStatusResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1ImportsClient(client).getImport(importJobId, configuration)
        }

    suspend fun rollbackImport(importJobId: String): Result<Unit> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1ImportsRollbackClient(client).rollbackImport(importJobId, configuration)
        }
}
