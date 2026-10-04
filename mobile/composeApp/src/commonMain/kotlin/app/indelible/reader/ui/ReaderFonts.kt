package app.indelible.reader.ui

import indelible.composeapp.generated.resources.Res
import kotlin.io.encoding.Base64

internal data class ReaderFontFace(
    val family: String,
    val style: String,
    val weights: String,
    val file: String,
)

internal val READER_FONT_FACES =
    listOf(
        ReaderFontFace("Newsreader", "normal", "200 800", "newsreader-normal.woff2"),
        ReaderFontFace("Newsreader", "italic", "200 800", "newsreader-italic.woff2"),
        ReaderFontFace("Geist", "normal", "100 900", "geist-normal.woff2"),
        ReaderFontFace("Geist Mono", "normal", "100 900", "geist-mono-normal.woff2"),
    )

private const val READER_FONT_DIR = "files/reader-fonts"

/** Faces embedded as data URLs, so a page never fetches a font and reads the same offline. */
internal fun fontFaceCss(faces: List<Pair<ReaderFontFace, ByteArray>>): String =
    faces.joinToString("\n") { (face, bytes) ->
        "@font-face { font-family: '${face.family}'; font-style: ${face.style}; font-weight: ${face.weights}; " +
            "font-display: block; src: url(data:font/woff2;base64,${Base64.encode(bytes)}) format('woff2'); }"
    }

internal suspend fun loadReaderFontCss(): String = fontFaceCss(READER_FONT_FACES.map { it to Res.readBytes(it.path) })

private val ReaderFontFace.path: String get() = "$READER_FONT_DIR/$file"
