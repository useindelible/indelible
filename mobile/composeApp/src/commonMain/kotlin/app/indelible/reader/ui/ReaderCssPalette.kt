package app.indelible.reader.ui

import app.indelible.reader.model.ReaderBackground

internal data class ReaderBackgroundColors(
    val bg: String,
    val ink: String,
    val body: String,
)

// Page background, heading ink, and body text for each reader background. The
// dark backgrounds (SLATE/BLACK) share the same ink/body so only the page
// colour differs between them.
internal fun backgroundColors(background: ReaderBackground): ReaderBackgroundColors =
    when (background) {
        ReaderBackground.PAPER -> ReaderBackgroundColors(bg = "#FBFAF6", ink = "#1B1712", body = "#2D2A24")
        ReaderBackground.SEPIA -> ReaderBackgroundColors(bg = "#F4ECD8", ink = "#3B2E1A", body = "#5B4636")
        ReaderBackground.SLATE -> ReaderBackgroundColors(bg = "#22272E", ink = "#E9EEF4", body = "#C9D1D9")
        ReaderBackground.BLACK -> ReaderBackgroundColors(bg = "#0D1117", ink = "#E9EEF4", body = "#C9D1D9")
    }

// Accent, chrome, and the five highlight colours (yellow/blue/green/pink/violet)
// for light vs dark reader backgrounds. Selected once by isDarkBg in buildCss so
// the CSS generator interpolates values rather than branching per token.
internal data class ReaderPalette(
    val accent: String,
    val accentSoft: String,
    val accentLine: String,
    val hairline: String,
    val border: String,
    val chipBg: String,
    val textSecondary: String,
    val textTertiary: String,
    val hlYBg: String,
    val hlYEdge: String,
    val hlBBg: String,
    val hlBEdge: String,
    val hlGBg: String,
    val hlGEdge: String,
    val hlPBg: String,
    val hlPEdge: String,
    val hlVBg: String,
    val hlVEdge: String,
)

internal val LIGHT_READER_PALETTE =
    ReaderPalette(
        accent = "#0969DA",
        accentSoft = "rgba(9,105,218,0.10)",
        accentLine = "rgba(9,105,218,0.35)",
        hairline = "rgba(27,31,36,0.09)",
        border = "#D0D7DE",
        chipBg = "rgba(27,31,36,0.05)",
        textSecondary = "#59636E",
        textTertiary = "#818B98",
        hlYBg = "#FFF1B8",
        hlYEdge = "#E3B341",
        hlBBg = "#D9EBFF",
        hlBEdge = "#0969DA",
        hlGBg = "#D6F5DE",
        hlGEdge = "#3FB950",
        hlPBg = "#FBE0EE",
        hlPEdge = "#DB61A2",
        hlVBg = "#EBE0FF",
        hlVEdge = "#A371F7",
    )

internal val DARK_READER_PALETTE =
    ReaderPalette(
        accent = "#2F81F7",
        accentSoft = "rgba(56,139,253,0.15)",
        accentLine = "rgba(56,139,253,0.45)",
        hairline = "rgba(240,246,252,0.10)",
        border = "#30363D",
        chipBg = "rgba(240,246,252,0.06)",
        textSecondary = "#8B949E",
        textTertiary = "#6E7681",
        hlYBg = "rgba(227,179,65,0.30)",
        hlYEdge = "rgba(227,179,65,0.6)",
        hlBBg = "rgba(56,139,253,0.30)",
        hlBEdge = "rgba(56,139,253,0.65)",
        hlGBg = "rgba(63,185,80,0.30)",
        hlGEdge = "rgba(63,185,80,0.6)",
        hlPBg = "rgba(219,97,162,0.32)",
        hlPEdge = "rgba(219,97,162,0.6)",
        hlVBg = "rgba(163,113,247,0.32)",
        hlVEdge = "rgba(163,113,247,0.6)",
    )
