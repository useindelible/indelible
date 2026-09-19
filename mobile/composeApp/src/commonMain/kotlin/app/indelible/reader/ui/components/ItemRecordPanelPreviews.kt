package app.indelible.reader.ui.components

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.offline.viewmodel.Availability
import app.indelible.offline.viewmodel.DocumentOfflineStatus
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.HighlightNoteData
import app.indelible.reader.model.ReaderDocument
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import kotlinx.datetime.Instant

private const val SAMPLE_EPOCH_MS = 1_715_000_000_000L

private val sampleInstant = Instant.fromEpochMilliseconds(SAMPLE_EPOCH_MS)

private fun sampleRecordItem(): ReaderDocument =
    ReaderDocument(
        libraryEntryId = "lib_1",
        id = "doc_1",
        itemType = "article",
        title = "The End of the Beginning",
        url = "https://stratechery.com/x",
        saved = true,
        readableReady = true,
        availableAssets = listOf("readable_html"),
        lastReadAt = sampleInstant,
    )

private fun sampleHighlight(
    id: String,
    color: String,
    text: String,
    noteBody: String?,
): HighlightData =
    HighlightData(
        color = color,
        createdAt = sampleInstant,
        id = id,
        documentId = "doc_1",
        tags = emptyList(),
        textContent = text,
        updatedAt = sampleInstant,
        note =
            noteBody?.let {
                HighlightNoteData(
                    body = it,
                    createdAt = sampleInstant,
                    highlightId = id,
                    id = "note_$id",
                    updatedAt = sampleInstant,
                )
            },
    )

private val sampleHighlights =
    listOf(
        sampleHighlight(
            "hl_1",
            "yellow",
            "The winners compound their advantages quietly, turning scale into a moat " +
                "that looks less like a wall and more like gravity.",
            null,
        ),
        sampleHighlight(
            "hl_2",
            "blue",
            "How incumbents allocate the surplus that maturity provides.",
            "The core question of the whole piece.",
        ),
    )

@Preview
@Composable
private fun ItemRecordPanelPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            ItemRecordPanel(
                item = sampleRecordItem(),
                note = "\"Gravity, not walls\" is the keeper here — distribution moats compound quietly.",
                tags = listOf("strategy", "platforms", "essays"),
                availableTags = emptyList(),
                highlights = sampleHighlights,
                progress = 34f,
                onEditNote = {},
                onTagsChanged = {},
                modifier = Modifier.padding(IndelibleSpacing.screenPaddingH),
                entities = previewEntities,
                offline = ItemOffline(DocumentOfflineStatus(Availability.KEPT, bytes = 1_300_000L)),
            )
        }
    }
}

@Preview
@Composable
private fun ItemRecordPanelPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            ItemRecordPanel(
                item = sampleRecordItem(),
                note = null,
                tags = emptyList(),
                availableTags = emptyList(),
                highlights = sampleHighlights,
                progress = 12f,
                onEditNote = {},
                onTagsChanged = {},
                modifier = Modifier.padding(IndelibleSpacing.screenPaddingH),
                entities = previewEntities,
            )
        }
    }
}
