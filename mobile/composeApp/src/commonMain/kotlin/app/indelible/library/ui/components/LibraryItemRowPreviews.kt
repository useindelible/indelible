package app.indelible.library.ui.components

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.core.model.LibraryItem
import app.indelible.offline.viewmodel.Availability
import app.indelible.offline.viewmodel.DocumentOfflineStatus
import app.indelible.offline.viewmodel.SyncBadge
import app.indelible.ui.theme.AppTheme
import kotlinx.datetime.Instant

@Preview(showBackground = true)
@Composable
private fun LibraryItemRowPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            Column {
                LibraryItemRow(
                    item = previewItem(id = "a1"),
                    onClick = {},
                    offlineStatus = DocumentOfflineStatus(Availability.KEPT, SyncBadge.Pending(2)),
                )
                LibraryItemRow(
                    item =
                        previewItem(
                            id = "b2",
                            itemType = "video",
                            title = "Apple Vision Pro 2: The Spatial Computing Reset",
                            domain = "youtube.com",
                        ),
                    onClick = {},
                )
                LibraryItemRow(
                    item = previewItem(id = "c3"),
                    onClick = {},
                    showDivider = false,
                )
            }
        }
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun LibraryItemRowPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            Column {
                LibraryItemRow(
                    item = previewItem(id = "d4"),
                    onClick = {},
                )
                LibraryItemRow(
                    item =
                        previewItem(
                            id = "e5",
                            itemType = "pdf",
                            title = "Attention Is All You Need — Annotated Edition",
                            domain = "arxiv.org",
                        ),
                    onClick = {},
                    showDivider = false,
                )
            }
        }
    }
}

private fun previewItem(
    id: String,
    itemType: String = "article",
    title: String = "The Future of Open-Source AI Models",
    domain: String? = "techcrunch.com",
) = LibraryItem(
    id = id,
    documentId = "doc_$id",
    itemType = itemType,
    triageState = "inbox",
    isFavorite = false,
    isShortlisted = false,
    title = title,
    excerpt = "A deep dive into what the next generation of open models will look like and who they serve.",
    domain = domain,
    author = "Sarah Chen",
    savedAt = Instant.parse("2024-01-15T12:00:00Z"),
    source = "url",
    createdAt = Instant.parse("2024-01-15T12:00:00Z"),
    updatedAt = Instant.parse("2024-01-15T12:00:00Z"),
)
