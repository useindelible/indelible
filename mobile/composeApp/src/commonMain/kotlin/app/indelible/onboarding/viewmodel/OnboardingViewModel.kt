package app.indelible.onboarding.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.core.model.StepData
import app.indelible.onboarding.repository.OnboardingRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class OnboardingViewModel(
    private val repository: OnboardingRepository,
) : ViewModel() {
    private val _state = MutableStateFlow(OnboardingState())
    val state: StateFlow<OnboardingState> = _state.asStateFlow()

    fun initialize() {
        viewModelScope.launch {
            _state.value = _state.value.copy(isLoading = true, error = null)
            repository
                .getOnboardingStatus()
                .onSuccess { response ->
                    val steps =
                        response.steps.map { dto ->
                            OnboardingStep(
                                number = dto.step,
                                name = dto.name,
                                completed = dto.completed,
                            )
                        }
                    val firstIncomplete = steps.firstOrNull { !it.completed }?.number
                    val resumePage =
                        if (steps.none { it.completed }) {
                            OnboardingPage.WELCOME.ordinal
                        } else {
                            firstIncomplete?.let { step ->
                            OnboardingPage.entries.indexOfFirst { it.backendStep == step }
                            } ?: OnboardingPage.READY.ordinal
                        }
                    _state.value =
                        _state.value.copy(
                            steps = steps,
                            currentPage = resumePage,
                            isLoading = false,
                            isCompleted = response.completed,
                        )
                }.onFailure { error ->
                    _state.value =
                        _state.value.copy(
                            isLoading = false,
                            error = error.message ?: "Failed to load onboarding status",
                            steps = defaultSteps(),
                        )
                }
        }
    }

    fun completeStep(
        stepNumber: Int,
        data: StepData = StepData(),
        onSuccess: () -> Unit = {},
    ) {
        val currentSteps = _state.value.steps
        val stepIndex = currentSteps.indexOfFirst { it.number == stepNumber }
        if (stepIndex < 0) return
        if (currentSteps[stepIndex].completed) return

        viewModelScope.launch {
            _state.value = _state.value.copy(isStepLoading = true, error = null)
            repository
                .completeOnboardingStep(stepNumber, data)
                .onSuccess { response ->
                    val steps =
                        response.steps.map { dto ->
                            OnboardingStep(
                                number = dto.step,
                                name = dto.name,
                                completed = dto.completed,
                            )
                        }
                    _state.value =
                        _state.value.copy(
                            steps = steps,
                            isStepLoading = false,
                            isCompleted = response.completed,
                        )
                    onSuccess()
                }.onFailure { error ->
                    _state.value =
                        _state.value.copy(
                            isStepLoading = false,
                            error = error.message ?: "Failed to complete step",
                        )
                }
        }
    }

    fun skipAll() {
        viewModelScope.launch {
            _state.value = _state.value.copy(isStepLoading = true, error = null)
            repository
                .skipOnboarding()
                .onSuccess { response ->
                    val steps =
                        response.steps.map { dto ->
                            OnboardingStep(
                                number = dto.step,
                                name = dto.name,
                                completed = dto.completed,
                            )
                        }
                    _state.value =
                        _state.value.copy(
                            steps = steps,
                            isStepLoading = false,
                            isCompleted = response.completed,
                        )
                }.onFailure { error ->
                    _state.value =
                        _state.value.copy(
                            isStepLoading = false,
                            error = error.message ?: "Failed to skip onboarding",
                        )
                }
        }
    }

    fun updateDisplayName(name: String) {
        _state.value = _state.value.copy(displayName = name)
    }

    fun updateSelectedTheme(theme: ThemeChoice) {
        _state.value = _state.value.copy(selectedTheme = theme)
    }

    fun updateUrlInput(url: String) {
        _state.value = _state.value.copy(urlInput = url)
    }

    fun updateRssUrlInput(url: String) {
        _state.value = _state.value.copy(rssUrlInput = url)
    }

    fun updateApiKeyInput(key: String) {
        _state.value = _state.value.copy(apiKeyInput = key)
    }

    fun toggleFeed(feedUrl: String) {
        val current = _state.value.selectedFeeds
        val updated =
            if (current.contains(feedUrl)) {
                current - feedUrl
            } else {
                current + feedUrl
            }
        _state.value = _state.value.copy(selectedFeeds = updated)
    }

    fun updateSelectedAiProvider(provider: AiProvider) {
        _state.value = _state.value.copy(selectedAiProvider = provider)
    }

    fun updateCurrentPage(page: Int) {
        _state.value = _state.value.copy(currentPage = page)
    }

    fun clearError() {
        _state.value = _state.value.copy(error = null)
    }

    companion object {
        fun defaultSteps(): List<OnboardingStep> =
            OnboardingPage.entries.mapNotNull {
                it.backendStep?.let { step -> OnboardingStep(step, it.pageName, false) }
            }
    }
}
