package app.indelible.profile.repository

interface AddLibraryRepository {
    suspend fun save(url: String): Result<Unit>
}
