package app.indelible.reader.model

import app.indelible.api.generated.models.DocumentAssetResponse
import app.indelible.api.generated.models.HighlightNoteResponse
import app.indelible.api.generated.models.HighlightWithNoteResponse
import app.indelible.api.generated.models.LocatorSchemaFlat
import app.indelible.api.generated.models.TagResponse
import kotlinx.datetime.Instant
import kotlinx.serialization.Serializable

/**
 * What the reader renders for the current item. The reader is HTML-only; PDF
 * items and EPUB books (item type `book`) show a coming-soon placeholder until
 * their dedicated readers ship.
 */
enum class ReaderContentMode { HTML, PDF_COMING_SOON, EPUB_COMING_SOON }

enum class ReaderTheme { LIGHT, DARK, SEPIA, AUTO }

enum class Typeface { SERIF, SANS, MONO }

enum class TextAlign { LEFT, JUSTIFIED }

enum class ReaderBackground { PAPER, SEPIA, SLATE, BLACK }

/** How a highlight paints: a soft fill plus edge, or just the underline edge. */
enum class HighlightStyle { FILL, UNDERLINE }

/**
 * Which reader dock panel is currently open. [NONE] is the resting state.
 * AA = display settings, HL = highlight style, NOTE = note/tags switcher,
 * LISTEN = TTS, INFO = item record, MILA = assistant, MOVE = move-to-collection.
 * Panels land across reader phases.
 */
enum class DataPanel { NONE, AA, HL, NOTE, LISTEN, INFO, MILA, MOVE, CONTENTS }

data class ReaderPreferences(
    // theme is retained for serialization/compat; background now drives the canvas colors.
    val theme: ReaderTheme = ReaderTheme.AUTO,
    val typeface: Typeface = Typeface.SERIF,
    val fontSize: Int = 19,
    val lineHeight: Float = 1.72f,
    val paragraphSpacing: Float = 1.05f,
    val textAlign: TextAlign = TextAlign.LEFT,
    val background: ReaderBackground = ReaderBackground.PAPER,
    val highlightStyle: HighlightStyle = HighlightStyle.FILL,
)

@Serializable
data class HighlightLocator(
    val type: String,
    val startOffset: Long? = null,
    val endOffset: Long? = null,
)

@Serializable
data class HighlightNoteData(
    val id: String,
    val highlightId: String,
    val body: String,
    val createdAt: Instant,
    val updatedAt: Instant,
)

@Serializable
data class HighlightData(
    val id: String,
    val color: String,
    val textContent: String,
    val locator: HighlightLocator? = null,
    val tags: List<String>,
    val createdAt: Instant,
    val updatedAt: Instant,
    val documentId: String? = null,
    val itemTitle: String? = null,
    val note: HighlightNoteData? = null,
)

@Serializable
data class CreateHighlightRequest(
    val color: String,
    val textContent: String,
    val locator: HighlightLocator,
)

data class AssetWithUrlResponse(
    val id: String,
    val documentId: String,
    val assetKind: String,
    val contentType: String,
    val sizeBytes: Long,
    val status: String,
    val downloadUrl: String,
    val createdAt: Instant,
)

@Serializable
data class TagData(
    val id: String,
    val name: String,
    val aliases: List<String> = emptyList(),
    val color: String? = null,
    val parentId: String? = null,
    val itemCount: Long = 0,
    val highlightCount: Long = 0,
    val createdAt: Instant? = null,
)

@Serializable
data class SetHighlightTagsRequest(
    val tags: List<String>,
)

@Serializable
data class TagListResponse(
    val `data`: List<TagData>,
)

fun LocatorSchemaFlat.toHighlightLocator(): HighlightLocator =
    HighlightLocator(
        type = type,
        startOffset = startOffset,
        endOffset = endOffset,
    )

fun HighlightLocator.toLocatorSchemaFlat(): LocatorSchemaFlat =
    LocatorSchemaFlat(
        type = type,
        startOffset = startOffset,
        endOffset = endOffset,
    )

fun DocumentAssetResponse.toAssetWithUrlResponse(): AssetWithUrlResponse =
    AssetWithUrlResponse(
        id = id,
        documentId = documentId,
        assetKind = assetKind,
        contentType = contentType,
        sizeBytes = sizeBytes,
        status = status,
        downloadUrl = downloadUrl,
        createdAt = createdAt,
    )

fun HighlightNoteResponse.toHighlightNoteData(): HighlightNoteData =
    HighlightNoteData(
        id = id,
        highlightId = highlightId,
        body = body,
        createdAt = createdAt,
        updatedAt = updatedAt,
    )

fun HighlightWithNoteResponse.toHighlightData(): HighlightData =
    HighlightData(
        id = id,
        color = color,
        textContent = textContent,
        locator = locator?.toHighlightLocator(),
        tags = tags,
        createdAt = createdAt,
        updatedAt = updatedAt,
        documentId = documentId,
        itemTitle = itemTitle,
        note = note?.toHighlightNoteData(),
    )

fun TagResponse.toTagData(): TagData =
    TagData(
        id = id,
        name = name,
        aliases = aliases,
        color = color,
        parentId = parentId,
        itemCount = itemCount,
        highlightCount = highlightCount,
        createdAt = createdAt,
    )

enum class HighlightColor(
    val apiValue: String,
) {
    YELLOW("yellow"),
    BLUE("blue"),
    GREEN("green"),
    PINK("pink"),
    PURPLE("purple"),
}
