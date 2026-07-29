package app.indelible.sidebar.repository

import app.indelible.core.network.CollectionsApiService
import app.indelible.sidebar.model.Collection
import app.indelible.sidebar.model.SmartList

class ApiSidebarRepository(
    private val collectionsApiService: CollectionsApiService,
) : SidebarRepository {
    override suspend fun listCollections(): Result<List<Collection>> = collectionsApiService.listCollections().map { it.data }

    override suspend fun listSmartLists(): Result<List<SmartList>> = collectionsApiService.listSmartLists().map { it.data }
}
