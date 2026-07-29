package app.indelible.profile.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.api.generated.models.DefaultViewDto
import app.indelible.api.generated.models.PreferencesSettingsResponse
import app.indelible.api.generated.models.ReaderFontFamilyDto
import app.indelible.api.generated.models.ReaderFontSizeDto
import app.indelible.api.generated.models.ReaderLineHeightDto
import app.indelible.api.generated.models.ThemeDto
import app.indelible.api.generated.models.TriageModeDto
import app.indelible.core.preferences.DefaultViewPreference
import app.indelible.core.preferences.ReaderFontFamilyPreference
import app.indelible.core.preferences.ReaderFontSizePreference
import app.indelible.core.preferences.ReaderLineHeightPreference
import app.indelible.core.preferences.ThemePreference
import app.indelible.core.preferences.TriageModePreference
import app.indelible.core.storage.UserPreferencesStorage
import app.indelible.profile.repository.PreferencesRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

class UserPreferencesViewModel(
    private val storage: UserPreferencesStorage,
    private val repository: PreferencesRepository,
) : ViewModel() {
    private val _theme = MutableStateFlow(ThemePreference.AUTO)
    val theme: StateFlow<ThemePreference> = _theme.asStateFlow()

    private val _defaultView = MutableStateFlow(DefaultViewPreference.LIBRARY)
    val defaultView: StateFlow<DefaultViewPreference> = _defaultView.asStateFlow()

    private val _fontFamily = MutableStateFlow(ReaderFontFamilyPreference.SERIF)
    val fontFamily: StateFlow<ReaderFontFamilyPreference> = _fontFamily.asStateFlow()

    private val _fontSize = MutableStateFlow(ReaderFontSizePreference.MEDIUM)
    val fontSize: StateFlow<ReaderFontSizePreference> = _fontSize.asStateFlow()

    private val _lineHeight = MutableStateFlow(ReaderLineHeightPreference.RELAXED)
    val lineHeight: StateFlow<ReaderLineHeightPreference> = _lineHeight.asStateFlow()

    private val _triageMode = MutableStateFlow(TriageModePreference.MANUAL)
    val triageMode: StateFlow<TriageModePreference> = _triageMode.asStateFlow()

    private val _autoAdvance = MutableStateFlow(true)
    val autoAdvance: StateFlow<Boolean> = _autoAdvance.asStateFlow()

    private val _milaEnabled = MutableStateFlow(true)
    val milaEnabled: StateFlow<Boolean> = _milaEnabled.asStateFlow()

    private val _loaded = MutableStateFlow(false)
    val loaded: StateFlow<Boolean> = _loaded.asStateFlow()

    private var cachedPreferences: PreferencesSettingsResponse? = null
    private val patchMutex = Mutex()

    init {
        // Apply local fallback values early so the first composition doesn't
        // flash the defaults (e.g. light theme before dark gets loaded). The
        // authoritative server sync is driven explicitly via [refresh] once
        // the user is authenticated.
        viewModelScope.launch {
            _theme.value = storage.getTheme()
            _defaultView.value = storage.getDefaultView()
        }
    }

    /**
     * Pull preferences from the server and publish them. Resets [loaded] to
     * false for the duration of the fetch so consumers can wait for the
     * authoritative value. Callers should invoke this whenever authentication
     * transitions to an authenticated state (cold start with a valid session,
     * or after an explicit login).
     */
    suspend fun refresh() {
        _loaded.value = false
        try {
            repository.getPreferences().onSuccess { prefs ->
                cachedPreferences = prefs
                val theme = prefs.theme.toThemePreference()
                val view = prefs.layout.defaultView.toDefaultViewPreference()
                _theme.value = theme
                _defaultView.value = view
                _fontFamily.value = prefs.reader.fontFamily.toFontFamilyPreference()
                _fontSize.value = prefs.reader.fontSize.toFontSizePreference()
                _lineHeight.value = prefs.reader.lineHeight.toLineHeightPreference()
                _triageMode.value = prefs.workflow.triageMode.toTriageModePreference()
                _autoAdvance.value = prefs.workflow.autoAdvance
                _milaEnabled.value = prefs.ai.milaEnabled
                storage.saveTheme(theme)
                storage.saveDefaultView(view)
            }
        } finally {
            _loaded.value = true
        }
    }

    fun setTheme(pref: ThemePreference) {
        _theme.value = pref
        viewModelScope.launch {
            storage.saveTheme(pref)
            patchPreferences { copy(theme = pref.toThemeDto()) }
        }
    }

    fun setDefaultView(pref: DefaultViewPreference) {
        _defaultView.value = pref
        viewModelScope.launch {
            storage.saveDefaultView(pref)
            patchPreferences { copy(layout = layout.copy(defaultView = pref.toDefaultViewDto())) }
        }
    }

    fun setFontFamily(pref: ReaderFontFamilyPreference) {
        _fontFamily.value = pref
        viewModelScope.launch {
            patchPreferences { copy(reader = reader.copy(fontFamily = pref.toFontFamilyDto())) }
        }
    }

    fun setFontSize(pref: ReaderFontSizePreference) {
        _fontSize.value = pref
        viewModelScope.launch {
            patchPreferences { copy(reader = reader.copy(fontSize = pref.toFontSizeDto())) }
        }
    }

    fun setLineHeight(pref: ReaderLineHeightPreference) {
        _lineHeight.value = pref
        viewModelScope.launch {
            patchPreferences { copy(reader = reader.copy(lineHeight = pref.toLineHeightDto())) }
        }
    }

    fun setTriageMode(pref: TriageModePreference) {
        _triageMode.value = pref
        viewModelScope.launch {
            patchPreferences { copy(workflow = workflow.copy(triageMode = pref.toTriageModeDto())) }
        }
    }

    fun setAutoAdvance(enabled: Boolean) {
        _autoAdvance.value = enabled
        viewModelScope.launch {
            patchPreferences { copy(workflow = workflow.copy(autoAdvance = enabled)) }
        }
    }

    fun setMilaEnabled(enabled: Boolean) {
        _milaEnabled.value = enabled
        viewModelScope.launch {
            patchPreferences { copy(ai = ai.copy(milaEnabled = enabled)) }
        }
    }

    private suspend fun patchPreferences(update: PreferencesSettingsResponse.() -> PreferencesSettingsResponse) {
        // Serialize PATCHes so concurrent field updates don't race on the same
        // stale `cachedPreferences` snapshot (one PATCH would overwrite the
        // other's field).
        patchMutex.withLock {
            val current =
                cachedPreferences
                    ?: repository.getPreferences().getOrNull()?.also { cachedPreferences = it }
                    ?: return
            repository.updatePreferences(current.update()).onSuccess { cachedPreferences = it }
        }
    }
}

private fun ThemeDto.toThemePreference(): ThemePreference =
    when (this) {
        ThemeDto.LIGHT -> ThemePreference.LIGHT
        ThemeDto.DARK -> ThemePreference.DARK
        ThemeDto.SYSTEM -> ThemePreference.AUTO
    }

private fun ThemePreference.toThemeDto(): ThemeDto =
    when (this) {
        ThemePreference.LIGHT -> ThemeDto.LIGHT
        ThemePreference.DARK -> ThemeDto.DARK
        ThemePreference.AUTO -> ThemeDto.SYSTEM
    }

private fun DefaultViewDto.toDefaultViewPreference(): DefaultViewPreference =
    when (this) {
        DefaultViewDto.LIBRARY -> DefaultViewPreference.LIBRARY
        DefaultViewDto.FEED -> DefaultViewPreference.FEED
        DefaultViewDto.SEARCH -> DefaultViewPreference.SEARCH
    }

private fun DefaultViewPreference.toDefaultViewDto(): DefaultViewDto =
    when (this) {
        DefaultViewPreference.LIBRARY -> DefaultViewDto.LIBRARY
        DefaultViewPreference.FEED -> DefaultViewDto.FEED
        DefaultViewPreference.SEARCH -> DefaultViewDto.SEARCH
    }

private fun ReaderFontFamilyDto.toFontFamilyPreference(): ReaderFontFamilyPreference =
    when (this) {
        ReaderFontFamilyDto.SERIF -> ReaderFontFamilyPreference.SERIF
        ReaderFontFamilyDto.SANS -> ReaderFontFamilyPreference.SANS
        ReaderFontFamilyDto.MONO -> ReaderFontFamilyPreference.MONO
    }

private fun ReaderFontFamilyPreference.toFontFamilyDto(): ReaderFontFamilyDto =
    when (this) {
        ReaderFontFamilyPreference.SERIF -> ReaderFontFamilyDto.SERIF
        ReaderFontFamilyPreference.SANS -> ReaderFontFamilyDto.SANS
        ReaderFontFamilyPreference.MONO -> ReaderFontFamilyDto.MONO
    }

private fun ReaderFontSizeDto.toFontSizePreference(): ReaderFontSizePreference =
    when (this) {
        ReaderFontSizeDto.SMALL -> ReaderFontSizePreference.SMALL
        ReaderFontSizeDto.MEDIUM -> ReaderFontSizePreference.MEDIUM
        ReaderFontSizeDto.LARGE -> ReaderFontSizePreference.LARGE
    }

private fun ReaderFontSizePreference.toFontSizeDto(): ReaderFontSizeDto =
    when (this) {
        ReaderFontSizePreference.SMALL -> ReaderFontSizeDto.SMALL
        ReaderFontSizePreference.MEDIUM -> ReaderFontSizeDto.MEDIUM
        ReaderFontSizePreference.LARGE -> ReaderFontSizeDto.LARGE
    }

private fun TriageModeDto.toTriageModePreference(): TriageModePreference =
    when (this) {
        TriageModeDto.MANUAL -> TriageModePreference.MANUAL
        TriageModeDto.FOCUS -> TriageModePreference.FOCUS
    }

private fun TriageModePreference.toTriageModeDto(): TriageModeDto =
    when (this) {
        TriageModePreference.MANUAL -> TriageModeDto.MANUAL
        TriageModePreference.FOCUS -> TriageModeDto.FOCUS
    }

private fun ReaderLineHeightDto.toLineHeightPreference(): ReaderLineHeightPreference =
    when (this) {
        ReaderLineHeightDto.COMPACT -> ReaderLineHeightPreference.COMPACT
        ReaderLineHeightDto.RELAXED -> ReaderLineHeightPreference.RELAXED
    }

private fun ReaderLineHeightPreference.toLineHeightDto(): ReaderLineHeightDto =
    when (this) {
        ReaderLineHeightPreference.COMPACT -> ReaderLineHeightDto.COMPACT
        ReaderLineHeightPreference.RELAXED -> ReaderLineHeightDto.RELAXED
    }
