package app.indelible.ui.theme

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.path
import androidx.compose.ui.unit.dp

object IndelibleIcons {
    val Folder: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "Folder",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(3f, 7f)
                    verticalLineToRelative(10f)
                    arcToRelative(2f, 2f, 0f, false, false, 2f, 2f)
                    horizontalLineToRelative(14f)
                    arcToRelative(2f, 2f, 0f, false, false, 2f, -2f)
                    verticalLineTo(9f)
                    arcToRelative(2f, 2f, 0f, false, false, -2f, -2f)
                    horizontalLineToRelative(-6f)
                    lineToRelative(-2f, -2f)
                    horizontalLineTo(5f)
                    arcToRelative(2f, 2f, 0f, false, false, -2f, 2f)
                    close()
                }
            }.build()
    }

    val Trash: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "Trash",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                // M3 6h18
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(3f, 6f)
                    horizontalLineToRelative(18f)
                }
                // M16 6V4a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v2
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(16f, 6f)
                    verticalLineTo(4f)
                    arcToRelative(2f, 2f, 0f, false, false, -2f, -2f)
                    horizontalLineToRelative(-4f)
                    arcToRelative(2f, 2f, 0f, false, false, -2f, 2f)
                    verticalLineToRelative(2f)
                }
                // M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(19f, 6f)
                    lineToRelative(-1f, 14f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, 2f)
                    horizontalLineTo(8f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, -2f)
                    lineTo(5f, 6f)
                }
                // line x1=10 y1=11 x2=10 y2=17
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(10f, 11f)
                    verticalLineToRelative(6f)
                }
                // line x1=14 y1=11 x2=14 y2=17
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(14f, 11f)
                    verticalLineToRelative(6f)
                }
            }.build()
    }

    val Feed: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "Feed",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                // circle cx=4 cy=20 r=1.5 (filled dot)
                path(
                    fill = SolidColor(Color(0xFF000000)),
                    stroke = null,
                ) {
                    moveTo(5.5f, 20f)
                    arcToRelative(1.5f, 1.5f, 0f, true, true, -3f, 0f)
                    arcToRelative(1.5f, 1.5f, 0f, false, true, 3f, 0f)
                }
                // M4 13a7 7 0 0 1 7 7
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(4f, 13f)
                    arcTo(7f, 7f, 0f, false, true, 11f, 20f)
                }
                // M4 6a14 14 0 0 1 14 14
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(4f, 6f)
                    arcTo(14f, 14f, 0f, false, true, 18f, 20f)
                }
            }.build()
    }

    val Archive: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "Archive",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                // outer box: M21 8 3 8 V20 a2 2 0 0 0 2 2h14 a2 2 0 0 0 2 -2V8z
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(21f, 8f)
                    horizontalLineTo(3f)
                    verticalLineTo(20f)
                    arcToRelative(2f, 2f, 0f, false, false, 2f, 2f)
                    horizontalLineToRelative(14f)
                    arcToRelative(2f, 2f, 0f, false, false, 2f, -2f)
                    verticalLineTo(8f)
                    close()
                }
                // top lid: M1 3h22v5H1z (rect)
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(1f, 3f)
                    horizontalLineToRelative(22f)
                    verticalLineToRelative(5f)
                    horizontalLineTo(1f)
                    close()
                }
                // inner dashes: M10 12h4
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(10f, 12f)
                    horizontalLineToRelative(4f)
                }
            }.build()
    }

    val Highlights: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "Highlights",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                // Pen/highlight icon: M12 20h9
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(12f, 20f)
                    horizontalLineToRelative(9f)
                }
                // M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(16.5f, 3.5f)
                    arcToRelative(2.121f, 2.121f, 0f, false, true, 3f, 3f)
                    lineTo(7f, 19f)
                    lineToRelative(-4f, 1f)
                    lineToRelative(1f, -4f)
                    close()
                }
            }.build()
    }

    val Favorites: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "Favorites",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                // M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78
                // l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(20.84f, 4.61f)
                    arcToRelative(5.5f, 5.5f, 0f, false, false, -7.78f, 0f)
                    lineTo(12f, 5.67f)
                    lineToRelative(-1.06f, -1.06f)
                    arcToRelative(5.5f, 5.5f, 0f, false, false, -7.78f, 7.78f)
                    lineToRelative(1.06f, 1.06f)
                    lineTo(12f, 21.23f)
                    lineToRelative(7.78f, -7.78f)
                    lineToRelative(1.06f, -1.06f)
                    arcToRelative(5.5f, 5.5f, 0f, false, false, 0f, -7.78f)
                    close()
                }
            }.build()
    }

    val Article: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "Article",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                // rect x=3 y=3 width=18 height=18 rx=2
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(5f, 3f)
                    horizontalLineTo(19f)
                    arcToRelative(2f, 2f, 0f, false, true, 2f, 2f)
                    verticalLineTo(19f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, 2f)
                    horizontalLineTo(5f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, -2f)
                    verticalLineTo(5f)
                    arcToRelative(2f, 2f, 0f, false, true, 2f, -2f)
                    close()
                }
                // line x1=7 y1=8 x2=17 y2=8
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(7f, 8f)
                    horizontalLineTo(17f)
                }
                // line x1=7 y1=12 x2=17 y2=12
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(7f, 12f)
                    horizontalLineTo(17f)
                }
                // line x1=7 y1=16 x2=13 y2=16
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(7f, 16f)
                    horizontalLineTo(13f)
                }
            }.build()
    }

    val Book: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "Book",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                // M2 4h6a4 4 0 0 1 4 4v13a3 3 0 0 0-3-3H2V4z
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(2f, 4f)
                    horizontalLineToRelative(6f)
                    arcToRelative(4f, 4f, 0f, false, true, 4f, 4f)
                    verticalLineToRelative(13f)
                    arcToRelative(3f, 3f, 0f, false, false, -3f, -3f)
                    horizontalLineTo(2f)
                    verticalLineTo(4f)
                    close()
                }
                // M22 4h-6a4 4 0 0 0-4 4v13a3 3 0 0 1 3-3h7V4z
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(22f, 4f)
                    horizontalLineToRelative(-6f)
                    arcToRelative(4f, 4f, 0f, false, false, -4f, 4f)
                    verticalLineToRelative(13f)
                    arcToRelative(3f, 3f, 0f, false, true, 3f, -3f)
                    horizontalLineToRelative(7f)
                    verticalLineTo(4f)
                    close()
                }
            }.build()
    }

    val Email: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "Email",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                // M4 4h16a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(4f, 4f)
                    horizontalLineToRelative(16f)
                    arcToRelative(2f, 2f, 0f, false, true, 2f, 2f)
                    verticalLineToRelative(12f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, 2f)
                    horizontalLineTo(4f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, -2f)
                    verticalLineTo(6f)
                    arcToRelative(2f, 2f, 0f, false, true, 2f, -2f)
                    close()
                }
                // polyline points="22,6 12,13 2,6"
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(22f, 6f)
                    lineTo(12f, 13f)
                    lineTo(2f, 6f)
                }
            }.build()
    }

    val Pdf: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "Pdf",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                // M5 3h14a1 1 0 0 1 1 1v16a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1z
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(5f, 3f)
                    horizontalLineToRelative(14f)
                    arcToRelative(1f, 1f, 0f, false, true, 1f, 1f)
                    verticalLineToRelative(16f)
                    arcToRelative(1f, 1f, 0f, false, true, -1f, 1f)
                    horizontalLineTo(5f)
                    arcToRelative(1f, 1f, 0f, false, true, -1f, -1f)
                    verticalLineTo(4f)
                    arcToRelative(1f, 1f, 0f, false, true, 1f, -1f)
                    close()
                }
                // line x1=4 y1=9 x2=20 y2=9
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(4f, 9f)
                    horizontalLineTo(20f)
                }
                // line x1=7 y1=13 x2=17 y2=13
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(7f, 13f)
                    horizontalLineTo(17f)
                }
                // line x1=7 y1=17 x2=13 y2=17
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(7f, 17f)
                    horizontalLineTo(13f)
                }
            }.build()
    }

    val Video: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "Video",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                // rect x=2 y=5 width=20 height=13 rx=2
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(4f, 5f)
                    horizontalLineTo(20f)
                    arcToRelative(2f, 2f, 0f, false, true, 2f, 2f)
                    verticalLineTo(16f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, 2f)
                    horizontalLineTo(4f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, -2f)
                    verticalLineTo(7f)
                    arcToRelative(2f, 2f, 0f, false, true, 2f, -2f)
                    close()
                }
                // play triangle: M10 8.5l5 3.5-5 3.5z (filled)
                path(
                    fill = SolidColor(Color(0xFF000000)),
                    stroke = null,
                ) {
                    moveTo(10f, 8.5f)
                    lineToRelative(5f, 3.5f)
                    lineToRelative(-5f, 3.5f)
                    close()
                }
                // bottom stick: line x1=8 y1=21 x2=16 y2=21
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(8f, 21f)
                    horizontalLineTo(16f)
                }
                // line x1=12 y1=18 x2=12 y2=21
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(12f, 18f)
                    verticalLineTo(21f)
                }
            }.build()
    }

    val Tweet: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "Tweet",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                // X logo (filled): M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68
                // l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z
                path(
                    fill = SolidColor(Color(0xFF000000)),
                    stroke = null,
                ) {
                    moveTo(18.244f, 2.25f)
                    horizontalLineToRelative(3.308f)
                    lineToRelative(-7.227f, 8.26f)
                    lineToRelative(8.502f, 11.24f)
                    horizontalLineTo(16.17f)
                    lineToRelative(-5.214f, -6.817f)
                    lineTo(4.99f, 21.75f)
                    horizontalLineTo(1.68f)
                    lineToRelative(7.73f, -8.835f)
                    lineTo(1.254f, 2.25f)
                    horizontalLineTo(8.08f)
                    lineToRelative(4.713f, 6.231f)
                    close()
                    moveTo(17.083f, 19.77f)
                    horizontalLineToRelative(1.833f)
                    lineTo(7.084f, 4.126f)
                    horizontalLineTo(5.117f)
                    close()
                }
            }.build()
    }

    val Podcast: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "Podcast",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                // Headphone icon: M3 18v-6a9 9 0 0 1 18 0v6
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(3f, 18f)
                    verticalLineToRelative(-6f)
                    arcToRelative(9f, 9f, 0f, false, true, 18f, 0f)
                    verticalLineToRelative(6f)
                }
                // M21 19a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3v5z
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(21f, 19f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, 2f)
                    horizontalLineToRelative(-1f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, -2f)
                    verticalLineToRelative(-3f)
                    arcToRelative(2f, 2f, 0f, false, true, 2f, -2f)
                    horizontalLineToRelative(3f)
                    verticalLineToRelative(5f)
                    close()
                }
                // M3 19a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2v-3a2 2 0 0 0-2-2H3v5z
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(3f, 19f)
                    arcToRelative(2f, 2f, 0f, false, false, 2f, 2f)
                    horizontalLineToRelative(1f)
                    arcToRelative(2f, 2f, 0f, false, false, 2f, -2f)
                    verticalLineToRelative(-3f)
                    arcToRelative(2f, 2f, 0f, false, false, -2f, -2f)
                    horizontalLineTo(3f)
                    verticalLineToRelative(5f)
                    close()
                }
            }.build()
    }

    val Tag: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "Tag",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(20.59f, 13.41f)
                    lineToRelative(-7.17f, 7.17f)
                    arcToRelative(2f, 2f, 0f, false, true, -2.83f, 0f)
                    lineTo(2f, 12f)
                    verticalLineTo(2f)
                    horizontalLineToRelative(10f)
                    lineToRelative(8.59f, 8.59f)
                    arcToRelative(2f, 2f, 0f, false, true, 0f, 2.82f)
                    close()
                }
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(7f, 7f)
                    lineTo(7.01f, 7f)
                }
            }.build()
    }

    // 2x2 rounded-square grid — the sidebar "All items" leading glyph.
    val Grid: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "Grid",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    // top-left square (x=3 y=3 w=7.5 rx=2)
                    moveTo(5f, 3f)
                    lineTo(8.5f, 3f)
                    arcToRelative(2f, 2f, 0f, false, true, 2f, 2f)
                    lineTo(10.5f, 8.5f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, 2f)
                    lineTo(5f, 10.5f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, -2f)
                    lineTo(3f, 5f)
                    arcToRelative(2f, 2f, 0f, false, true, 2f, -2f)
                    close()
                    // top-right square (x=13.5 y=3)
                    moveTo(15.5f, 3f)
                    lineTo(19f, 3f)
                    arcToRelative(2f, 2f, 0f, false, true, 2f, 2f)
                    lineTo(21f, 8.5f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, 2f)
                    lineTo(15.5f, 10.5f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, -2f)
                    lineTo(13.5f, 5f)
                    arcToRelative(2f, 2f, 0f, false, true, 2f, -2f)
                    close()
                    // bottom-left square (x=3 y=13.5)
                    moveTo(5f, 13.5f)
                    lineTo(8.5f, 13.5f)
                    arcToRelative(2f, 2f, 0f, false, true, 2f, 2f)
                    lineTo(10.5f, 19f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, 2f)
                    lineTo(5f, 21f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, -2f)
                    lineTo(3f, 15.5f)
                    arcToRelative(2f, 2f, 0f, false, true, 2f, -2f)
                    close()
                    // bottom-right square (x=13.5 y=13.5)
                    moveTo(15.5f, 13.5f)
                    lineTo(19f, 13.5f)
                    arcToRelative(2f, 2f, 0f, false, true, 2f, 2f)
                    lineTo(21f, 19f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, 2f)
                    lineTo(15.5f, 21f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, -2f)
                    lineTo(13.5f, 15.5f)
                    arcToRelative(2f, 2f, 0f, false, true, 2f, -2f)
                    close()
                }
            }.build()
    }

    // Bookmark ribbon — the sidebar smart-list leading glyph.
    val SmartList: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "SmartList",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                // M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(19f, 21f)
                    lineTo(12f, 16f)
                    lineTo(5f, 21f)
                    verticalLineTo(5f)
                    arcToRelative(2f, 2f, 0f, false, true, 2f, -2f)
                    horizontalLineToRelative(10f)
                    arcToRelative(2f, 2f, 0f, false, true, 2f, 2f)
                    close()
                }
            }.build()
    }

    // Plus — "New collection" / "New smart list" placeholder rows.
    val Plus: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "Plus",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(12f, 5f)
                    verticalLineTo(19f)
                }
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(5f, 12f)
                    horizontalLineTo(19f)
                }
            }.build()
    }

    // Inbox tray — the scope popover "Inbox" view glyph.
    val Inbox: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "Inbox",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                // lid: M4 13l2.4-7.2A2 2 0 0 1 8.3 4.4h7.4a2 2 0 0 1 1.9 1.4L20 13
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(4f, 13f)
                    lineToRelative(2.4f, -7.2f)
                    arcToRelative(2f, 2f, 0f, false, true, 1.9f, -1.4f)
                    horizontalLineToRelative(7.4f)
                    arcToRelative(2f, 2f, 0f, false, true, 1.9f, 1.4f)
                    lineTo(20f, 13f)
                }
                // tray: M4 13h4l1.4 2.2h5.2L16 13h4v5a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2z
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(4f, 13f)
                    horizontalLineToRelative(4f)
                    lineToRelative(1.4f, 2.2f)
                    horizontalLineToRelative(5.2f)
                    lineTo(16f, 13f)
                    horizontalLineToRelative(4f)
                    verticalLineToRelative(5f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, 2f)
                    horizontalLineTo(6f)
                    arcToRelative(2f, 2f, 0f, false, true, -2f, -2f)
                    close()
                }
            }.build()
    }

    // Clock — the scope popover "Later" view glyph.
    val Clock: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "Clock",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                // circle cx=12 cy=12 r=8
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(4f, 12f)
                    arcToRelative(8f, 8f, 0f, true, true, 16f, 0f)
                    arcToRelative(8f, 8f, 0f, true, true, -16f, 0f)
                    close()
                }
                // hands: M12 8v4.2l2.8 1.8
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(12f, 8f)
                    verticalLineToRelative(4.2f)
                    lineToRelative(2.8f, 1.8f)
                }
            }.build()
    }

    // Gear — the sidebar footer "Settings" tile glyph.
    val Settings: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "Settings",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                // inner circle (cx=12 cy=12 r=3.3)
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(8.7f, 12f)
                    arcToRelative(3.3f, 3.3f, 0f, true, true, 6.6f, 0f)
                    arcToRelative(3.3f, 3.3f, 0f, true, true, -6.6f, 0f)
                    close()
                }
                // gear body
                path(
                    fill = null,
                    stroke = SolidColor(Color(0xFF000000)),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(19.4f, 12f)
                    arcToRelative(7.4f, 7.4f, 0f, false, false, -0.1f, -1.2f)
                    lineToRelative(2f, -1.55f)
                    lineToRelative(-2f, -3.46f)
                    lineToRelative(-2.36f, 0.96f)
                    arcToRelative(7.3f, 7.3f, 0f, false, false, -2.04f, -1.18f)
                    lineTo(14.4f, 3f)
                    horizontalLineToRelative(-4f)
                    lineToRelative(-0.5f, 2.59f)
                    arcToRelative(7.3f, 7.3f, 0f, false, false, -2.04f, 1.18f)
                    lineTo(5.5f, 5.81f)
                    lineToRelative(-2f, 3.46f)
                    lineToRelative(2f, 1.55f)
                    arcToRelative(7.4f, 7.4f, 0f, false, false, 0f, 2.36f)
                    lineToRelative(-2f, 1.55f)
                    lineToRelative(2f, 3.46f)
                    lineToRelative(2.36f, -0.96f)
                    arcToRelative(7.3f, 7.3f, 0f, false, false, 2.04f, 1.18f)
                    lineTo(10f, 21.5f)
                    horizontalLineToRelative(4f)
                    lineToRelative(0.5f, -2.59f)
                    arcToRelative(7.3f, 7.3f, 0f, false, false, 2.04f, -1.18f)
                    lineToRelative(2.36f, 0.96f)
                    lineToRelative(2f, -3.46f)
                    lineToRelative(-2f, -1.55f)
                    curveToRelative(0.07f, -0.39f, 0.1f, -0.79f, 0.1f, -1.18f)
                    close()
                }
            }.build()
    }

    val WarningTriangle: ImageVector by lazy {
        ImageVector
            .Builder(
                name = "WarningTriangle",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            ).apply {
                path(
                    stroke = SolidColor(Color.Black),
                    strokeLineWidth = 1.5f,
                    strokeLineCap = StrokeCap.Round,
                    strokeLineJoin = StrokeJoin.Round,
                ) {
                    moveTo(10.29f, 3.86f)
                    lineTo(1.82f, 18f)
                    arcTo(2f, 2f, 0f, false, false, 3.53f, 21f)
                    horizontalLineTo(20.47f)
                    arcTo(2f, 2f, 0f, false, false, 22.18f, 18f)
                    lineTo(13.71f, 3.86f)
                    arcTo(2f, 2f, 0f, false, false, 10.29f, 3.86f)
                    close()
                }
                path(
                    stroke = SolidColor(Color.Black),
                    strokeLineWidth = 1.5f,
                    strokeLineCap = StrokeCap.Round,
                ) {
                    moveTo(12f, 9f)
                    lineTo(12f, 13f)
                }
                path(
                    stroke = SolidColor(Color.Black),
                    strokeLineWidth = 2f,
                    strokeLineCap = StrokeCap.Round,
                ) {
                    moveTo(12f, 17f)
                    lineTo(12.01f, 17f)
                }
            }.build()
    }
}
