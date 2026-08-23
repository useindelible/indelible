package app.indelible.onboarding.viewmodel

import app.indelible.core.i18n.UiMessage
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.onboarding_ai_none
import indelible.composeapp.generated.resources.onboarding_ai_ollama
import indelible.composeapp.generated.resources.onboarding_ai_openai
import indelible.composeapp.generated.resources.onboarding_feeds_description_ars
import indelible.composeapp.generated.resources.onboarding_feeds_description_hacker_news
import indelible.composeapp.generated.resources.onboarding_feeds_description_verge
import indelible.composeapp.generated.resources.onboarding_feeds_description_wired
import indelible.composeapp.generated.resources.onboarding_step_account
import indelible.composeapp.generated.resources.onboarding_step_add_content
import indelible.composeapp.generated.resources.onboarding_step_ai
import indelible.composeapp.generated.resources.onboarding_step_feeds
import indelible.composeapp.generated.resources.onboarding_step_ready
import indelible.composeapp.generated.resources.onboarding_step_welcome
import org.jetbrains.compose.resources.StringResource

data class OnboardingState(
    val steps: List<OnboardingStep> = emptyList(),
    val currentPage: Int = 0,
    val isLoading: Boolean = false,
    val isStepLoading: Boolean = false,
    val isCompleted: Boolean = false,
    val error: UiMessage? = null,
    val displayName: String = "",
    val selectedTheme: ThemeChoice = ThemeChoice.AUTO,
    val urlInput: String = "",
    val rssUrlInput: String = "",
    val selectedFeeds: Set<String> = emptySet(),
    val selectedAiProvider: AiProvider = AiProvider.NONE,
    val apiKeyInput: String = "",
)

data class OnboardingStep(
    val number: Int,
    val completed: Boolean,
)

enum class OnboardingPage(
    val labelRes: StringResource,
    val backendStep: Int?,
) {
    WELCOME(Res.string.onboarding_step_welcome, null),
    ACCOUNT_SETUP(Res.string.onboarding_step_account, 1),
    ADD_CONTENT(Res.string.onboarding_step_add_content, 2),
    FEEDS(Res.string.onboarding_step_feeds, 3),
    AI_SETUP(Res.string.onboarding_step_ai, 4),
    READY(Res.string.onboarding_step_ready, 5),
}

enum class ThemeChoice {
    LIGHT,
    DARK,
    AUTO,
}

enum class AiProvider(
    val labelRes: StringResource,
) {
    NONE(Res.string.onboarding_ai_none),
    OLLAMA(Res.string.onboarding_ai_ollama),
    OPENAI(Res.string.onboarding_ai_openai),
}

data class SuggestedFeed(
    val title: String,
    val url: String,
    val descriptionRes: StringResource,
)

val DEFAULT_SUGGESTED_FEEDS =
    listOf(
        SuggestedFeed(
            title = "Hacker News", // i18n-ignore: curated publication name
            url = "https://news.ycombinator.com/rss",
            descriptionRes = Res.string.onboarding_feeds_description_hacker_news,
        ),
        SuggestedFeed(
            title = "Ars Technica", // i18n-ignore: curated publication name
            url = "https://feeds.arstechnica.com/arstechnica/index",
            descriptionRes = Res.string.onboarding_feeds_description_ars,
        ),
        SuggestedFeed(
            title = "The Verge", // i18n-ignore: curated publication name
            url = "https://www.theverge.com/rss/index.xml",
            descriptionRes = Res.string.onboarding_feeds_description_verge,
        ),
        SuggestedFeed(
            title = "Wired", // i18n-ignore: curated publication name
            url = "https://www.wired.com/feed/rss",
            descriptionRes = Res.string.onboarding_feeds_description_wired,
        ),
    )
