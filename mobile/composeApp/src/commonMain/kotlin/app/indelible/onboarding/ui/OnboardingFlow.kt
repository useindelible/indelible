package app.indelible.onboarding.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.PagerState
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import app.indelible.core.model.StepData
import app.indelible.onboarding.ui.components.PagerIndicator
import app.indelible.onboarding.viewmodel.OnboardingPage
import app.indelible.onboarding.viewmodel.OnboardingState
import app.indelible.onboarding.viewmodel.OnboardingViewModel
import app.indelible.ui.theme.IndelibleSpacing
import kotlinx.coroutines.launch

private const val PAGE_COUNT = 6

@Composable
fun OnboardingFlow(
    viewModel: OnboardingViewModel,
    onComplete: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsState()
    val pagerState =
        rememberPagerState(
            initialPage = state.currentPage,
            pageCount = { PAGE_COUNT },
        )
    val coroutineScope = rememberCoroutineScope()

    LaunchedEffect(Unit) {
        viewModel.initialize()
    }

    LaunchedEffect(state.currentPage) {
        if (state.currentPage != pagerState.currentPage && !state.isLoading) {
            pagerState.animateScrollToPage(state.currentPage)
        }
    }

    LaunchedEffect(state.isCompleted) {
        if (state.isCompleted) {
            onComplete()
        }
    }

    if (state.isLoading) {
        Box(
            modifier = Modifier.fillMaxSize(),
            contentAlignment = Alignment.Center,
        ) {
            CircularProgressIndicator()
        }
        return
    }

    val advancePage: () -> Unit = {
        coroutineScope.launch {
            if (pagerState.currentPage < PAGE_COUNT - 1) {
                pagerState.animateScrollToPage(pagerState.currentPage + 1)
            }
        }
    }

    Scaffold(
        modifier = modifier,
        topBar = {
            OnboardingTopBar(pagerState = pagerState, onSkip = { viewModel.skipAll() })
        },
    ) { paddingValues ->
        Column(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(paddingValues),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            HorizontalPager(
                state = pagerState,
                modifier =
                    Modifier
                        .weight(1f)
                        .fillMaxWidth(),
            ) { page ->
                OnboardingPageContent(
                    page = page,
                    state = state,
                    viewModel = viewModel,
                    advancePage = advancePage,
                )
            }

            PagerIndicator(
                pagerState = pagerState,
                pageCount = PAGE_COUNT,
                modifier = Modifier.padding(IndelibleSpacing.step16),
            )
        }
    }
}

@Composable
private fun OnboardingTopBar(
    pagerState: PagerState,
    onSkip: () -> Unit,
) {
    Box(
        modifier =
            Modifier
                .fillMaxWidth()
                .padding(horizontal = IndelibleSpacing.step16, vertical = IndelibleSpacing.step8),
        contentAlignment = Alignment.CenterEnd,
    ) {
        if (pagerState.currentPage < PAGE_COUNT - 1) {
            TextButton(onClick = onSkip) {
                Text("Skip All")
            }
        }
    }
}

@Composable
private fun OnboardingPageContent(
    page: Int,
    state: OnboardingState,
    viewModel: OnboardingViewModel,
    advancePage: () -> Unit,
) {
    when (OnboardingPage.entries[page]) {
        OnboardingPage.WELCOME ->
            WelcomeStep(
                onContinue = advancePage,
            )
        OnboardingPage.ACCOUNT_SETUP ->
            AccountSetupStep(
                viewModel = viewModel,
                displayName = state.displayName,
                selectedTheme = state.selectedTheme,
                onContinue = {
                    viewModel.completeAccountStep(onSuccess = advancePage)
                },
                onSkip = {
                    viewModel.completeStep(1, onSuccess = advancePage)
                },
            )
        OnboardingPage.ADD_CONTENT ->
            AddContentStep(
                urlInput = state.urlInput,
                onUrlChange = viewModel::updateUrlInput,
                onContinue = {
                    viewModel.completeAddContentStep(onSuccess = advancePage)
                },
                onSkip = {
                    viewModel.completeStep(2, onSuccess = advancePage)
                },
            )
        OnboardingPage.FEEDS ->
            FeedsStep(
                selectedFeeds = state.selectedFeeds,
                rssUrlInput = state.rssUrlInput,
                onToggleFeed = viewModel::toggleFeed,
                onRssUrlChange = viewModel::updateRssUrlInput,
                onContinue = {
                    val urls =
                        buildList {
                            addAll(state.selectedFeeds)
                            if (state.rssUrlInput.isNotBlank()) add(state.rssUrlInput)
                        }
                    viewModel.completeStep(3, StepData(feedUrls = urls), advancePage)
                },
                onSkip = {
                    viewModel.completeStep(3, onSuccess = advancePage)
                },
            )
        OnboardingPage.AI_SETUP ->
            AiSetupStep(
                selectedProvider = state.selectedAiProvider,
                apiKeyInput = state.apiKeyInput,
                onSelectProvider = viewModel::updateSelectedAiProvider,
                onApiKeyChange = viewModel::updateApiKeyInput,
                onContinue = {
                    viewModel.completeStep(
                        4,
                        StepData(
                            chatProvider = state.selectedAiProvider.name.lowercase(),
                            embeddingProvider = state.selectedAiProvider.name.lowercase(),
                            chatApiKey = state.apiKeyInput.takeIf { it.isNotBlank() },
                            embeddingApiKey = state.apiKeyInput.takeIf { it.isNotBlank() },
                        ),
                        onSuccess = advancePage,
                    )
                },
                onSkip = {
                    viewModel.completeStep(4, onSuccess = advancePage)
                },
            )
        OnboardingPage.READY ->
            ReadyStep(
                onComplete = {
                    viewModel.completeStep(5)
                },
            )
    }
}
