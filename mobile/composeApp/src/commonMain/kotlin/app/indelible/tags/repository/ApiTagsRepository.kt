package app.indelible.tags.repository

import app.indelible.core.model.PaginatedItems
import app.indelible.core.model.toPaginatedItems
import app.indelible.core.network.TagsApiService
import app.indelible.reader.model.TagData
import app.indelible.reader.model.toTagData

class ApiTagsRepository(
    private val tagsApiService: TagsApiService,
) : TagsRepository {
    override suspend fun listTags(scope: String?): Result<List<TagData>> =
        tagsApiService.listTags(scope = scope).map { tags -> tags.map { it.toTagData() } }

    override suspend fun getTag(id: String): Result<TagData> = tagsApiService.getTag(id).map { it.toTagData() }

    override suspend fun listTagItems(
        tagId: String,
        cursor: String?,
    ): Result<PaginatedItems> = tagsApiService.listTagItems(tagId, cursor).map { it.toPaginatedItems() }
}
