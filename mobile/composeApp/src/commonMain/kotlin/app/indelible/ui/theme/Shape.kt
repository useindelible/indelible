package app.indelible.ui.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Shapes
import androidx.compose.ui.unit.dp

/**
 * Indelible border radius scale.
 *
 * radius-xs   → 4dp   Small badges, source color dots
 * radius-sm   → 7dp   Segmented control, sidebar items, buttons, text fields
 * radius-md   → 10dp  Cards, metadata rows, banners
 * radius-lg   → 12dp  Thumbnails, article row avatars
 * radius-xl   → 14dp  Author cards, grouped sections
 * radius-xxl  → 20dp  Hero cards, reader panels, FAB (reimagined large surfaces)
 * radius-full → 980dp Pill buttons, toggle backgrounds
 * drawerEnd   → 20dp leading corners only — right-anchored slide-over panels
 *               (the trailing edge is flush with the screen, so it stays square)
 */
object IndelibleShape {
    val xs = RoundedCornerShape(4.dp)
    val sm = RoundedCornerShape(7.dp)
    val md = RoundedCornerShape(10.dp)
    val lg = RoundedCornerShape(12.dp)
    val xl = RoundedCornerShape(14.dp)
    val xxl = RoundedCornerShape(20.dp)
    val full = RoundedCornerShape(980.dp)
    val drawerEnd = RoundedCornerShape(topStart = 20.dp, bottomStart = 20.dp)

    /** Outgoing chat bubble — the corner nearest the sender stays sharp. */
    val chatBubbleEnd =
        RoundedCornerShape(
            topStart = 14.dp,
            topEnd = 14.dp,
            bottomEnd = 4.dp,
            bottomStart = 14.dp,
        )
}

/**
 * Material Shapes populated from the Indelible radius scale.
 * Material components (Button, Card, TextField, etc.) automatically
 * use these shapes without any per-call configuration.
 *
 *   small      → radius-sm (7dp)  — chips, text fields, buttons
 *   medium     → radius-md (10dp) — cards, dialogs
 *   large      → radius-lg (12dp) — bottom sheets, navigation drawer
 *   extraLarge → radius-xl (14dp) — large modal surfaces
 */
val IndelibleShapes =
    Shapes(
        small = IndelibleShape.sm,
        medium = IndelibleShape.md,
        large = IndelibleShape.lg,
        extraLarge = IndelibleShape.xl,
    )
