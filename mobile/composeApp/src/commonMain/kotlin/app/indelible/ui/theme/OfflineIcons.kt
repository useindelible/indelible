package app.indelible.ui.theme

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.addPathNodes
import androidx.compose.ui.unit.dp

/** Glyphs of the offline surfaces, on the 24-unit grid of [IndelibleIcons]. */
object OfflineIcons {
    val Airplane: ImageVector by lazy {
        stroked(
            "Airplane",
            "M17.8 19.2L16 11l3.5-3.5C21 6 21.5 4 21 3c-1-.5-3 0-4.5 1.5L13 8 4.8 6.2c-.5-.1-.9.1-1.1.5l-.3.5" +
                "c-.2.5-.1 1 .3 1.3L9 12l-2 3H4l-1 1 3 2 2 3 1-1v-3l3-2 3.5 5.3c.3.4.8.5 1.3.3l.5-.2c.4-.3.6-.7.5-1.2z",
        )
    }
    val Wifi: ImageVector by lazy {
        stroked("Wifi", "M5 12.5a10 10 0 0114 0M8.5 16a5 5 0 017 0M2 9a15 15 0 0120 0M12 19.5h.01")
    }
    val Check: ImageVector by lazy { stroked("Check", "M5 12.5l4.5 4.5L19 7.5") }
    val ArrowUp: ImageVector by lazy { stroked("ArrowUp", "M12 19V5M6 11l6-6 6 6") }
    val Pin: ImageVector by lazy { stroked("Pin", "M12 17v5M8 3h8l-1 6 3 4H6l3-4z") }
    val Download: ImageVector by lazy { stroked("Download", "M12 3v12M7 10l5 5 5-5M5 21h14") }
    val Play: ImageVector by lazy {
        ImageVector
            .Builder("Play", 24.dp, 24.dp, 24f, 24f)
            .addPath(addPathNodes("M7 4.5v15L19 12z"), fill = SolidColor(Color.Black))
            .build()
    }

    private fun stroked(
        name: String,
        pathData: String,
    ): ImageVector =
        ImageVector
            .Builder(name, 24.dp, 24.dp, 24f, 24f)
            .addPath(
                pathData = addPathNodes(pathData),
                stroke = SolidColor(Color.Black),
                strokeLineWidth = 2f,
                strokeLineCap = StrokeCap.Round,
                strokeLineJoin = StrokeJoin.Round,
            ).build()
}
