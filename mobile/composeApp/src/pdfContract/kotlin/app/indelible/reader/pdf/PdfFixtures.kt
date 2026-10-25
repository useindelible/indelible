package app.indelible.reader.pdf

import okio.FileSystem
import okio.Path
import okio.SYSTEM
import kotlin.random.Random

internal data class FixtureText(
    val text: String,
    val x: Double,
    val baseline: Double,
)

internal data class FixturePage(
    val width: Double,
    val height: Double,
    val lines: List<FixtureText>,
    val crop: PageSpaceRect? = null,
    val rotation: Int = 0,
    val redSwatch: PageSpaceRect? = null,
) {
    val box: PageSpaceRect get() = crop ?: PageSpaceRect(0.0, 0.0, width, height)
}

/** Small uncompressed Helvetica PDFs whose geometry is visible in source, written to the temporary directory. */
internal object PdfFixtures {
    const val FONT_SIZE = 12.0
    private const val DESCENT = 0.25
    private const val ASCENT = 0.8
    private const val LETTER_W = 612.0
    private const val LETTER_H = 792.0

    val plain =
        listOf(
            FixturePage(
                LETTER_W,
                LETTER_H,
                listOf(
                    FixtureText("The quick brown fox", 72.0, 720.0),
                    FixtureText("jumps over the lazy dog", 72.0, 700.0),
                ),
            ),
        )
    val rotated = plain.map { it.copy(rotation = 90) }

    val columns =
        listOf(
            FixturePage(
                LETTER_W,
                LETTER_H,
                listOf(
                    FixtureText("alpha beta", 72.0, 720.0),
                    FixtureText("gamma delta", 350.0, 720.0),
                    FixtureText("epsilon zeta", 72.0, 700.0),
                    FixtureText("eta theta", 350.0, 700.0),
                ),
            ),
        )
    val swatch = plain.map { it.copy(redSwatch = PageSpaceRect(72.0, 72.0, 72.0, 72.0)) }
    val cropped =
        listOf(
            FixturePage(
                LETTER_W,
                LETTER_H,
                listOf(FixtureText("Cropped words", 100.0, 600.0)),
                crop = PageSpaceRect(50.0, 100.0, 512.0, 592.0),
            ),
        )
    val multi =
        listOf(LETTER_W to LETTER_H, LETTER_H to LETTER_W, 300.0 to 300.0).mapIndexed { index, (w, h) ->
            FixturePage(w, h, listOf(FixtureText("Page ${index + 1} word", 40.0, h - 60.0)))
        }
    val tall =
        listOf(
            FixturePage(
                300.0,
                6000.0,
                listOf(FixtureText("Top line", 40.0, 5940.0), FixtureText("Bottom line", 40.0, 40.0)),
            ),
        )

    fun write(
        name: String,
        pages: List<FixturePage>,
    ): String = writeBytes(name, pdf(pages))

    fun writeBytes(
        name: String,
        bytes: ByteArray,
    ): String {
        val path: Path = FileSystem.SYSTEM_TEMPORARY_DIRECTORY / "pdf-contract-$name-${Random.nextLong()}.pdf"
        FileSystem.SYSTEM.write(path) { write(bytes) }
        return path.toString()
    }

    /** The advance of [text] in points at [FONT_SIZE], from the Helvetica AFM widths. */
    fun textWidth(text: String): Double = text.sumOf { HELVETICA.getValue(it) } * FONT_SIZE / THOUSAND

    /** The display rect of [line]'s characters [range] (whole line by default), from the AFM metrics. */
    fun expectedRect(
        page: FixturePage,
        line: FixtureText,
        range: IntRange = line.text.indices,
    ): NormalizedRect {
        val left = line.x + textWidth(line.text.substring(0, range.first))
        val width = textWidth(line.text.substring(range.first, range.last + 1))
        val rect = PageSpaceRect(left, line.baseline - DESCENT * FONT_SIZE, width, (ASCENT + DESCENT) * FONT_SIZE)
        return fromPageSpace(rect, page.box, page.rotation)
    }

    /** The display point at the middle of [line]'s characters [range]. */
    fun pointIn(
        page: FixturePage,
        line: FixtureText,
        range: IntRange,
    ): NormalizedPoint {
        val rect = expectedRect(page, line, range)
        return NormalizedPoint(rect.x + rect.width / 2, rect.y + rect.height / 2)
    }

    fun displaySize(page: FixturePage): PdfPageSize {
        val box = page.box
        val swapped = page.rotation % HALF_TURN == QUARTER_TURN
        return if (swapped) PdfPageSize(box.height, box.width) else PdfPageSize(box.width, box.height)
    }

    fun pdf(pages: List<FixturePage>): ByteArray {
        val objects = mutableListOf<String>()
        val kids = pages.indices.joinToString(" ") { "${FIRST_PAGE_OBJECT + 2 * it} 0 R" }
        objects += "<< /Type /Catalog /Pages 2 0 R >>"
        objects += "<< /Type /Pages /Kids [$kids] /Count ${pages.size} >>"
        objects += "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
        pages.forEachIndexed { index, page ->
            val contents = FIRST_PAGE_OBJECT + 2 * index + 1
            val crop =
                page.crop
                    ?.let { " /CropBox [${num(it.x)} ${num(it.y)} ${num(it.x + it.width)} ${num(it.y + it.height)}]" }
                    .orEmpty()
            val rotate = if (page.rotation != 0) " /Rotate ${page.rotation}" else ""
            val box = "/MediaBox [0 0 ${num(page.width)} ${num(page.height)}]$crop$rotate"
            val resources = "/Resources << /Font << /F1 3 0 R >> >>"
            objects += "<< /Type /Page /Parent 2 0 R $box $resources /Contents $contents 0 R >>"
            val fill =
                page.redSwatch?.let { "1 0 0 rg ${num(it.x)} ${num(it.y)} ${num(it.width)} ${num(it.height)} re f\n" }
            val stream =
                fill.orEmpty() +
                    page.lines.joinToString("\n") {
                        "BT /F1 ${num(FONT_SIZE)} Tf ${num(it.x)} ${num(it.baseline)} Td (${it.text}) Tj ET"
                    }
            objects += "<< /Length ${stream.length} >>\nstream\n$stream\nendstream"
        }
        val out = StringBuilder("%PDF-1.4\n")
        val offsets = mutableListOf<Int>()
        objects.forEachIndexed { index, body ->
            offsets += out.length
            out.append("${index + 1} 0 obj\n$body\nendobj\n")
        }
        val xref = out.length
        out.append("xref\n0 ${objects.size + 1}\n0000000000 65535 f \n")
        offsets.forEach { out.append(it.toString().padStart(XREF_DIGITS, '0')).append(" 00000 n \n") }
        out.append("trailer\n<< /Size ${objects.size + 1} /Root 1 0 R >>\nstartxref\n$xref\n%%EOF\n")
        return out.toString().encodeToByteArray()
    }

    private fun num(value: Double): String {
        val whole = value.toLong()
        return if (value == whole.toDouble()) whole.toString() else value.toString()
    }

    private const val FIRST_PAGE_OBJECT = 4
    private const val XREF_DIGITS = 10
    private const val THOUSAND = 1000.0
    private const val HALF_TURN = 180
    private const val QUARTER_TURN = 90

    private val HELVETICA: Map<Char, Int> =
        mapOf(
            ' ' to 278,
            'T' to 611,
            'h' to 556,
            'e' to 556,
            'q' to 556,
            'u' to 556,
            'i' to 222,
            'c' to 500,
            'k' to 500,
            'b' to 556,
            'r' to 333,
            'o' to 556,
            'w' to 722,
            'n' to 556,
            'f' to 278,
            'x' to 500,
            'j' to 222,
            'm' to 833,
            'p' to 556,
            's' to 500,
            'v' to 500,
            't' to 278,
            'l' to 222,
            'a' to 556,
            'z' to 500,
            'y' to 500,
            'd' to 556,
            'g' to 556,
            'C' to 722,
            'P' to 667,
            'B' to 667,
            '1' to 556,
            '2' to 556,
            '3' to 556,
        )
}
