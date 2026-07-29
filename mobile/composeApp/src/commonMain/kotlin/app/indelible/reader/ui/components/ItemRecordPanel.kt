package app.indelible.reader.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Notes
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import app.indelible.reader.model.DocumentEntity
import app.indelible.reader.model.HighlightColor
import app.indelible.reader.model.HighlightData
import app.indelible.reader.model.HighlightNoteData
import app.indelible.reader.model.ReaderDocument
import app.indelible.reader.model.TagData
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import app.indelible.ui.theme.SerifFontFamily
import app.indelible.ui.theme.geistMonoFontFamily
import kotlinx.datetime.Instant

/**
 * Read-only record for the current item, opened from the reader's "Item details".
 * Mirrors the prototype `pii-*` sheet: a serif hero, a hairline-divided info grid,
 * a serif summary, a bordered note card, mono `#tag` chips, and color-barred
 * highlight quotes. Holds no scroll of its own — the host scaffold provides a
 * bounded, scrollable surface. Tapping the note card hands editing back to the
 * note/tags panel via [onEditNote].
 */
@Composable
fun ItemRecordPanel(
    item: ReaderDocument,
    note: String?,
    tags: List<String>,
    availableTags: List<TagData>,
    highlights: List<HighlightData>,
    progress: Float,
    onEditNote: () -> Unit,
    onTagsChanged: (List<String>) -> Unit,
    modifier: Modifier = Modifier,
    entities: List<DocumentEntity> = emptyList(),
    onSaveToLibrary: () -> Unit = {},
    onShare: () -> Unit = {},
) {
    Column(
        modifier = modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step24),
    ) {
        RecordHero(item)

        // Share moved here when the reader's top bar was removed: it is an action on
        // the item, and the item record is where the item's actions live.
        if ((item.canonicalUrl ?: item.url) != null) {
            RecordSection("Actions") {
                IndelibleButton(
                    text = "Share link",
                    onClick = onShare,
                    style = IndelibleButtonStyle.Secondary,
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        }

        RecordSection("Info") {
            InfoGrid(item = item, progress = progress)
        }

        item.summary?.takeIf { it.isNotBlank() }?.let { summary ->
            RecordSection("Summary") {
                Text(
                    text = summary,
                    style = MaterialTheme.typography.bodyLarge.copy(fontFamily = SerifFontFamily),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }

        if (entities.isNotEmpty()) {
            RecordSection("Entities") {
                EntityGroups(entities)
            }
        }

        RecordSection("Note") {
            NoteCard(note = note, onEditNote = onEditNote)
        }

        RecordSection("Tags") {
            RecordTagsSection(
                saved = item.saved,
                tags = tags,
                availableTags = availableTags,
                onTagsChanged = onTagsChanged,
                onSaveToLibrary = onSaveToLibrary,
            )
        }

        if (highlights.isNotEmpty()) {
            RecordSection("Highlights", count = highlights.size) {
                Column(verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step14)) {
                    highlights.forEach { HighlightRow(it) }
                }
            }
        }
    }
}

// ============================================================
// Section scaffolding
// ============================================================

@Composable
private fun RecordSection(
    title: String,
    count: Int? = null,
    content: @Composable () -> Unit,
) {
    Column(
        modifier = Modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
    ) {
        SectionLabel(title = title, count = count)
        content()
    }
}

/** Tags require a library entry, so an unsaved (feed) document offers Save instead of the editor. */
@Composable
private fun RecordTagsSection(
    saved: Boolean,
    tags: List<String>,
    availableTags: List<TagData>,
    onTagsChanged: (List<String>) -> Unit,
    onSaveToLibrary: () -> Unit,
) {
    if (saved) {
        TagEditor(
            tags = tags,
            availableTags = availableTags,
            onTagsChanged = onTagsChanged,
            startCollapsed = true,
        )
    } else {
        ReaderSaveToLibraryPrompt(
            onSave = onSaveToLibrary,
            message = "Save this item to your library to add tags.",
        )
    }
}

@Composable
private fun SectionLabel(
    title: String,
    count: Int?,
) {
    val accent = MaterialTheme.colorScheme.primary
    val label =
        buildAnnotatedString {
            append(title.uppercase())
            if (count != null) {
                withStyle(SpanStyle(color = accent)) {
                    append(" · $count")
                }
            }
        }
    Text(
        text = label,
        style = monoLabelStyle(),
        color = IndelibleTheme.colors.textTertiary,
    )
}

@Composable
internal fun monoLabelStyle() =
    MaterialTheme.typography.labelSmall.copy(
        fontFamily = geistMonoFontFamily(),
        fontWeight = FontWeight.SemiBold,
        letterSpacing = 0.12.em,
    )

// ============================================================
// Hero
// ============================================================

@Composable
private fun RecordHero(item: ReaderDocument) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
        verticalAlignment = Alignment.Top,
    ) {
        SourceCover(item.domain)
        Column(verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step6)) {
            item.domain?.takeIf { it.isNotBlank() }?.let { domain ->
                Text(
                    text = sourceLabel(domain),
                    style = monoLabelStyle(),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Text(
                text = item.title,
                style =
                    MaterialTheme.typography.headlineSmall.copy(
                        fontFamily = SerifFontFamily,
                        fontWeight = FontWeight.Bold,
                    ),
                color = MaterialTheme.colorScheme.onSurface,
                maxLines = 3,
                overflow = TextOverflow.Ellipsis,
            )
            item.author?.takeIf { it.isNotBlank() }?.let { author ->
                Text(
                    text = author,
                    style =
                        MaterialTheme.typography.labelSmall.copy(
                            fontFamily = geistMonoFontFamily(),
                            letterSpacing = 0.03.em,
                        ),
                    color = IndelibleTheme.colors.textTertiary,
                )
            }
        }
    }
}

@Composable
private fun SourceCover(domain: String?) {
    val glyph =
        domain
            ?.trim()
            ?.firstOrNull { it.isLetterOrDigit() }
            ?.uppercaseChar()
            ?.toString() ?: "?"
    Box(
        modifier =
            Modifier
                .size(IndelibleSpacing.step48)
                .clip(IndelibleShape.lg)
                .background(MaterialTheme.colorScheme.surfaceVariant),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = glyph,
            style =
                MaterialTheme.typography.headlineSmall.copy(
                    fontFamily = SerifFontFamily,
                    fontWeight = FontWeight.Bold,
                ),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

// ============================================================
// Note card
// ============================================================

@Composable
private fun NoteCard(
    note: String?,
    onEditNote: () -> Unit,
) {
    val hasNote = !note.isNullOrBlank()
    Column(
        modifier =
            Modifier
                .fillMaxWidth()
                .clip(IndelibleShape.xl)
                .border(1.dp, MaterialTheme.colorScheme.outline, IndelibleShape.xl)
                .background(MaterialTheme.colorScheme.surfaceVariant)
                .clickable(onClick = onEditNote)
                .padding(
                    horizontal = IndelibleSpacing.cardPaddingH,
                    vertical = IndelibleSpacing.cardPaddingV,
                ),
        verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step10),
    ) {
        Text(
            text = if (hasNote) note else "No note yet.",
            style = MaterialTheme.typography.bodyLarge.copy(fontFamily = SerifFontFamily),
            color = if (hasNote) MaterialTheme.colorScheme.onSurface else IndelibleTheme.colors.textTertiary,
        )
        Row(
            horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step6),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                imageVector = Icons.Filled.Edit,
                contentDescription = null,
                tint = IndelibleTheme.colors.textTertiary,
                modifier = Modifier.size(IndelibleSpacing.step12),
            )
            Text(
                text = if (hasNote) "Tap to edit" else "Add a note",
                style =
                    MaterialTheme.typography.labelSmall.copy(
                        fontFamily = geistMonoFontFamily(),
                        letterSpacing = 0.04.em,
                    ),
                color = IndelibleTheme.colors.textTertiary,
            )
        }
    }
}

// ============================================================
// Highlights
// ============================================================

@Composable
private fun HighlightRow(highlight: HighlightData) {
    val barColor =
        HighlightColor.entries
            .firstOrNull { it.apiValue == highlight.color }
            ?.let { highlightColorToCompose(it) }
            ?: MaterialTheme.colorScheme.outlineVariant
    Row(
        modifier = Modifier.fillMaxWidth().height(IntrinsicSize.Min),
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
    ) {
        Box(
            modifier =
                Modifier
                    .width(IndelibleSpacing.step4)
                    .fillMaxHeight()
                    .clip(IndelibleShape.xs)
                    .background(barColor),
        )
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
        ) {
            Text(
                text = highlight.textContent,
                style = MaterialTheme.typography.bodyLarge.copy(fontFamily = SerifFontFamily),
                color = MaterialTheme.colorScheme.onSurface,
            )
            highlight.note?.body?.takeIf { it.isNotBlank() }?.let { body ->
                Row(
                    horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step6),
                    verticalAlignment = Alignment.Top,
                ) {
                    Icon(
                        imageVector = Icons.AutoMirrored.Filled.Notes,
                        contentDescription = null,
                        tint = IndelibleTheme.colors.textTertiary,
                        modifier =
                            Modifier
                                .padding(top = IndelibleSpacing.step2)
                                .size(IndelibleSpacing.step12),
                    )
                    Text(
                        text = body,
                        style = MaterialTheme.typography.bodySmall,
                        color = IndelibleTheme.colors.textTertiary,
                    )
                }
            }
        }
    }
}

// ============================================================
// Helpers
// ============================================================

/** "stratechery.com" -> "STRATECHERY" for the hero source line. */
private fun sourceLabel(domain: String): String {
    val host = domain.trim().removePrefix("www.")
    return host.substringBefore('.').ifBlank { host }.uppercase()
}

// ============================================================
// Previews
// ============================================================

@Suppress("MagicNumber") // preview-only sample timestamp
private val sampleInstant = Instant.fromEpochMilliseconds(1_715_000_000_000L)

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
