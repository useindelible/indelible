package app.indelible.profile.repository

import app.indelible.core.model.SaveItemRequest
import app.indelible.core.network.LibraryApiService

class ApiAddLibraryRepository(
    private val libraryApiService: LibraryApiService,
) : AddLibraryRepository {
    override suspend fun save(url: String): Result<Unit> =
        libraryApiService
            .saveItem(SaveItemRequest(url = url))
            .map {}
}
