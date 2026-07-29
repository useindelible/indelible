package app.indelible.onboarding.viewmodel

data class OnboardingState(
    val steps: List<OnboardingStep> = emptyList(),
    val currentPage: Int = 0,
    val isLoading: Boolean = false,
    val isStepLoading: Boolean = false,
    val isCompleted: Boolean = false,
    val error: String? = null,
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
    val name: String,
    val completed: Boolean,
)

enum class OnboardingPage(
    val pageName: String,
    val backendStep: Int?,
) {
    WELCOME("Welcome", null),
    ACCOUNT_SETUP("Account Setup", 1),
    ADD_CONTENT("Add Content", 2),
    FEEDS("RSS Feeds", 3),
    AI_SETUP("AI Setup", 4),
    READY("Ready", 5),
    ;
}

enum class ThemeChoice {
    LIGHT,
    DARK,
    AUTO,
}

enum class AiProvider(
    val label: String,
) {
    NONE("None"),
    OLLAMA("Ollama"),
    OPENAI("OpenAI"),
}

data class SuggestedFeed(
    val title: String,
    val url: String,
    val description: String,
)

val DEFAULT_SUGGESTED_FEEDS =
    listOf(
        SuggestedFeed(
            title = "Hacker News",
            url = "https://news.ycombinator.com/rss",
            description = "Tech news and discussion",
        ),
        SuggestedFeed(
            title = "Ars Technica",
            url = "https://feeds.arstechnica.com/arstechnica/index",
            description = "Technology, science, and culture",
        ),
        SuggestedFeed(
            title = "The Verge",
            url = "https://www.theverge.com/rss/index.xml",
            description = "Technology, science, art, and culture",
        ),
        SuggestedFeed(
            title = "Wired",
            url = "https://www.wired.com/feed/rss",
            description = "Future technology and culture",
        ),
    )
