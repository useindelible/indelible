package app.indelible.offline.ui

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsNotSelected
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import app.indelible.core.offline.CatalogEntry
import app.indelible.core.offline.OutboxKind
import app.indelible.offline.viewmodel.ChangeStatus
import app.indelible.offline.viewmodel.PendingChange
import app.indelible.offline.viewmodel.StorageUiState
import app.indelible.ui.theme.AppTheme
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalTestApi::class)
class StorageScreenTest {
    private val cached =
        CatalogEntry(
            documentId = "doc_a",
            documentType = "article",
            title = "Cached article",
            pinned = false,
            lastOpenedAt = 1L,
            lastSyncedAt = null,
            bytes = 2_048L,
            generation = 1L,
        )

    @Test
    fun cachedRowOpensReader() =
        runComposeUiTest {
            var opened: String? = null
            setContent {
                AppTheme {
                    StorageContent(
                        StorageUiState(cached = listOf(cached)),
                        StorageActions(onOpenReader = { opened = it }),
                    )
                }
            }

            onNodeWithText("Cached article").performClick()

            assertEquals("doc_a", opened)
        }

    @Test
    fun onlyRetryableChangesOfferRetry() =
        runComposeUiTest {
            val failed =
                PendingChange("row_1", OutboxKind.DOCUMENT_NOTE, "Doc", ChangeStatus.FAILED, "Note too long", 1L)
            val superseded = failed.copy(id = "row_2", status = ChangeStatus.SUPERSEDED)
            var retried: String? = null
            setContent {
                AppTheme {
                    StorageContent(
                        StorageUiState(changes = listOf(failed, superseded)),
                        StorageActions(onRetry = { retried = it }),
                    )
                }
            }

            onNodeWithText("Replaced by a later change").assertExists()
            onAllNodesWithText("Retry").assertCountEquals(1)
            onNodeWithText("Retry").performClick()

            assertEquals("row_1", retried)
        }

    @Test
    fun aKeptRowOffersUnpinAndACachedRowOffersRemove() =
        runComposeUiTest {
            val kept = cached.copy(documentId = "doc_b", title = "Kept book", documentType = "book", pinned = true)
            var unpinned: String? = null
            var removed: String? = null
            setContent {
                AppTheme {
                    StorageContent(
                        StorageUiState(kept = listOf(kept), cached = listOf(cached)),
                        StorageActions(onUnpin = { unpinned = it }, onRemoveCopy = { removed = it }),
                    )
                }
            }

            onNodeWithText("Unpin").performClick()
            onNodeWithText("Remove").performClick()

            assertEquals("doc_b", unpinned)
            assertEquals("doc_a", removed)
        }

    @Test
    fun switchersMarkTheChoiceAndTheFilterNarrowsTheList() =
        runComposeUiTest {
            val kept = cached.copy(documentId = "doc_b", title = "Kept book", pinned = true)
            var cap: Long? = null
            setContent {
                AppTheme {
                    StorageContent(
                        StorageUiState(kept = listOf(kept), cached = listOf(cached), capBytes = 1_073_741_824L),
                        StorageActions(onCapChange = { cap = it }),
                    )
                }
            }

            onNodeWithText("1 GB").assertIsSelected()
            onNodeWithText("2 GB").assertIsNotSelected().performClick()
            onNodeWithText("Kept 1").performClick().assertIsSelected()

            assertEquals(2_147_483_648L, cap)
            onNodeWithText("Kept book").assertExists()
            onNodeWithText("Cached article").assertDoesNotExist()
        }

    @Test
    fun theCapCarriesNoEvictionHintUnderTheMeter() =
        runComposeUiTest {
            setContent { AppTheme { StorageContent(StorageUiState(cached = listOf(cached)), StorageActions()) } }

            onNodeWithText("Copies you haven", substring = true).assertDoesNotExist()
        }

    @Test
    fun theRemovalButtonStandsWithoutAFootnote() =
        runComposeUiTest {
            setContent { AppTheme { StorageContent(StorageUiState(cached = listOf(cached)), StorageActions()) } }

            onNodeWithText("Remove all offline data on this server").assertExists()
            onNodeWithText("Removes downloaded files", substring = true).assertDoesNotExist()
        }

    @Test
    fun aQueuedHighlightReadsAsItsQuote() =
        runComposeUiTest {
            val change =
                PendingChange(
                    id = "row_1",
                    kind = OutboxKind.HIGHLIGHT_CREATE,
                    documentTitle = "The Patient Mind",
                    status = ChangeStatus.PENDING,
                    error = null,
                    createdAt = 1L,
                    detail = "Attention is not a resource.",
                )
            setContent { AppTheme { StorageContent(StorageUiState(changes = listOf(change)), StorageActions()) } }

            onNodeWithText("New highlight \u00b7 \u201cAttention is not a resource.\u201d").assertExists()
            onNodeWithText("Queued").assertExists()
        }
}
