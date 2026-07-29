package app.indelible.profile.repository

import app.indelible.core.network.AccountApiService

class ApiAccountRepository(
    private val accountApiService: AccountApiService,
) : AccountRepository {
    override suspend fun deleteAccount(confirmation: String): Result<Unit> = accountApiService.deleteAccount(confirmation)

    override suspend fun changePassword(
        currentPassword: String,
        newPassword: String,
    ): Result<Unit> = accountApiService.changePassword(currentPassword, newPassword)
}
