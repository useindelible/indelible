package app.indelible.reader.pdf

/** `[startChar, endChar)` of [text] grown to whole words with edge whitespace dropped, as a long-press drag selects. */
fun snapToWords(
    text: String,
    startChar: Int,
    endChar: Int,
): IntRange {
    if (text.isEmpty()) return IntRange.EMPTY
    var start = startChar.coerceIn(0, text.length - 1)
    var end = endChar.coerceIn(start + 1, text.length)
    while (start < end && text[start].isWhitespace()) start++
    while (end > start && text[end - 1].isWhitespace()) end--
    while (start in 1 until end && joins(text[start - 1], text[start])) start--
    while (start < end && end < text.length && joins(text[end - 1], text[end])) end++
    return start until end
}

// An ideograph is a word of its own, so a drag over Chinese or Japanese text keeps the characters it covered.
private fun joins(
    left: Char,
    right: Char,
) = !left.isWhitespace() && !right.isWhitespace() && !left.isIdeograph() && !right.isIdeograph()

private fun Char.isIdeograph() = code in CJK_UNIFIED || code in CJK_EXTENSION_A || code in KANA || code in FULLWIDTH

private val CJK_EXTENSION_A = 0x3400..0x4DBF
private val CJK_UNIFIED = 0x4E00..0x9FFF
private val KANA = 0x3040..0x30FF
private val FULLWIDTH = 0xFF00..0xFFEF
