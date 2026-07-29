package app.indelible.sidebar.repository

import app.indelible.sidebar.model.Collection
import app.indelible.sidebar.model.SmartList

interface SidebarRepository {
    suspend fun listCollections(): Result<List<Collection>>

    suspend fun listSmartLists(): Result<List<SmartList>>
}
