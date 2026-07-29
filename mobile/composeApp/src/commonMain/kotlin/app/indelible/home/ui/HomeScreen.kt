package app.indelible.home.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import app.indelible.home.ui.components.ContinueReadingHero
import app.indelible.home.ui.components.GreetingHeader
import app.indelible.home.ui.components.HomeAppBar
import app.indelible.home.ui.components.HomeFab
import app.indelible.home.ui.components.JumpBackRail
import app.indelible.home.ui.components.RecentlySavedRow
import app.indelible.home.ui.components.StatsRow
import app.indelible.home.viewmodel.HomeUiState
import app.indelible.home.viewmodel.HomeViewModel
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.theme.IndelibleSpacing
import kotlinx.coroutines.launch

/**
 * The home dashboard: greeting, the continue-reading hero, reading stats, a
 * jump-back rail, and recently-saved rows, with a save FAB. Backed by
 * [HomeViewModel]; the display name is supplied by the caller (the dashboard
 * endpoint carries no name), as is every navigation callback. The save FAB
 * surfaces a "Coming soon" notice, mirroring the library Add action, until a
 * save flow exists.
 */
@Composable
fun HomeScreen(
    viewModel: HomeViewModel,
    userDisplayName: String?,
    onMenuClick: () -> Unit,
    onSearchClick: () -> Unit,
    onProfileClick: () -> Unit,
    onOpenItem: (String) -> Unit,
    modifier: Modifier = Modifier,
    avatarUrl: String? = null,
    avatarBytes: ByteArray? = null,
) {
    val uiState by viewModel.uiState.collectAsState()
    val firstName = userDisplayName?.trim()?.takeIf { it.isNotBlank() }?.substringBefore(' ')
    val snackbarHostState = remember { SnackbarHostState() }
    val coroutineScope = rememberCoroutineScope()

    Scaffold(
        modifier = modifier,
        containerColor = MaterialTheme.colorScheme.background,
        topBar = {
            HomeAppBar(
                userDisplayName = userDisplayName.orEmpty(),
                onMenuClick = onMenuClick,
                onSearchClick = onSearchClick,
                onProfileClick = onProfileClick,
                avatarUrl = avatarUrl,
                avatarBytes = avatarBytes,
            )
        },
        floatingActionButton = {
            HomeFab(
                onClick = {
                    coroutineScope.launch { snackbarHostState.showSnackbar("Coming soon") }
                },
            )
        },
        snackbarHost = { SnackbarHost(snackbarHostState) },
    ) { paddingValues ->
        when (val state = uiState) {
            is HomeUiState.Loading ->
                Box(
                    modifier = Modifier.fillMaxSize().padding(paddingValues),
                    contentAlignment = Alignment.Center,
                ) {
                    CircularProgressIndicator()
                }

            is HomeUiState.Error ->
                Column(
                    modifier =
                        Modifier
                            .fillMaxSize()
                            .padding(paddingValues)
                            .padding(IndelibleSpacing.step24),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.Center,
                ) {
                    Text(
                        text = state.message,
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.error,
                        textAlign = TextAlign.Center,
                    )
                    Spacer(Modifier.height(IndelibleSpacing.step16))
                    IndelibleButton(text = "Retry", onClick = viewModel::load)
                }

            is HomeUiState.Ready ->
                HomeContent(
                    state = state,
                    firstName = firstName,
                    onOpenItem = onOpenItem,
                    contentPadding = paddingValues,
                )
        }
    }
}

@Composable
private fun HomeContent(
    state: HomeUiState.Ready,
    firstName: String?,
    onOpenItem: (String) -> Unit,
    contentPadding: PaddingValues,
) {
    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding =
            PaddingValues(
                top = contentPadding.calculateTopPadding(),
                bottom = contentPadding.calculateBottomPadding() + IndelibleSpacing.step80,
            ),
    ) {
        item {
            GreetingHeader(
                greeting = state.greeting,
                name = firstName,
                modifier =
                    Modifier.padding(
                        horizontal = IndelibleSpacing.step20,
                        vertical = IndelibleSpacing.step16,
                    ),
            )
        }

        state.continueReading?.let { hero ->
            item {
                ContinueReadingHero(
                    item = hero,
                    onResume = { onOpenItem(hero.id) },
                    onOpen = { onOpenItem(hero.id) },
                    modifier = Modifier.padding(horizontal = IndelibleSpacing.step20),
                )
            }
        }

        if (state.stats.isNotEmpty()) {
            item {
                StatsRow(
                    stats = state.stats,
                    modifier =
                        Modifier.padding(
                            horizontal = IndelibleSpacing.step20,
                            vertical = IndelibleSpacing.step20,
                        ),
                )
            }
        }

        if (state.jumpBack.isNotEmpty()) {
            item {
                HomeSectionHeader(
                    title = "Jump back in",
                    modifier =
                        Modifier.padding(
                            start = IndelibleSpacing.step20,
                            end = IndelibleSpacing.step20,
                            top = IndelibleSpacing.step24,
                            bottom = IndelibleSpacing.step12,
                        ),
                )
            }
            item {
                JumpBackRail(items = state.jumpBack, onItem = { onOpenItem(it.id) })
            }
        }

        if (state.recentlySaved.isNotEmpty()) {
            item {
                HomeSectionHeader(
                    title = "Recently saved",
                    modifier =
                        Modifier.padding(
                            start = IndelibleSpacing.step20,
                            end = IndelibleSpacing.step20,
                            top = IndelibleSpacing.step24,
                            bottom = IndelibleSpacing.step4,
                        ),
                )
            }
            itemsIndexed(state.recentlySaved, key = { _, item -> item.id }) { index, item ->
                RecentlySavedRow(
                    item = item,
                    onClick = { onOpenItem(item.id) },
                    showDivider = index < state.recentlySaved.lastIndex,
                )
            }
        }
    }
}

@Composable
private fun HomeSectionHeader(
    title: String,
    modifier: Modifier = Modifier,
) {
    Text(
        text = title,
        style = MaterialTheme.typography.titleLarge,
        color = MaterialTheme.colorScheme.onSurface,
        modifier = modifier,
    )
}
