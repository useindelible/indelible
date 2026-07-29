package app.indelible.ui.theme

import androidx.compose.ui.graphics.Color

// ============================================================
// BACKGROUND
// ============================================================
val BgPrimaryLight = Color(0xFFFFFFFF)
val BgPrimaryDark = Color(0xFF0D1117)

val BgSecondaryLight = Color(0xFFF6F8FA)
val BgSecondaryDark = Color(0xFF161B22)

val BgTertiaryLight = Color(0xFFE4E8EC)
val BgTertiaryDark = Color(0xFF1C2128)

val BgElevatedLight = Color(0xFFFFFFFF)
val BgElevatedDark = Color(0xFF1C2128)

// ============================================================
// TEXT
// ============================================================
val TextPrimaryLight = Color(0xFF1F2328)
val TextPrimaryDark = Color(0xFFE6EDF3)

val TextSecondaryLight = Color(0xFF59636E)
val TextSecondaryDark = Color(0xFF8B949E)

val TextTertiaryLight = Color(0xFF818B98)
val TextTertiaryDark = Color(0xFF6E7681)

val TextQuaternaryLight = Color(0xFFAFB8C1)
val TextQuaternaryDark = Color(0xFF484F58)

// ============================================================
// ACCENT & SEMANTIC
// ============================================================
val AccentLight = Color(0xFF0969DA)
val AccentDark = Color(0xFF2F81F7)

val AccentHoverLight = Color(0xFF0860CA)
val AccentHoverDark = Color(0xFF388BFD)

val DestructiveLight = Color(0xFFCF222E)
val DestructiveDark = Color(0xFFF85149)

val SuccessLight = Color(0xFF1A7F37)
val SuccessDark = Color(0xFF3FB950)

val WarningLight = Color(0xFF9A6700)
val WarningDark = Color(0xFFE3B341)

// ============================================================
// BORDERS & FILLS
// Stored as alpha-encoded Color values (AARRGGBB) so the
// semi-transparent spec values can be used directly in Compose.
// ============================================================
// border-primary  = rgba(27,31,36,0.09) light / rgba(240,246,252,0.10) dark
val BorderPrimaryLight = Color(0x171B1F24)
val BorderPrimaryDark = Color(0x1AF0F6FC)

// border-secondary = solid hairlines
val BorderSecondaryLight = Color(0xFFD0D7DE)
val BorderSecondaryDark = Color(0xFF30363D)

// fill-selected    = accent @ 0.10 light / accent-strong @ 0.15 dark
val FillSelectedLight = Color(0x1A0969DA)
val FillSelectedDark = Color(0x26388BFD)

// fill-selected-strong = accent @ 0.16 light / accent-strong @ 0.25 dark
val FillSelectedStrongLight = Color(0x290969DA)
val FillSelectedStrongDark = Color(0x40388BFD)

// fill-hover = rgba(27,31,36,0.06) light / rgba(240,246,252,0.078) dark
val FillHoverLight = Color(0x0F1B1F24)
val FillHoverDark = Color(0x14F0F6FC)

// ============================================================
// HIGHLIGHTS (reader)
// ============================================================
val HighlightYellowLight = Color(0xFFFFF3BF)
val HighlightYellowDark = Color(0x38FFD600)
val HighlightYellowBorder = Color(0xFFFFD600)

val HighlightBlueLight = Color(0xFFD4EAFF)
val HighlightBlueDark = Color(0x2E0A84FF)
val HighlightBlueBorder = Color(0xFF0A84FF)

val HighlightGreenLight = Color(0xFFD4F5DD)
val HighlightGreenDark = Color(0x2E34C759)
val HighlightGreenBorder = Color(0xFF34C759)

val HighlightPinkLight = Color(0xFFFFD6E0)
val HighlightPinkDark = Color(0x2EFF2D55)
val HighlightPinkBorder = Color(0xFFFF2D55)

val HighlightPurpleLight = Color(0xFFE8D5F5)
val HighlightPurpleDark = Color(0x2EAF52DE)
val HighlightPurpleBorder = Color(0xFFAF52DE)

// ============================================================
// ACCENT PALETTE — vivid per-colour accents (tag dots, etc.)
// ============================================================
val PurpleLight = Color(0xFFAF52DE)
val PurpleDark = Color(0xFFBF5AF2)

// ============================================================
// READER SURFACES — native reader canvas (behind WebView aura)
// ============================================================
val ReaderBgPaper = Color(0xFFFBFAF6)
val ReaderBgDark = Color(0xFF0D1117)
val ReaderInkLight = Color(0xFF1B1712)
val ReaderInkDark = Color(0xFFE9EEF4)
val ReaderBodyLight = Color(0xFF2D2A24)
val ReaderBodyDark = Color(0xFFC9D1D9)

// Reader background swatches shown in display settings. Paper/black reuse the
// canvas tokens above; sepia/slate are swatch-only mid tones.
val ReaderSwatchSepia = Color(0xFFF2E5CC)
val ReaderSwatchSlate = Color(0xFF22272E)

// ============================================================
// ACCENT VARIANTS — used by native reader chrome
// ============================================================
// accent-line = accent @ 0.35 light / accent-strong @ 0.45 dark
val AccentLineLight = Color(0x590969DA)
val AccentLineDark = Color(0x73388BFD)

// ============================================================
// AURA palettes — three blob stops each, drawn behind the
// ============================================================




// ============================================================
// ALWAYS-FIXED
// ============================================================
val White = Color(0xFFFFFFFF)
val Black = Color(0xFF000000)

// ============================================================
// ACCENT SWATCHES (user-selectable accent palette)
// ============================================================
val AccentBlue = Color(0xFF0071E3)
val AccentPink = Color(0xFFFF2D55)
val AccentGreen = Color(0xFF34C759)
val AccentOrange = Color(0xFFFF9500)
