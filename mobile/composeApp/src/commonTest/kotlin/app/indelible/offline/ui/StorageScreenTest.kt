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
import app.indelible.core.offline.STILL_PREPARING_ERROR
import app.indelible.core.offline.SyncFailure
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
                        StorageActions(sync = SyncActions(onRetry = { retried = it })),
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

    @Test
    fun aDeletedItemShowsItsFullTextWithCopyAndDiscard() =
        runComposeUiTest {
            val quote = "Attention is not a resource. It is a stance the whole body takes towards the world."
            val change =
                failed(
                    OutboxKind.HIGHLIGHT_CREATE,
                    SyncFailure(SyncFailure.Kind.DELETED_ELSEWHERE, status = 404, serverCode = "not_found"),
                    detail = quote,
                )
            var copied: String? = null
            setContent {
                AppTheme {
                    StorageContent(
                        StorageUiState(changes = listOf(change)),
                        StorageActions(sync = SyncActions(onCopyText = { copied = it })),
                    )
                }
            }

            onNodeWithText("This item was deleted on another device").assertExists()
            onNodeWithText(quote).assertExists()
            onNodeWithText("Discard").assertExists()
            onNodeWithText("Retry").assertDoesNotExist()
            onNodeWithText("Copy text").performClick()

            assertEquals(quote, copied)
        }

    @Test
    fun discardAsksBeforeDeletingAndNamesTheDependants() =
        runComposeUiTest {
            val change =
                failed(
                    OutboxKind.HIGHLIGHT_CREATE,
                    SyncFailure(SyncFailure.Kind.ANCHOR_LOST, status = 422, serverCode = "validation_error"),
                    detail = "quoted words",
                    blocked = listOf(blockedColour("row_2"), blockedColour("row_3")),
                )
            var discarded: String? = null
            setContent {
                AppTheme {
                    StorageContent(
                        StorageUiState(changes = listOf(change)),
                        StorageActions(sync = SyncActions(onDiscard = { discarded = it })),
                    )
                }
            }

            onNodeWithText("Couldn\u2019t place this highlight").assertExists()
            onAllNodesWithText("Waiting on the highlight above").assertCountEquals(2)
            onNodeWithText("Discard").performClick()
            onNodeWithText("2 changes", substring = true).assertExists()
            assertEquals(null, discarded)
            onNodeWithText("Keep it").performClick()
            onNodeWithText("Delete permanently").assertDoesNotExist()
            onNodeWithText("Discard").performClick()
            onNodeWithText("Delete permanently").performClick()

            assertEquals("row_1", discarded)
        }

    @Test
    fun aServerTooOldChangeOffersNoRetry() =
        runComposeUiTest {
            val change =
                failed(
                    OutboxKind.READING_EVENT,
                    SyncFailure(
                        SyncFailure.Kind.SERVER_TOO_OLD,
                        status = 405,
                        feature = SyncFailure.Feature.READING_EVENTS,
                    ),
                )
            setContent { AppTheme { StorageContent(StorageUiState(changes = listOf(change)), StorageActions()) } }

            onNodeWithText("Your server needs updating").assertExists()
            onNodeWithText("Needs a newer server \u00b7 Reading events").assertExists()
            onNodeWithText("Retry").assertDoesNotExist()
            onNodeWithText("Discard").assertDoesNotExist()
        }

    @Test
    fun aRejectedChangeShowsTheServerMessage() =
        runComposeUiTest {
            val change =
                failed(
                    OutboxKind.DOCUMENT_NOTE,
                    SyncFailure(SyncFailure.Kind.REJECTED, status = 400, serverMessage = "Note too long"),
                    detail = "a very long note",
                )
            var retried: String? = null
            setContent {
                AppTheme {
                    StorageContent(
                        StorageUiState(changes = listOf(change)),
                        StorageActions(sync = SyncActions(onRetry = { retried = it })),
                    )
                }
            }

            onNodeWithText("Rejected by the server").assertExists()
            onNodeWithText("Note too long").assertExists()
            onNodeWithText("Discard").assertExists()
            onNodeWithText("Retry").performClick()

            assertEquals("row_1", retried)
        }

    @Test
    fun viewBothOpensTheConflictSheet() =
        runComposeUiTest {
            val conflict =
                SyncFailure(SyncFailure.Kind.NOTE_CONFLICT, serverNoteBody = "theirs", serverNoteUpdatedAtEpochMs = 2L)
            val change = failed(OutboxKind.DOCUMENT_NOTE, conflict, detail = "mine")
            var kept: Pair<String, Long>? = null
            setContent {
                AppTheme {
                    StorageContent(
                        StorageUiState(changes = listOf(change)),
                        StorageActions(sync = SyncActions(onKeepMine = { id, seenAt -> kept = id to seenAt })),
                    )
                }
            }

            onNodeWithText("This note changed on another device").assertExists()
            onNodeWithText("Retry").assertDoesNotExist()
            onNodeWithText("Keep mine").assertDoesNotExist()
            onNodeWithText("View both").performClick()
            onNodeWithText("theirs").assertExists()
            onNodeWithText("Keep mine").performClick()

            assertEquals("row_1" to 2L, kept)
        }

    @Test
    fun anUnreachableServerReadsAsNotAnsweringWithRetryNow() =
        runComposeUiTest {
            val queued = PendingChange("row_1", OutboxKind.DOCUMENT_NOTE, "Doc", ChangeStatus.RETRYING, "timeout", 1L)
            var retriedNow = 0
            setContent {
                AppTheme {
                    StorageContent(
                        StorageUiState(changes = listOf(queued), serverUnreachable = true),
                        StorageActions(sync = SyncActions(onRetryNow = { retriedNow++ })),
                    )
                }
            }

            onNodeWithText("Server not answering").assertExists()
            onNodeWithText("Will keep trying").assertExists()
            onNodeWithText("Retry now").performClick()

            assertEquals(1, retriedNow)
        }

    @Test
    fun aChangeTheServerIsStillPreparingForSaysSo() =
        runComposeUiTest {
            val preparing =
                PendingChange(
                    id = "row_1",
                    kind = OutboxKind.HIGHLIGHT_CREATE,
                    documentTitle = "Doc",
                    status = ChangeStatus.RETRYING,
                    error = STILL_PREPARING_ERROR,
                    createdAt = 1L,
                )
            setContent { AppTheme { StorageContent(StorageUiState(changes = listOf(preparing)), StorageActions()) } }

            onNodeWithText("The server is still preparing this item").assertExists()
        }
}
