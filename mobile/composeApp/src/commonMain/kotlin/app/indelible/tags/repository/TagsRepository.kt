package app.indelible.tags.repository

import app.indelible.core.model.PaginatedItems
import app.indelible.reader.model.TagData

interface TagsRepository {
    suspend fun listTags(scope: String? = null): Result<List<TagData>>

    suspend fun getTag(id: String): Result<TagData>

    suspend fun listTagItems(
        tagId: String,
        cursor: String? = null,
    ): Result<PaginatedItems>
}
