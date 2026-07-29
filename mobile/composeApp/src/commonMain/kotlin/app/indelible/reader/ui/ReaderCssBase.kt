package app.indelible.reader.ui

import app.indelible.reader.model.HighlightStyle
import app.indelible.reader.model.ReaderPreferences
import app.indelible.reader.model.TextAlign
import app.indelible.reader.model.Typeface

internal fun readerFontFamily(typeface: Typeface): String =
    when (typeface) {
        Typeface.SERIF -> "'Newsreader', Georgia, 'Times New Roman', serif"
        Typeface.SANS -> "'Geist', -apple-system, BlinkMacSystemFont, sans-serif"
        Typeface.MONO -> "'Geist Mono', 'SF Mono', 'Fira Code', monospace"
    }

internal fun readerTextAlign(textAlign: TextAlign): String =
    when (textAlign) {
        TextAlign.LEFT -> "left"
        TextAlign.JUSTIFIED -> "justify"
    }

internal fun buildReaderRootCss(
    preferences: ReaderPreferences,
    colors: ReaderBackgroundColors,
    palette: ReaderPalette,
    colorScheme: String,
): String =
    """
:root {
  color-scheme: $colorScheme;
  --serif: 'Newsreader', Georgia, 'Times New Roman', serif;
  --sans: 'Geist', -apple-system, BlinkMacSystemFont, sans-serif;
  --mono: 'Geist Mono', 'SF Mono', 'Fira Code', monospace;
  --font-family: ${readerFontFamily(preferences.typeface)};
  --font-size: ${preferences.fontSize}px;
  --line-height: ${preferences.lineHeight};
  --paragraph-spacing: ${preferences.paragraphSpacing}em;
  --text-align: ${readerTextAlign(preferences.textAlign)};
  --bg-color: ${colors.bg};
  --reader-ink: ${colors.ink};
  --reader-body: ${colors.body};
  --text-color: ${colors.body};
  --accent: ${palette.accent};
  --accent-soft: ${palette.accentSoft};
  --accent-line: ${palette.accentLine};
  --hairline: ${palette.hairline};
  --border: ${palette.border};
  --chip-bg: ${palette.chipBg};
  --text-secondary: ${palette.textSecondary};
  --text-tertiary: ${palette.textTertiary};
}
    """.trimIndent()

// The document does not scroll; .rscroll does. The artwork and the two veils are
// non-scrolling siblings of the scroller, which is what lets the field travel at its
// own rate and the veils sit still. Body-scrolling would force position:fixed on
// exactly those layers, which judders under momentum and rubber-band scrolling.
internal val READER_RESET_CSS =
    """
* { margin: 0; padding: 0; box-sizing: border-box; }
html { background: transparent; height: 100%; }
body {
  font-family: var(--font-family);
  font-size: var(--font-size);
  line-height: var(--line-height);
  color: var(--text-color);
  background: transparent;
  height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  word-wrap: break-word;
  overflow-wrap: break-word;
  -webkit-text-size-adjust: 100%;
  text-align: var(--text-align);
  --y: 0;
  --p: 0;
}
body::before {
  content: ''; position: fixed; inset: 0; z-index: -1;
  background: var(--bg-color);
}
.rscroll {
  flex: 1; min-height: 0; overflow-y: auto; overscroll-behavior: contain;
  position: relative; z-index: 10;
  padding: 0 22px 230px;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: none;
}
.rscroll::-webkit-scrollbar { display: none; }
.article-body * { color: var(--text-color) !important; }
.article-body p {
  font-family: var(--font-family); letter-spacing: -0.003em;
  margin-bottom: var(--paragraph-spacing);
}
    """.trimIndent()

internal val READER_PROSE_CSS =
    """
.article-body a {
  color: var(--accent) !important; text-decoration: underline;
  text-decoration-thickness: 1.5px; text-underline-offset: 3px;
  text-decoration-color: var(--accent-line);
}
.article-body h1, .article-body h2, .article-body h3,
.article-body h4, .article-body h5, .article-body h6 {
  font-family: var(--serif); color: var(--reader-ink) !important;
  margin-top: 1.5em; margin-bottom: 0.5em; line-height: 1.25;
}
.article-body h2 { font-size: 22px; font-weight: 600; }
.article-body img { max-width: 100%; height: auto; border-radius: 8px; }
.article-body pre {
  overflow-x: auto; padding: 12px; border-radius: 8px;
  background: var(--chip-bg); font-size: 0.85em;
}
.article-body code { font-family: var(--mono); font-size: 0.9em; }
.article-body blockquote {
  border-left: 3px solid var(--hairline);
  padding-left: 16px; margin: 1em 0; opacity: 0.85;
}
.article-body ul, .article-body ol { padding-left: 1.5em; margin-bottom: var(--paragraph-spacing); }
.article-body li { margin-bottom: 0.3em; }
.article-body .pullquote {
  margin: 1.4em 0; padding: 6px 0 6px 20px; border-left: 3px solid var(--accent);
  font-family: var(--serif); font-style: italic; font-size: 23px; font-weight: 500;
  line-height: 1.38; color: var(--reader-ink) !important;
}
    """.trimIndent()

// Underline style drops the soft fill and keeps only the inset edge underline.
internal fun buildReaderHighlightCss(
    palette: ReaderPalette,
    highlightStyle: HighlightStyle,
): String {
    val fill = highlightStyle == HighlightStyle.FILL
    fun bg(color: String) = if (fill) color else "transparent"
    return """
mark[data-highlight-id] {
  border-radius: 4px; padding: 1px 1px; cursor: pointer;
  box-decoration-break: clone; -webkit-box-decoration-break: clone;
}
mark.hl-yellow { background: ${bg(palette.hlYBg)} !important; box-shadow: inset 0 -2px 0 ${palette.hlYEdge}; }
mark.hl-blue { background: ${bg(palette.hlBBg)} !important; box-shadow: inset 0 -2px 0 ${palette.hlBEdge}; }
mark.hl-green { background: ${bg(palette.hlGBg)} !important; box-shadow: inset 0 -2px 0 ${palette.hlGEdge}; }
mark.hl-pink { background: ${bg(palette.hlPBg)} !important; box-shadow: inset 0 -2px 0 ${palette.hlPEdge}; }
mark.hl-purple { background: ${bg(palette.hlVBg)} !important; box-shadow: inset 0 -2px 0 ${palette.hlVEdge}; }
.say { border-radius: 4px; transition: background .35s ease, box-shadow .35s ease; }
.say.speaking {
  background: var(--accent-soft); box-shadow: inset 3px 0 0 var(--accent);
  padding: 2px 4px; margin: 0 -4px; box-decoration-break: clone; -webkit-box-decoration-break: clone;
}
.hl-tag-indicator {
  display: inline-flex; align-items: center; gap: 3px;
  font-size: 11px; font-family: var(--mono);
  opacity: 0.75; vertical-align: middle; margin-left: 3px;
  pointer-events: none;
}
.hl-tag-dot {
  width: 6px; height: 6px; border-radius: 50%; display: inline-block; flex-shrink: 0;
}
.hl-tag-label { color: var(--text-tertiary); }
    """.trimIndent()
}
