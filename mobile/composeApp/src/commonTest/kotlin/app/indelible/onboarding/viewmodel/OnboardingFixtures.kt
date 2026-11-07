package app.indelible.onboarding.viewmodel

import app.indelible.core.storage.InMemoryTokenStorage
import kotlinx.coroutines.runBlocking

private val STEP_NAMES = listOf("Account Setup", "Add Content", "RSS Feeds", "AI Configuration", "Complete")

internal fun onboardingStatusJson(
    currentStep: Int,
    completedSteps: Int,
): String {
    val steps =
        STEP_NAMES.mapIndexed { index, name ->
            """{"step": ${index + 1}, "name": "$name", "completed": ${index < completedSteps}}"""
        }
    return """{"current_step": $currentStep, "completed": ${completedSteps == STEP_NAMES.size}, """ +
        """"steps": [${steps.joinToString(", ")}]}"""
}

internal val onboardingStatusResponse = onboardingStatusJson(currentStep = 0, completedSteps = 0)
internal val step1CompletedResponse = onboardingStatusJson(currentStep = 1, completedSteps = 1)
internal val partiallyCompletedStatusResponse = onboardingStatusJson(currentStep = 3, completedSteps = 3)
internal val allCompletedResponse = onboardingStatusJson(currentStep = 5, completedSteps = 5)

internal fun createTokenStorageWithToken(): InMemoryTokenStorage {
    val storage = InMemoryTokenStorage()
    runBlocking { storage.saveToken("test-token") }
    return storage
}
