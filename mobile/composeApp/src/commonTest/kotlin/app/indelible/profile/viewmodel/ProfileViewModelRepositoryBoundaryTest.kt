package app.indelible.profile.viewmodel

import app.indelible.api.generated.models.CreateMilaPromptPresetBody
import app.indelible.api.generated.models.MilaConfigResponse
import app.indelible.api.generated.models.MilaPromptPresetResponse
import app.indelible.api.generated.models.MilaPromptPresetsResponse
import app.indelible.api.generated.models.PreferencesSettingsResponse
import app.indelible.api.generated.models.TestMilaConfigBody
import app.indelible.api.generated.models.TestMilaConfigResponse
import app.indelible.api.generated.models.UpdateMilaPromptPresetBody
import app.indelible.api.generated.models.UpsertMilaConfigBody
import app.indelible.core.preferences.DefaultViewPreference
import app.indelible.core.preferences.ThemePreference
import app.indelible.core.storage.UserPreferencesStorage
import app.indelible.profile.repository.AccountRepository
import app.indelible.profile.repository.AddLibraryRepository
import app.indelible.profile.repository.MilaSettingsRepository
import app.indelible.profile.repository.PreferencesRepository
import kotlin.test.Test
import kotlin.test.assertNotNull

class ProfileViewModelRepositoryBoundaryTest {
    @Test
    fun profileViewModelsAreConstructedFromRepositories() {
        val addLibraryViewModel = AddLibraryViewModel(FakeAddLibraryRepository())
        val accountViewModel = AccountViewModel(FakeAccountRepository())
        val changePasswordViewModel = ChangePasswordViewModel(FakeAccountRepository())
        val aiSettingsViewModel = AiSettingsViewModel(FakeMilaSettingsRepository())
        val promptPresetEditViewModel =
            PromptPresetEditViewModel(
                repository = FakeMilaSettingsRepository(),
                existingPreset = null,
            )
        val userPreferencesViewModel =
            UserPreferencesViewModel(
                storage = FakeUserPreferencesStorage(),
                repository = FakePreferencesRepository(),
            )

        assertNotNull(addLibraryViewModel)
        assertNotNull(accountViewModel)
        assertNotNull(changePasswordViewModel)
        assertNotNull(aiSettingsViewModel)
        assertNotNull(promptPresetEditViewModel)
        assertNotNull(userPreferencesViewModel)
    }
}

private class FakeAddLibraryRepository : AddLibraryRepository {
    override suspend fun save(url: String): Result<Unit> = Result.success(Unit)
}

private class FakeAccountRepository : AccountRepository {
    override suspend fun deleteAccount(confirmation: String): Result<Unit> = Result.success(Unit)

    override suspend fun changePassword(
        currentPassword: String,
        newPassword: String,
    ): Result<Unit> = Result.success(Unit)
}

private class FakeMilaSettingsRepository : MilaSettingsRepository {
    override suspend fun getConfig(): Result<MilaConfigResponse> = unused()

    override suspend fun upsertConfig(body: UpsertMilaConfigBody): Result<MilaConfigResponse> = unused()

    override suspend fun reindexConfig(body: UpsertMilaConfigBody): Result<MilaConfigResponse> = unused()

    override suspend fun testConfig(body: TestMilaConfigBody): Result<TestMilaConfigResponse> = unused()

    override suspend fun getPromptPresets(): Result<MilaPromptPresetsResponse> = unused()

    override suspend fun createPromptPreset(body: CreateMilaPromptPresetBody): Result<MilaPromptPresetResponse> =
        unused()

    override suspend fun updatePromptPreset(
        presetId: String,
        body: UpdateMilaPromptPresetBody,
    ): Result<MilaPromptPresetResponse> = unused()

    override suspend fun deletePromptPreset(presetId: String): Result<Unit> = unused()
}

private class FakePreferencesRepository : PreferencesRepository {
    override suspend fun getPreferences(): Result<PreferencesSettingsResponse> =
        Result.failure(UnsupportedOperationException("not used"))

    override suspend fun updatePreferences(body: PreferencesSettingsResponse): Result<PreferencesSettingsResponse> =
        Result.failure(UnsupportedOperationException("not used"))
}

private class FakeUserPreferencesStorage : UserPreferencesStorage {
    override suspend fun saveTheme(theme: ThemePreference) = Unit

    override suspend fun getTheme(): ThemePreference = ThemePreference.AUTO

    override suspend fun saveDefaultView(view: DefaultViewPreference) = Unit

    override suspend fun getDefaultView(): DefaultViewPreference = DefaultViewPreference.LIBRARY
}

private fun <T> unused(): Result<T> = Result.failure(UnsupportedOperationException("not used"))
