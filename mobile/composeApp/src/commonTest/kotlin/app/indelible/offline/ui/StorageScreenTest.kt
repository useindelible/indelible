package app.indelible.offline.ui

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertCountEquals
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
            val failed = PendingChange("row_1", OutboxKind.DOCUMENT_NOTE, "Doc", ChangeStatus.FAILED, "Note too long")
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
}
