package app.indelible.ui.theme

import androidx.compose.ui.unit.dp

/**
 * Indelible spacing system — all values derive from a 4dp base grid.
 *
 * Permitted values: 0, 2, 4, 6, 8, 10, 12, 14, 16, 20, 24, 28, 32, 40, 48, 56, 64, 80, 96
 * Do NOT use raw dp values outside this list.
 */
object IndelibleSpacing {
    // Base grid steps
    val step0 = 0.dp
    val step2 = 2.dp
    val step4 = 4.dp
    val step6 = 6.dp
    val step8 = 8.dp
    val step10 = 10.dp
    val step12 = 12.dp
    val step14 = 14.dp
    val step16 = 16.dp
    val step20 = 20.dp
    val step24 = 24.dp
    val step28 = 28.dp
    val step32 = 32.dp
    val step40 = 40.dp
    val step48 = 48.dp
    val step56 = 56.dp
    val step64 = 64.dp
    val step80 = 80.dp
    val step96 = 96.dp

    // --------------------------------------------------------
    // Named semantic aliases — use these in screens/components
    // --------------------------------------------------------

    /**
     * A single-pixel rule. Off the 4dp grid on purpose: a hairline is a stroke, not
     * a space, and rounding it up to the nearest step doubles its visual weight.
     */
    val hairline = 1.dp

    /** Horizontal padding for full-screen cards (auth, onboarding) */
    val screenPaddingH = step24

    /** Vertical padding for full-screen cards (auth, onboarding) */
    val screenPaddingV = step32

    /** Standard horizontal padding for list rows */
    val rowPaddingH = step20

    /** Standard vertical padding for list rows */
    val rowPaddingV = step14

    /** Gap between icon and label in sidebar/nav items */
    val iconLabelGap = step10

    /** Gap between thumbnail and text block in article rows */
    val rowContentGap = step16

    /** Standard gap between stacked content items in a card */
    val contentGap = step16

    /** Gap between major sections on a screen */
    val sectionGap = step24

    /** Padding inside a card/grouped section (12 × 14) */
    val cardPaddingH = step14
    val cardPaddingV = step12

    /** Modal/popover item padding (8 × 14) */
    val menuItemPaddingH = step14
    val menuItemPaddingV = step8

    /** Minimum touch target height */
    val touchTarget = step48
}
