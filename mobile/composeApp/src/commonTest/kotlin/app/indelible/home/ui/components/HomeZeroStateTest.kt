package app.indelible.home.ui.components

import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.runComposeUiTest
import androidx.compose.ui.unit.dp
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test

@OptIn(ExperimentalTestApi::class)
class HomeZeroStateTest {
    @Test
    fun empty_continue_reading_keeps_the_hero_without_an_action() =
        runComposeUiTest {
            setContent {
                AppTheme {
                    EmptyContinueReadingHero(
                        modifier = Modifier.width(360.dp).height(240.dp),
                    )
                }
            }

            onNodeWithText("Nothing in progress").assertIsDisplayed()
            onNodeWithText("Whatever you start reading waits for you here").assertIsDisplayed()
            onNodeWithText("Resumes at the exact paragraph").assertIsDisplayed()
        }

    @Test
    fun empty_home_sections_explain_what_will_fill_them() =
        runComposeUiTest {
            setContent {
                AppTheme {
                    HomeZeroedSection(
                        message = "Nothing started yet. Anything you leave half-read shows up in this row.",
                        modifier = Modifier.width(360.dp).height(120.dp),
                    )
                }
            }

            onNodeWithText("Nothing started yet. Anything you leave half-read shows up in this row.")
                .assertIsDisplayed()
        }
}
