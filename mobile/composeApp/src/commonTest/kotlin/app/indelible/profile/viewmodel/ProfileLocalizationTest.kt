package app.indelible.profile.viewmodel

import app.indelible.api.generated.models.CreateMilaPromptPresetBody
import app.indelible.api.generated.models.MilaConfigResponse
import app.indelible.api.generated.models.MilaPromptPresetResponse
import app.indelible.api.generated.models.MilaPromptPresetsResponse
import app.indelible.api.generated.models.TestMilaConfigBody
import app.indelible.api.generated.models.TestMilaConfigResponse
import app.indelible.api.generated.models.UpdateMilaPromptPresetBody
import app.indelible.api.generated.models.UpsertMilaConfigBody
import app.indelible.core.i18n.UiMessage
import app.indelible.core.preferences.DefaultViewPreference
import app.indelible.core.preferences.ReaderFontFamilyPreference
import app.indelible.core.preferences.ReaderFontSizePreference
import app.indelible.core.preferences.ReaderLineHeightPreference
import app.indelible.core.preferences.ThemePreference
import app.indelible.core.preferences.TriageModePreference
import app.indelible.profile.repository.MilaSettingsRepository
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.mila_name_required
import indelible.composeapp.generated.resources.mila_preset_save_failed
import indelible.composeapp.generated.resources.mila_required_settings
import indelible.composeapp.generated.resources.mila_save_failed
import indelible.composeapp.generated.resources.mila_system_prompt_required
import indelible.composeapp.generated.resources.prefs_default_view_library
import indelible.composeapp.generated.resources.prefs_font_lora
import indelible.composeapp.generated.resources.prefs_font_lora_description
import indelible.composeapp.generated.resources.prefs_font_size_medium
import indelible.composeapp.generated.resources.prefs_line_height_relaxed
import indelible.composeapp.generated.resources.prefs_theme_auto
import indelible.composeapp.generated.resources.prefs_triage_mode_manual
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalCoroutinesApi::class)
class ProfileLocalizationTest {
    private val testDispatcher = UnconfinedTestDispatcher()

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun preference_options_expose_resource_metadata() {
        assertEquals(Res.string.prefs_theme_auto, ThemePreference.AUTO.labelRes)
        assertEquals(Res.string.prefs_default_view_library, DefaultViewPreference.LIBRARY.labelRes)
        assertEquals(Res.string.prefs_triage_mode_manual, TriageModePreference.MANUAL.labelRes)
        assertEquals(Res.string.prefs_font_lora, ReaderFontFamilyPreference.SERIF.labelRes)
        assertEquals(Res.string.prefs_font_lora_description, ReaderFontFamilyPreference.SERIF.descriptionRes)
        assertEquals(Res.string.prefs_font_size_medium, ReaderFontSizePreference.MEDIUM.labelRes)
        assertEquals(Res.string.prefs_line_height_relaxed, ReaderLineHeightPreference.RELAXED.labelRes)
    }

    @Test
    fun ai_settings_select_semantic_validation_and_repository_errors() =
        runTest(testDispatcher) {
            val viewModel = AiSettingsViewModel(FailingMilaSettingsRepository())

            viewModel.save()
            assertEquals(UiMessage(Res.string.mila_required_settings), viewModel.uiState.value.saveError)

            viewModel.updateApiBase("https://example.com")
            viewModel.updateChatModel("model-id")
            viewModel.save()
            advanceUntilIdle()

            assertEquals(UiMessage(Res.string.mila_save_failed), viewModel.uiState.value.saveError)
        }

    @Test
    fun preset_editor_selects_specific_validation_and_repository_errors() =
        runTest(testDispatcher) {
            val viewModel = PromptPresetEditViewModel(FailingMilaSettingsRepository(), existingPreset = null)

            viewModel.save()
            assertEquals(UiMessage(Res.string.mila_name_required), viewModel.uiState.value.saveError)

            viewModel.updateName("My preset")
            viewModel.save()
            assertEquals(UiMessage(Res.string.mila_system_prompt_required), viewModel.uiState.value.saveError)

            viewModel.updateSystemPrompt("Summarize this")
            viewModel.save()
            advanceUntilIdle()

            assertEquals(UiMessage(Res.string.mila_preset_save_failed), viewModel.uiState.value.saveError)
        }

    private class FailingMilaSettingsRepository : MilaSettingsRepository {
        override suspend fun getConfig(): Result<MilaConfigResponse> = failure()

        override suspend fun upsertConfig(body: UpsertMilaConfigBody): Result<MilaConfigResponse> = failure()

        override suspend fun reindexConfig(body: UpsertMilaConfigBody): Result<MilaConfigResponse> = failure()

        override suspend fun testConfig(body: TestMilaConfigBody): Result<TestMilaConfigResponse> = failure()

        override suspend fun getPromptPresets(): Result<MilaPromptPresetsResponse> = failure()

        override suspend fun createPromptPreset(body: CreateMilaPromptPresetBody): Result<MilaPromptPresetResponse> = failure()

        override suspend fun updatePromptPreset(
            presetId: String,
            body: UpdateMilaPromptPresetBody,
        ): Result<MilaPromptPresetResponse> = failure()

        override suspend fun deletePromptPreset(presetId: String): Result<Unit> = failure()
    }
}

private fun <T> failure(): Result<T> = Result.failure(IllegalStateException("sensitive provider detail"))
