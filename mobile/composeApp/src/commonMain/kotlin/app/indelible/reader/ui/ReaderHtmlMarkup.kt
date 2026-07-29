package app.indelible.reader.ui

import kotlinx.datetime.Instant
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime

internal fun escapeHtml(s: String): String = s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")

internal fun escapeJs(s: String): String =
    s
        .replace("\\", "\\\\")
        .replace("\"", "\\\"")
        .replace("\n", "\\n")
        .replace("\r", "\\r")

internal fun formatPublishedDate(instant: Instant): String {
    val dt = instant.toLocalDateTime(TimeZone.UTC)
    val monthName = dt.month.name.lowercase().replaceFirstChar { it.uppercase() }
    return "$monthName ${dt.dayOfMonth}, ${dt.year}"
}

internal data class ReaderMastheadData(
    val title: String,
    val author: String?,
    val domain: String?,
    val publishedAt: Instant?,
    val readingTimeMinutes: Int?,
    val hasSummary: Boolean,
) {
    // Video documents render their masthead natively above the web view, so they pass no
    // metadata here — but the summary handle still belongs to the scrolling content.
    val isEmpty: Boolean
        get() =
            title.isBlank() && author == null && domain == null &&
                publishedAt == null && readingTimeMinutes == null
}

// Emits raw HTML/SVG markup; splitting these literals mid-tag harms readability.
@Suppress("MaxLineLength")
internal fun buildMasthead(data: ReaderMastheadData): String {
    if (data.isEmpty) {
        return if (data.hasSummary) summaryOnlyDivider() else ""
    }
    val sb = StringBuilder()
    sb.append("""<header class="masthead">""")
    data.domain?.let { domain ->
        val chipChar = domain.trim().firstOrNull()?.uppercase() ?: ""
        sb.append("""<div class="mh-source"><span class="mh-mark">""")
        sb.append(escapeHtml(chipChar))
        sb.append("""</span><span class="mh-name">""")
        sb.append(escapeHtml(domain.uppercase()))
        sb.append("</span></div>")
    }
    if (data.title.isNotBlank()) {
        sb.append("""<h1 class="mh-title">""").append(escapeHtml(data.title)).append("</h1>")
    }
    val readingTime = data.readingTimeMinutes?.let { "$it min" }
    val bylineParts = listOfNotNull(data.author, readingTime, data.publishedAt?.let { formatPublishedDate(it) })
    if (bylineParts.isNotEmpty()) {
        val joined =
            bylineParts.joinToString("""<span class="sep">/</span>""") {
                "<span>${escapeHtml(it)}</span>"
            }
        sb.append("""<div class="mh-meta">""").append(joined).append("</div>")
    }
    sb.append("""<div class="mh-rule"></div>""")
    if (data.hasSummary) sb.append(summaryToggle())
    sb.append("</header>")
    return sb.toString()
}

// A labelled disclosure rather than a bare chevron: it says what it opens.
@Suppress("MaxLineLength")
private fun summaryToggle(): String =
    """<button class="sum-toggle" id="reader-summary-handle" aria-expanded="false">""" +
        """<svg class="spark" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3l1.9 5.1L19 10l-5.1 1.9L12 17l-1.9-5.1L5 10l5.1-1.9z"/></svg>""" +
        """<span class="lab">Mila summary</span>""" +
        """<svg class="chev" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9.5l6 6 6-6"/></svg>""" +
        """</button>"""

// Video documents render their masthead natively, but the summary still belongs to
// the scrolling content, so it keeps its rule and disclosure.
private fun summaryOnlyDivider(): String =
    """<div class="mh-rule"></div>""" + summaryToggle()

// Emits raw HTML/SVG markup; splitting these literals mid-tag harms readability.
@Suppress("MaxLineLength")
internal fun buildSummaryBlock(
    summaryHtml: String?,
    summaryPoints: List<String>,
): String {
    if (summaryHtml == null) return ""
    val sb = StringBuilder()
    sb.append("""<div class="sum" id="reader-summary"><div class="sum-in"><div class="sum-card">""")
    sb.append("""<p class="sum-text">""").append(escapeHtml(summaryHtml)).append("</p>")
    if (summaryPoints.isNotEmpty()) {
        sb.append("""<ul class="sum-points">""")
        summaryPoints.forEach { sb.append("<li>").append(escapeHtml(it)).append("</li>") }
        sb.append("</ul>")
    }
    // Hide is gone: the disclosure that opened this is right above it and says so.
    sb.append("""<div class="sum-foot">""")
    sb.append("""<button class="sum-ask" type="button">Ask a follow up</button>""")
    sb.append("</div></div></div></div>")
    return sb.toString()
}
