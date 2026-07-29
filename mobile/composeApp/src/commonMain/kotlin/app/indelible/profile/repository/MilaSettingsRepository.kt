package app.indelible.profile.repository

import app.indelible.api.generated.models.CreateMilaPromptPresetBody
import app.indelible.api.generated.models.MilaConfigResponse
import app.indelible.api.generated.models.MilaPromptPresetResponse
import app.indelible.api.generated.models.MilaPromptPresetsResponse
import app.indelible.api.generated.models.TestMilaConfigBody
import app.indelible.api.generated.models.TestMilaConfigResponse
import app.indelible.api.generated.models.UpdateMilaPromptPresetBody
import app.indelible.api.generated.models.UpsertMilaConfigBody

interface MilaSettingsRepository {
    suspend fun getConfig(): Result<MilaConfigResponse>

    suspend fun upsertConfig(body: UpsertMilaConfigBody): Result<MilaConfigResponse>

    suspend fun reindexConfig(body: UpsertMilaConfigBody): Result<MilaConfigResponse>

    suspend fun testConfig(body: TestMilaConfigBody): Result<TestMilaConfigResponse>

    suspend fun getPromptPresets(): Result<MilaPromptPresetsResponse>

    suspend fun createPromptPreset(body: CreateMilaPromptPresetBody): Result<MilaPromptPresetResponse>

    suspend fun updatePromptPreset(
        presetId: String,
        body: UpdateMilaPromptPresetBody,
    ): Result<MilaPromptPresetResponse>

    suspend fun deletePromptPreset(presetId: String): Result<Unit>
}
