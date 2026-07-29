package app.indelible.reader.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.reader.model.TagData
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleTextField
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import kotlinx.datetime.Instant

private enum class NoteTagsTab(
    val label: String,
) {
    NOTE("Note"),
    TAGS("Tags"),
}

/**
 * Panel content for the reader Note button: a segmented switch between the
 * article note editor and the tag picker. The host wraps this in
 * [ReaderBottomSheetScaffold]; both tabs commit through the supplied callbacks,
 * which the ViewModel persists optimistically.
 */
@Composable
fun NoteTagsSwitcherSheet(
    note: String,
    tags: List<String>,
    availableTags: List<TagData>,
    onSaveNote: (String) -> Unit,
    onTagsChanged: (List<String>) -> Unit,
    modifier: Modifier = Modifier,
    tagsEnabled: Boolean = true,
    onSaveToLibrary: () -> Unit = {},
) {
    var selectedTab by remember { mutableStateOf(NoteTagsTab.NOTE) }
    val tabs = NoteTagsTab.entries

    Column(modifier = modifier.fillMaxWidth()) {
        SingleChoiceSegmentedButtonRow(modifier = Modifier.fillMaxWidth()) {
            tabs.forEachIndexed { index, tab ->
                SegmentedButton(
                    selected = selectedTab == tab,
                    onClick = { selectedTab = tab },
                    shape = SegmentedButtonDefaults.itemShape(index, tabs.size),
                ) {
                    Text(text = tab.label, style = MaterialTheme.typography.bodySmall)
                }
            }
        }

        Spacer(modifier = Modifier.height(IndelibleSpacing.sectionGap))

        when (selectedTab) {
            NoteTagsTab.NOTE -> NoteEditor(note = note, onSaveNote = onSaveNote)
            NoteTagsTab.TAGS ->
                if (tagsEnabled) {
                    TagEditor(
                        tags = tags,
                        availableTags = availableTags,
                        onTagsChanged = onTagsChanged,
                    )
                } else {
                    ReaderSaveToLibraryPrompt(
                        onSave = onSaveToLibrary,
                        message = "Save this item to your library to add tags.",
                    )
                }
        }
    }
}

@Composable
private fun NoteEditor(
    note: String,
    onSaveNote: (String) -> Unit,
) {
    var draft by remember(note) { mutableStateOf(note) }
    Column(modifier = Modifier.fillMaxWidth()) {
        IndelibleTextField(
            value = draft,
            onValueChange = { draft = it },
            label = "Article note",
            singleLine = false,
            minLines = 4,
            imeAction = ImeAction.Default,
        )
        Spacer(modifier = Modifier.height(IndelibleSpacing.step10))
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Text(
                text = "Saved with this article",
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            IndelibleButton(
                text = "Save note",
                onClick = { onSaveNote(draft) },
                compact = true,
                enabled = draft != note,
            )
        }
    }
}

/**
 * Tag editor shared by the Note/Tags switcher and the item-record panel: removable
 * chips plus a search-or-create field with suggestions, committing optimistically
 * through [onTagsChanged]. With [startCollapsed] the field stays hidden behind an
 * "Add tag" button until tapped, so the record panel can show tags compactly among
 * its other sections; the switcher's dedicated Tags tab leaves it expanded.
 */
private const val MAX_TAG_SUGGESTIONS = 8

@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun TagEditor(
    tags: List<String>,
    availableTags: List<TagData>,
    onTagsChanged: (List<String>) -> Unit,
    startCollapsed: Boolean = false,
) {
    var currentTags by remember(tags) { mutableStateOf(tags) }
    var query by remember { mutableStateOf("") }
    var inputRevealed by remember { mutableStateOf(!startCollapsed) }

    val suggestions =
        remember(availableTags, query, currentTags) {
            val q = query.trim().lowercase()
            availableTags
                .filter { it.name !in currentTags && (q.isEmpty() || it.name.lowercase().contains(q)) }
                .take(MAX_TAG_SUGGESTIONS)
        }
    val canCreateNew =
        remember(query, currentTags, availableTags) {
            val trimmed = query.trim()
            trimmed.isNotEmpty() &&
                trimmed !in currentTags &&
                availableTags.none { it.name.equals(trimmed, ignoreCase = true) }
        }

    fun addTag(name: String) {
        if (name !in currentTags) {
            currentTags = currentTags + name
            onTagsChanged(currentTags)
        }
        query = ""
    }

    fun removeTag(name: String) {
        currentTags = currentTags - name
        onTagsChanged(currentTags)
    }

    Column(modifier = Modifier.fillMaxWidth()) {
        if (currentTags.isNotEmpty()) {
            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
                verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
                modifier = Modifier.fillMaxWidth(),
            ) {
                currentTags.forEach { tag ->
                    AppliedTagChip(name = tag, onRemove = { removeTag(tag) })
                }
            }
            Spacer(modifier = Modifier.height(IndelibleSpacing.step12))
        }

        if (!inputRevealed) {
            AddTagButton(onClick = { inputRevealed = true })
            return@Column
        }

        TagSearchField(
            query = query,
            onQueryChange = { query = it },
            onDone = { trimmed -> addTag(trimmed) },
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

        TagSuggestionList(
            query = query,
            suggestions = suggestions,
            canCreateNew = canCreateNew,
            hasTags = currentTags.isNotEmpty(),
            onAdd = { addTag(it) },
        )
    }
}

@Composable
private fun TagSuggestionList(
    query: String,
    suggestions: List<TagData>,
    canCreateNew: Boolean,
    hasTags: Boolean,
    onAdd: (String) -> Unit,
) {
    Column(modifier = Modifier.fillMaxWidth()) {
        if (canCreateNew) {
            TagSuggestionRow(
                name = query.trim(),
                count = null,
                isCreate = true,
                onClick = { onAdd(query.trim()) },
            )
        }
        if (canCreateNew && suggestions.isNotEmpty()) {
            HorizontalDivider(
                color = MaterialTheme.colorScheme.outlineVariant,
                modifier = Modifier.padding(vertical = IndelibleSpacing.step4),
            )
        }
        suggestions.forEach { tag ->
            TagSuggestionRow(
                name = tag.name,
                count = tag.highlightCount.toInt(),
                isCreate = false,
                onClick = { onAdd(tag.name) },
            )
        }
        if (suggestions.isEmpty() && !canCreateNew && query.isEmpty() && !hasTags) {
            Text(
                text = "No tags yet. Type to create one.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(vertical = IndelibleSpacing.step16),
            )
        }
    }
}

@Composable
private fun AddTagButton(onClick: () -> Unit) {
    Row(
        modifier =
            Modifier
                .clip(IndelibleShape.full)
                .background(MaterialTheme.colorScheme.surfaceVariant)
                .clickable(onClick = onClick)
                .padding(horizontal = IndelibleSpacing.step12, vertical = IndelibleSpacing.step8),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step6),
    ) {
        Icon(
            Icons.Filled.Add,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(IndelibleSpacing.step16),
        )
        Text(
            text = "Add tag",
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun AppliedTagChip(
    name: String,
    onRemove: () -> Unit,
) {
    Row(
        modifier =
            Modifier
                .clip(IndelibleShape.full)
                .background(MaterialTheme.colorScheme.secondaryContainer)
                .padding(
                    start = IndelibleSpacing.step12,
                    top = IndelibleSpacing.step6,
                    end = IndelibleSpacing.step6,
                    bottom = IndelibleSpacing.step6,
                ),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step6),
    ) {
        Text(
            text = name,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSecondaryContainer,
        )
        Box(
            modifier =
                Modifier
                    .size(IndelibleSpacing.step20)
                    .clip(CircleShape)
                    .clickable(onClick = onRemove),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                Icons.Filled.Close,
                contentDescription = "Remove tag",
                tint = MaterialTheme.colorScheme.onSecondaryContainer,
                modifier = Modifier.size(IndelibleSpacing.step14),
            )
        }
    }
}

@Composable
private fun TagSuggestionRow(
    name: String,
    count: Int?,
    isCreate: Boolean,
    onClick: () -> Unit,
) {
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .clickable(onClick = onClick)
                .padding(vertical = IndelibleSpacing.step10),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
    ) {
        Icon(
            IndelibleIcons.Tag,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(IndelibleSpacing.step20),
        )
        Text(
            text = if (isCreate) "Create \"$name\"" else name,
            style = MaterialTheme.typography.bodyMedium,
            color = if (isCreate) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.weight(1f),
        )
        if (!isCreate && count != null) {
            Text(
                text = count.toString(),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

private fun sampleTag(
    id: String,
    name: String,
    count: Long,
): TagData =
    TagData(
        aliases = emptyList(),
        color = null,
        createdAt = Instant.fromEpochMilliseconds(0),
        highlightCount = count,
        id = id,
        itemCount = count,
        name = name,
        parentId = null,
    )

@Suppress("MagicNumber") // preview-only sample tag counts
private val previewTags =
    listOf(
        sampleTag("tag_1", "Design", 8),
        sampleTag("tag_2", "Systems", 3),
        sampleTag("tag_3", "Reading", 12),
    )

@Preview
@Composable
private fun NoteTagsSwitcherSheetPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            NoteTagsSwitcherSheet(
                note = "The author's framing of attention as a finite resource reframes the whole piece.",
                tags = listOf("Design", "Focus"),
                availableTags = previewTags,
                onSaveNote = {},
                onTagsChanged = {},
                modifier = Modifier.padding(IndelibleSpacing.screenPaddingH),
            )
        }
    }
}

@Preview
@Composable
private fun NoteTagsSwitcherSheetPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            NoteTagsSwitcherSheet(
                note = "",
                tags = emptyList(),
                availableTags = previewTags,
                onSaveNote = {},
                onTagsChanged = {},
                modifier = Modifier.padding(IndelibleSpacing.screenPaddingH),
            )
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Preview
@Composable
private fun TagEditorPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            TagEditor(
                tags = listOf("Design", "Focus"),
                availableTags = previewTags,
                onTagsChanged = {},
            )
        }
    }
}
