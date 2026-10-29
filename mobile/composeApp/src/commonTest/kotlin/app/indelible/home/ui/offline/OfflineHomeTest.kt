package app.indelible.home.ui.offline

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.hasClickAction
import androidx.compose.ui.test.hasScrollAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.onFirst
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollToNode
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.home.ui.offline.OfflineSection.CONTINUE
import app.indelible.home.ui.offline.OfflineSection.EMPTY_SHELF
import app.indelible.home.ui.offline.OfflineSection.HERO
import app.indelible.home.ui.offline.OfflineSection.HINT
import app.indelible.home.ui.offline.OfflineSection.LEDE
import app.indelible.home.ui.offline.OfflineSection.PICK_UP
import app.indelible.home.ui.offline.OfflineSection.RECENT
import app.indelible.home.ui.offline.OfflineSection.SHELF
import app.indelible.home.viewmodel.Greeting
import app.indelible.home.viewmodel.OfflineShelfState
import app.indelible.home.viewmodel.ShelfItem
import app.indelible.home.viewmodel.ShelfNet
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleTheme
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalTestApi::class)
class OfflineHomeTest {
    private fun item(
        id: String,
        onPhone: Boolean = true,
    ) = ShelfItem(id, "Title $id", "pdf", pinned = false, progressPercent = null, onPhone = onPhone)

    private val copy = item("doc_kept")
    private val seen = item("doc_seen", onPhone = false)

    @Test
    fun sectionsSitBetweenTheHeroAndTheShelf() {
        val shelf =
            OfflineShelfState(
                ShelfNet.OFFLINE,
                listOf(copy),
                continueReading = listOf(seen),
                recentlySaved = listOf(seen),
            )

        assertEquals(listOf(LEDE, HERO, PICK_UP, CONTINUE, RECENT, SHELF, HINT), offlineSections(shelf))
    }

    @Test
    fun emptySectionsAreHidden() {
        val shelf = OfflineShelfState(ShelfNet.OFFLINE, listOf(copy), recentlySaved = listOf(seen))

        assertEquals(listOf(LEDE, HERO, PICK_UP, RECENT, SHELF, HINT), offlineSections(shelf))
    }

    @Test
    fun nothingOnThePhoneShowsTheEmptyShelfUnderTheSections() {
        val shelf = OfflineShelfState(ShelfNet.OFFLINE, continueReading = listOf(seen))

        assertEquals(listOf(LEDE, HERO, CONTINUE, EMPTY_SHELF), offlineSections(shelf))
    }

    @Test
    fun nothingOnThePhoneAndNothingLoadedShowsTheEmptyShelf() {
        assertEquals(listOf(LEDE, HERO, EMPTY_SHELF), offlineSections(OfflineShelfState(ShelfNet.OFFLINE)))
    }

    @Test
    fun tapOnAContinueItemOpensIt() =
        runComposeUiTest {
            var opened: String? = null
            val shelf = OfflineShelfState(ShelfNet.OFFLINE, listOf(copy), continueReading = listOf(seen))
            setContent {
                AppTheme {
                    OfflineHome(shelf, Greeting.MORNING, "Sam", PaddingValues(), onOpenItem = { opened = it })
                }
            }

            onAllNodes(hasScrollAction()).onFirst().performScrollToNode(hasText("Title doc_seen"))
            onNodeWithText("Title doc_seen").performClick()

            assertEquals("doc_seen", opened)
        }

    @Test
    fun aLedgeObjectReadsItsTitleOnceAsAButton() =
        runComposeUiTest {
            setContent {
                AppTheme {
                    val mono = IndelibleTheme.monochrome
                    LedgeObject(seen.copy(domain = "example.com", documentType = "article"), mono, onOpen = {})
                }
            }

            val node = onNode(hasClickAction()).fetchSemanticsNode()

            val spoken = node.config[SemanticsProperties.Text].map { it.text }
            assertEquals(listOf("Title doc_seen", "Not on phone"), spoken)
            assertEquals(Role.Button, node.config[SemanticsProperties.Role])
        }
}
