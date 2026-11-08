package app.indelible.offline.ui

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.ui.theme.AppTheme
import kotlinx.datetime.Instant
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.time.Duration.Companion.minutes

private val NOW = Instant.fromEpochMilliseconds(1_767_225_600_000L)

@OptIn(ExperimentalTestApi::class)
class NoteConflictSheetTest {
    @Test
    fun showsBothBodiesWithTheirOrigins() =
        runComposeUiTest {
            setContent {
                AppTheme {
                    NoteConflictContent(
                        mine = "mine words",
                        theirs = "their words",
                        theirsAt = NOW - 5.minutes,
                        now = NOW,
                        onKeepMine = {},
                        onKeepOther = {},
                    )
                }
            }

            onNodeWithText("This device · written offline").assertExists()
            onNodeWithText("mine words").assertExists()
            onNodeWithText("Other device · 5 min ago").assertExists()
            onNodeWithText("their words").assertExists()
        }

    @Test
    fun keepMineCallsBack() =
        runComposeUiTest {
            var mine = 0
            var other = 0
            setContent {
                AppTheme {
                    NoteConflictContent("mine", "theirs", NOW, NOW, onKeepMine = { mine++ }, onKeepOther = { other++ })
                }
            }

            onNodeWithText("Keep mine").performClick()

            assertEquals(1 to 0, mine to other)
        }

    @Test
    fun keepOtherCallsBack() =
        runComposeUiTest {
            var mine = 0
            var other = 0
            setContent {
                AppTheme {
                    NoteConflictContent("mine", "theirs", NOW, NOW, onKeepMine = { mine++ }, onKeepOther = { other++ })
                }
            }

            onNodeWithText("Keep other").performClick()

            assertEquals(0 to 1, mine to other)
        }
}
