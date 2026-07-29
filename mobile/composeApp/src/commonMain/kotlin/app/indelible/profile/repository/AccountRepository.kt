package app.indelible.profile.repository

interface AccountRepository {
    suspend fun deleteAccount(confirmation: String): Result<Unit>

    suspend fun changePassword(
        currentPassword: String,
        newPassword: String,
    ): Result<Unit>
}
