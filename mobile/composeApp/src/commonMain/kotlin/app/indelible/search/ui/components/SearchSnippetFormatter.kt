package app.indelible.search.ui.components

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString

internal fun parseSnippetHtml(
    html: String,
    highlightColor: Color,
) = buildAnnotatedString {
    var index = 0
    var highlightStart: Int? = null

    while (index < html.length) {
        when (html[index]) {
            '<' -> {
                val tagEnd = html.indexOf('>', startIndex = index + 1)
                if (tagEnd < 0) break

                val tag = html.substring(index + 1, tagEnd)
                val name = tagName(tag)
                if (name == MARK_TAG) {
                    if (tag.isClosingTag()) {
                        highlightStart?.let { start ->
                            addStyle(SpanStyle(background = highlightColor), start, length)
                        }
                        highlightStart = null
                    } else if (highlightStart == null) {
                        highlightStart = length
                    }
                }
                index = tagEnd + 1
            }

            '&' -> {
                val decoded = decodeEntity(html, index)
                if (decoded == null) {
                    append(html[index])
                    index += 1
                } else {
                    append(decoded.value)
                    index = decoded.nextIndex
                }
            }

            else -> {
                append(html[index])
                index += 1
            }
        }
    }

    highlightStart?.let { start ->
        addStyle(SpanStyle(background = highlightColor), start, length)
    }
}

private data class DecodedEntity(
    val value: String,
    val nextIndex: Int,
)

private fun tagName(tag: String): String {
    val trimmed =
        tag
            .trimStart()
            .removePrefix("/")
            .trimStart()
    return trimmed
        .takeWhile { it.isLetterOrDigit() }
        .lowercase()
}

private fun String.isClosingTag(): Boolean = trimStart().startsWith("/")

private fun decodeEntity(
    html: String,
    entityStart: Int,
): DecodedEntity? {
    val entityEnd = html.indexOf(';', startIndex = entityStart + 1)
    if (entityEnd < 0) return null

    val body = html.substring(entityStart + 1, entityEnd)
    val decoded =
        when (body.lowercase()) {
            "amp" -> "&"
            "lt" -> "<"
            "gt" -> ">"
            "quot" -> "\""
            "#39", "#x27" -> "'"
            "nbsp" -> " "
            else -> decodeNumericEntity(body)
        } ?: return null

    return DecodedEntity(decoded, entityEnd + 1)
}

private fun decodeNumericEntity(body: String): String? {
    val codePoint =
        when {
            body.startsWith("#x", ignoreCase = true) -> body.drop(2).toIntOrNull(radix = 16)
            body.startsWith("#") -> body.drop(1).toIntOrNull()
            else -> null
        } ?: return null

    return codePoint
        .takeIf { it in 0..Char.MAX_VALUE.code }
        ?.toChar()
        ?.toString()
}

private const val MARK_TAG = "mark"
