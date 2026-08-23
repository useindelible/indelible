package app.indelible.profile.ui

import androidx.compose.foundation.layout.width
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import androidx.compose.ui.unit.dp
import app.indelible.core.i18n.AppLanguage
import app.indelible.core.i18n.AppLanguageSettings
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test
import kotlin.test.assertTrue

@OptIn(ExperimentalTestApi::class)
class LanguageSettingsRowTest {
    @Test
    fun language_row_opens_the_language_choices_and_selects_one() =
        runComposeUiTest {
            var selected: AppLanguage? = null
            setContent {
                AppTheme {
                    LanguageSettingsRow(
                        settings = AppLanguageSettings.Selectable(AppLanguage.FRENCH) { selected = it },
                        modifier = Modifier.width(360.dp),
                    )
                }
            }

            onNodeWithText("French").performClick()
            onNodeWithText("English").assertIsDisplayed().performClick()
            runOnIdle { assertTrue(selected == AppLanguage.ENGLISH) }
        }

    @Test
    fun language_row_shows_the_effective_language_and_opens_settings() =
        runComposeUiTest {
            var opened = false
            setContent {
                AppTheme {
                    LanguageSettingsRow(
                        settings = AppLanguageSettings.SystemManaged(AppLanguage.FRENCH) { opened = true },
                        modifier = Modifier.width(360.dp),
                    )
                }
            }

            onNodeWithText("Language").assertIsDisplayed()
            onNodeWithText("French").assertIsDisplayed().performClick()
            runOnIdle { assertTrue(opened) }
        }
}
