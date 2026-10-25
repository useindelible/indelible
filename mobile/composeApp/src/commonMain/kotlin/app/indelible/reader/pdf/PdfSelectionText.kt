package app.indelible.reader.pdf

/** The characters `[startChar, endChar)` of [text] grown to whole words, as a long-press drag selects on a phone. */
fun snapToWords(
    text: String,
    startChar: Int,
    endChar: Int,
): IntRange {
    if (text.isEmpty()) return IntRange.EMPTY
    var start = startChar.coerceIn(0, text.length - 1)
    var end = endChar.coerceIn(start + 1, text.length)
    while (start > 0 && !text[start - 1].isWhitespace() && !text[start].isWhitespace()) start--
    while (end < text.length && !text[end].isWhitespace() && !text[end - 1].isWhitespace()) end++
    return start until end
}
