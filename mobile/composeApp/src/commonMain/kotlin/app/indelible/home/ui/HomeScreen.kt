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
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import app.indelible.core.i18n.resolve
import app.indelible.home.ui.components.ContinueReadingHero
import app.indelible.home.ui.components.EmptyContinueReadingHero
import app.indelible.home.ui.components.GreetingHeader
import app.indelible.home.ui.components.HomeAppBar
import app.indelible.home.ui.components.HomeZeroedSection
import app.indelible.home.ui.components.JumpBackRail
import app.indelible.home.ui.components.RecentlySavedRow
import app.indelible.home.ui.components.StatsRow
import app.indelible.home.viewmodel.HomeUiState
import app.indelible.home.viewmodel.HomeViewModel
import app.indelible.home.viewmodel.StatIcon
import app.indelible.home.viewmodel.StatTile
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_retry
import indelible.composeapp.generated.resources.home_jump_back
import indelible.composeapp.generated.resources.home_jump_back_empty
import indelible.composeapp.generated.resources.home_recent_empty
import indelible.composeapp.generated.resources.home_recently_saved
import indelible.composeapp.generated.resources.home_stat_finished
import indelible.composeapp.generated.resources.home_stat_read
import indelible.composeapp.generated.resources.home_stat_streak
import org.jetbrains.compose.resources.stringResource

/**
 * The home dashboard: greeting, the continue-reading hero, reading stats, a
 * jump-back rail, and recently-saved rows. Backed by [HomeViewModel]; the
 * display name is supplied by the caller (the dashboard endpoint carries no
 * name), as is every navigation callback.
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
                        text = state.message.resolve(),
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.error,
                        textAlign = TextAlign.Center,
                    )
                    Spacer(Modifier.height(IndelibleSpacing.step16))
                    IndelibleButton(text = stringResource(Res.string.common_retry), onClick = viewModel::load)
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
    val isZeroed =
        state.continueReading == null &&
            state.jumpBack.isEmpty() &&
            state.recentlySaved.isEmpty()
    val visibleStats = if (state.stats.isEmpty() && isZeroed) zeroStats else state.stats

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

        item {
            state.continueReading?.let { hero ->
                ContinueReadingHero(
                    item = hero,
                    onResume = { onOpenItem(hero.id) },
                    onOpen = { onOpenItem(hero.id) },
                    modifier = Modifier.padding(horizontal = IndelibleSpacing.step20),
                )
            } ?: EmptyContinueReadingHero(
                modifier = Modifier.padding(horizontal = IndelibleSpacing.step20),
            )
        }

        if (visibleStats.isNotEmpty()) {
            item {
                StatsRow(
                    stats = visibleStats,
                    zeroed = isZeroed,
                    modifier =
                        Modifier.padding(
                            horizontal = IndelibleSpacing.step20,
                            vertical = IndelibleSpacing.step20,
                        ),
                )
            }
        }

        if (state.jumpBack.isEmpty()) {
            item {
                HomeSectionHeader(
                    title = stringResource(Res.string.home_jump_back),
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
                HomeZeroedSection(
                    message = stringResource(Res.string.home_jump_back_empty),
                    modifier = Modifier.padding(horizontal = IndelibleSpacing.step20),
                )
            }
        } else {
            item {
                HomeSectionHeader(
                    title = stringResource(Res.string.home_jump_back),
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

        item {
            HomeSectionHeader(
                title = stringResource(Res.string.home_recently_saved),
                modifier =
                    Modifier.padding(
                        start = IndelibleSpacing.step20,
                        end = IndelibleSpacing.step20,
                        top = IndelibleSpacing.step24,
                        bottom =
                            if (state.recentlySaved.isEmpty()) {
                                IndelibleSpacing.step12
                            } else {
                                IndelibleSpacing.step4
                            },
                    ),
            )
        }
        if (state.recentlySaved.isEmpty()) {
            item {
                HomeZeroedSection(
                    message = stringResource(Res.string.home_recent_empty),
                    modifier = Modifier.padding(horizontal = IndelibleSpacing.step20),
                )
            }
        } else {
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

private val zeroStats =
    listOf(
        StatTile(labelRes = Res.string.home_stat_read, value = 0, icon = StatIcon.READING_TIME),
        StatTile(labelRes = Res.string.home_stat_finished, value = 0, icon = StatIcon.ITEMS_COMPLETED),
        StatTile(labelRes = Res.string.home_stat_streak, value = 0, icon = StatIcon.STREAK),
    )
