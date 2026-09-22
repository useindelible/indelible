package app.indelible.home.ui.components

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.home.viewmodel.FakeHomeRepository
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test

@OptIn(ExperimentalTestApi::class)
class ContinueReadingHeroTest {
    @Test
    fun progressReadsAsASinglePercentSign() =
        runComposeUiTest {
            setContent {
                AppTheme {
                    ContinueReadingHero(
                        item = FakeHomeRepository.item(progressPercent = 62f),
                        onResume = {},
                        onOpen = {},
                    )
                }
            }

            onNodeWithText("62%").assertIsDisplayed()
        }
}
