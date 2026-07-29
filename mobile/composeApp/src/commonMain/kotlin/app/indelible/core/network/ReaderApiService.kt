package app.indelible.core.network

import app.indelible.api.generated.client.ApiV1DocumentsAssetsClient
import app.indelible.api.generated.client.ApiV1DocumentsClient
import app.indelible.api.generated.client.ApiV1DocumentsEntitiesClient
import app.indelible.api.generated.client.ApiV1DocumentsHighlightsClient
import app.indelible.api.generated.client.ApiV1DocumentsNoteClient
import app.indelible.api.generated.client.ApiV1DocumentsProgressClient
import app.indelible.api.generated.client.ApiV1DocumentsReprocessClient
import app.indelible.api.generated.client.ApiV1DocumentsTocClient
import app.indelible.api.generated.client.ApiV1HighlightsClient
import app.indelible.api.generated.client.ApiV1HighlightsNoteClient
import app.indelible.api.generated.client.ApiV1HighlightsTagsClient
import app.indelible.api.generated.client.ApiV1LibraryTagsClient
import app.indelible.api.generated.client.ApiV1TagsClient
import app.indelible.api.generated.models.CreateHighlightBody
import app.indelible.api.generated.models.DocumentNoteResponse
import app.indelible.api.generated.models.DocumentReaderResponse
import app.indelible.api.generated.models.ArticleTocResponse
import app.indelible.api.generated.models.DocumentReprocessResponse
import app.indelible.api.generated.models.DocumentUpsertNoteBody
import app.indelible.api.generated.models.EntitySummaryResponse
import app.indelible.api.generated.models.HighlightListResponse
import app.indelible.api.generated.models.HighlightNoteResponse
import app.indelible.api.generated.models.HighlightResponse
import app.indelible.api.generated.models.HighlightTagsBody
import app.indelible.api.generated.models.HighlightWithNoteResponse
import app.indelible.api.generated.models.LibraryEntryTagsBody
import app.indelible.api.generated.models.PatchHighlightBody
import app.indelible.api.generated.models.TagResponse
import app.indelible.api.generated.models.UpdateDocumentProgressBody
import app.indelible.api.generated.models.UpsertNoteBody
import app.indelible.reader.model.AssetWithUrlResponse
import app.indelible.reader.model.CreateHighlightRequest
import app.indelible.reader.model.toAssetWithUrlResponse
import app.indelible.reader.model.toLocatorSchemaFlat
import io.ktor.client.request.get
import io.ktor.client.request.header
import io.ktor.client.statement.bodyAsText
import io.ktor.http.isSuccess

class ReaderApiService(
    private val transport: AuthenticatedApiTransport,
) {
    suspend fun getDocumentReader(documentId: String): Result<DocumentReaderResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1DocumentsClient(client).getDocumentReader(documentId, configuration)
        }

    suspend fun listDocumentEntities(documentId: String): Result<List<EntitySummaryResponse>> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1DocumentsEntitiesClient(client).listDocumentEntities(documentId, configuration)
        }

    suspend fun reprocessDocument(documentId: String): Result<DocumentReprocessResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1DocumentsReprocessClient(client).reprocessDocument(documentId, configuration)
        }

    suspend fun getArticleToc(documentId: String): Result<ArticleTocResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1DocumentsTocClient(client).getArticleToc(documentId, configuration)
        }

    suspend fun getAssetWithUrl(
        itemId: String,
        assetKind: String,
    ): Result<AssetWithUrlResponse> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1DocumentsAssetsClient(client).getDocumentAsset(
                    documentId = itemId,
                    assetKind = assetKind,
                    apiConfiguration = configuration,
                )
            }.map { it.toAssetWithUrlResponse() }

    suspend fun streamAsset(
        itemId: String,
        assetKind: String,
    ): Result<String> =
        transport.directAuthenticatedRequest { client, baseUrl, token ->
            val response =
                client.get("$baseUrl/api/v1/assets/documents/$itemId/$assetKind") {
                    header("Authorization", "Bearer $token")
                }
            if (!response.status.isSuccess()) {
                throw ApiException(response.status.value, "Failed to stream asset")
            }
            response.bodyAsText()
        }

    suspend fun updateProgress(
        itemId: String,
        progressPercent: Float,
    ): Result<Unit> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1DocumentsProgressClient(client).updateDocumentProgress(
                updateDocumentProgressBody = UpdateDocumentProgressBody(progressPercent = progressPercent),
                documentId = itemId,
                apiConfiguration = configuration,
            )
        }

    suspend fun listHighlights(itemId: String): Result<HighlightListResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1DocumentsHighlightsClient(client).listDocumentHighlights(itemId, configuration)
        }

    suspend fun createHighlight(
        itemId: String,
        request: CreateHighlightRequest,
    ): Result<HighlightWithNoteResponse> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1DocumentsHighlightsClient(client).createDocumentHighlight(
                    createHighlightBody =
                        CreateHighlightBody(
                            color = request.color,
                            locator = request.locator.toLocatorSchemaFlat(),
                            textContent = request.textContent,
                        ),
                    documentId = itemId,
                    apiConfiguration = configuration,
                )
            }.map { it.toHighlightWithNote() }

    suspend fun deleteHighlight(highlightId: String): Result<Unit> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1HighlightsClient(client).deleteHighlight(highlightId, configuration)
        }

    suspend fun patchHighlight(
        highlightId: String,
        color: String,
    ): Result<HighlightWithNoteResponse> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1HighlightsClient(client).patchHighlight(
                    patchHighlightBody = PatchHighlightBody(color = color),
                    highlightId = highlightId,
                    apiConfiguration = configuration,
                )
            }.map { it.toHighlightWithNote() }

    suspend fun upsertHighlightNote(
        highlightId: String,
        body: String,
    ): Result<HighlightNoteResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1HighlightsNoteClient(client).upsertNote(
                upsertNoteBody = UpsertNoteBody(body = body),
                highlightId = highlightId,
                apiConfiguration = configuration,
            )
        }

    suspend fun deleteHighlightNote(highlightId: String): Result<Unit> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1HighlightsNoteClient(client).deleteNote(highlightId, configuration)
        }

    suspend fun setHighlightTags(
        highlightId: String,
        tags: List<String>,
    ): Result<List<String>> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1HighlightsTagsClient(client).setHighlightTags(
                    highlightTagsBody = HighlightTagsBody(tags = tags),
                    highlightId = highlightId,
                    apiConfiguration = configuration,
                )
            }.map { it.tags }

    suspend fun listTags(
        scope: String? = null,
        limit: Int = 100,
    ): Result<List<TagResponse>> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1TagsClient(client).listTags(
                    limit = limit,
                    scope = scope,
                    apiConfiguration = configuration,
                )
            }.map { it.data }

    suspend fun getItemNote(itemId: String): Result<DocumentNoteResponse?> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1DocumentsNoteClient(client).getDocumentNote(itemId, configuration)
            }.fold(
                onSuccess = { Result.success(it) },
                onFailure = { error ->
                    if (error is ApiException && error.statusCode == NOT_FOUND_STATUS) {
                        Result.success(null)
                    } else {
                        Result.failure(error)
                    }
                },
            )

    suspend fun upsertItemNote(
        itemId: String,
        body: String,
    ): Result<DocumentNoteResponse> =
        transport.authenticatedRequest { client, configuration ->
            ApiV1DocumentsNoteClient(client).upsertDocumentNote(
                documentUpsertNoteBody = DocumentUpsertNoteBody(body = body),
                documentId = itemId,
                apiConfiguration = configuration,
            )
        }

    suspend fun getItemTags(itemId: String): Result<List<String>> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1LibraryTagsClient(client).getEntryTags(itemId, configuration)
            }.map { it.tags }

    suspend fun setItemTags(
        itemId: String,
        tags: List<String>,
    ): Result<List<String>> =
        transport
            .authenticatedRequest { client, configuration ->
                ApiV1LibraryTagsClient(client).setEntryTags(
                    libraryEntryTagsBody = LibraryEntryTagsBody(tags = tags),
                    libraryEntryId = itemId,
                    apiConfiguration = configuration,
                )
            }.map { it.tags }

    private fun HighlightResponse.toHighlightWithNote(): HighlightWithNoteResponse =
        HighlightWithNoteResponse(
            color = color,
            createdAt = createdAt,
            id = id,
            documentId = documentId,
            itemTitle = null,
            locator = locator,
            note = null,
            sourceLocator = sourceLocator,
            tags = emptyList(),
            textContent = textContent,
            updatedAt = updatedAt,
        )

    private companion object {
        const val NOT_FOUND_STATUS = 404
    }
}
