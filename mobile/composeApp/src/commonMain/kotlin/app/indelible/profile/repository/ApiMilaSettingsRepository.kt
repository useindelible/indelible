package app.indelible.profile.repository

import app.indelible.api.generated.models.CreateMilaPromptPresetBody
import app.indelible.api.generated.models.MilaConfigResponse
import app.indelible.api.generated.models.MilaPromptPresetResponse
import app.indelible.api.generated.models.MilaPromptPresetsResponse
import app.indelible.api.generated.models.TestMilaConfigBody
import app.indelible.api.generated.models.TestMilaConfigResponse
import app.indelible.api.generated.models.UpdateMilaPromptPresetBody
import app.indelible.api.generated.models.UpsertMilaConfigBody
import app.indelible.core.network.MilaApiService

class ApiMilaSettingsRepository(
    private val milaApiService: MilaApiService,
) : MilaSettingsRepository {
    override suspend fun getConfig(): Result<MilaConfigResponse> = milaApiService.getConfig()

    override suspend fun upsertConfig(body: UpsertMilaConfigBody): Result<MilaConfigResponse> = milaApiService.upsertConfig(body)

    override suspend fun reindexConfig(body: UpsertMilaConfigBody): Result<MilaConfigResponse> = milaApiService.reindexConfig(body)

    override suspend fun testConfig(body: TestMilaConfigBody): Result<TestMilaConfigResponse> = milaApiService.testConfig(body)

    override suspend fun getPromptPresets(): Result<MilaPromptPresetsResponse> = milaApiService.getPromptPresets()

    override suspend fun createPromptPreset(body: CreateMilaPromptPresetBody): Result<MilaPromptPresetResponse> =
        milaApiService.createPromptPreset(body)

    override suspend fun updatePromptPreset(
        presetId: String,
        body: UpdateMilaPromptPresetBody,
    ): Result<MilaPromptPresetResponse> = milaApiService.updatePromptPreset(presetId, body)

    override suspend fun deletePromptPreset(presetId: String): Result<Unit> = milaApiService.deletePromptPreset(presetId)
}
