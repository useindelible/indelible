package app.indelible.ui.theme

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.path
import androidx.compose.ui.unit.dp

private val OUTLINE = SolidColor(Color(0xFF000000))
private const val STROKE = 2f

private fun readerIcon(
    name: String,
    block: ImageVector.Builder.() -> Unit,
): ImageVector =
    ImageVector
        .Builder(
            name = name,
            defaultWidth = 24.dp,
            defaultHeight = 24.dp,
            viewportWidth = 24f,
            viewportHeight = 24f,
        ).apply(block)
        .build()

private fun ImageVector.Builder.stroked(block: androidx.compose.ui.graphics.vector.PathBuilder.() -> Unit) {
    path(
        fill = null,
        stroke = OUTLINE,
        strokeLineWidth = STROKE,
        strokeLineCap = StrokeCap.Round,
        strokeLineJoin = StrokeJoin.Round,
        pathBuilder = block,
    )
}

private fun ImageVector.Builder.filled(block: androidx.compose.ui.graphics.vector.PathBuilder.() -> Unit) {
    path(fill = OUTLINE, stroke = null, pathBuilder = block)
}

// A dot is two half-arcs; the vector path DSL has no circle primitive.
private fun ImageVector.Builder.dot(
    cx: Float,
    cy: Float,
    r: Float,
) {
    filled {
        moveTo(cx - r, cy)
        arcToRelative(r, r, 0f, true, true, r * 2, 0f)
        arcToRelative(r, r, 0f, true, true, -r * 2, 0f)
        close()
    }
}

/**
 * The reader's authored icon set, traced from the reader design's own SVGs rather
 * than substituted from a stock library: the set shares one stroke weight, one
 * corner treatment and one optical size, which is what makes the dock read as a
 * single instrument instead of a row of borrowed glyphs.
 */
object ReaderIcons {
    /** Marker nib. */
    val Highlight: ImageVector by lazy {
        readerIcon("ReaderHighlight") {
            stroked {
                moveTo(4f, 19.5f)
                lineToRelative(1f, -3.5f)
                lineTo(15f, 6f)
                arcToRelative(2f, 2f, 0f, false, true, 2.8f, 0f)
                lineToRelative(0.2f, 0.2f)
                arcToRelative(2f, 2f, 0f, false, true, 0f, 2.8f)
                lineTo(8f, 19f)
                lineToRelative(-3.5f, 1f)
                close()
            }
            stroked {
                moveTo(13f, 8f)
                lineTo(16f, 11f)
            }
        }
    }

    /** Speech bubble with a tail. */
    val Note: ImageVector by lazy {
        readerIcon("ReaderNote") {
            stroked {
                moveTo(5f, 5f)
                horizontalLineToRelative(14f)
                arcToRelative(1f, 1f, 0f, false, true, 1f, 1f)
                verticalLineToRelative(9f)
                arcToRelative(1f, 1f, 0f, false, true, -1f, 1f)
                horizontalLineTo(10f)
                lineToRelative(-4f, 3.5f)
                verticalLineTo(16f)
                horizontalLineTo(5f)
                arcToRelative(1f, 1f, 0f, false, true, -1f, -1f)
                verticalLineTo(6f)
                arcToRelative(1f, 1f, 0f, false, true, 1f, -1f)
                close()
            }
        }
    }

    /** Arrow moving into a container. */
    val Move: ImageVector by lazy {
        readerIcon("ReaderMove") {
            stroked {
                moveTo(14f, 4f)
                horizontalLineToRelative(5f)
                arcToRelative(1f, 1f, 0f, false, true, 1f, 1f)
                verticalLineToRelative(14f)
                arcToRelative(1f, 1f, 0f, false, true, -1f, 1f)
                horizontalLineToRelative(-5f)
            }
            stroked {
                moveTo(3f, 12f)
                horizontalLineToRelative(11f)
            }
            stroked {
                moveTo(10.5f, 8.5f)
                lineTo(14f, 12f)
                lineToRelative(-3.5f, 3.5f)
            }
        }
    }

    /** Four-point spark; the assistant's mark. */
    val Mila: ImageVector by lazy {
        readerIcon("ReaderMila") {
            stroked {
                moveTo(12f, 3f)
                lineToRelative(1.9f, 5.1f)
                lineTo(19f, 10f)
                lineToRelative(-5.1f, 1.9f)
                lineTo(12f, 17f)
                lineToRelative(-1.9f, -5.1f)
                lineTo(5f, 10f)
                lineToRelative(5.1f, -1.9f)
                close()
            }
        }
    }

    /** Solid play triangle — the one filled glyph in the set, because it is the primary action. */
    val Listen: ImageVector by lazy {
        readerIcon("ReaderListen") {
            filled {
                moveTo(7f, 4f)
                lineTo(20f, 12f)
                lineTo(7f, 20f)
                close()
            }
        }
    }

    /** Bare chevron, not an arrow: back is navigation, not an action. */
    val Back: ImageVector by lazy {
        readerIcon("ReaderBack") {
            stroked {
                moveTo(15f, 5f)
                lineToRelative(-7f, 7f)
                lineToRelative(7f, 7f)
            }
        }
    }

    /** Bulleted list — the article's own outline. */
    val Contents: ImageVector by lazy {
        readerIcon("ReaderContents") {
            stroked {
                moveTo(9f, 7f)
                horizontalLineTo(20f)
            }
            stroked {
                moveTo(9f, 12f)
                horizontalLineTo(20f)
            }
            stroked {
                moveTo(9f, 17f)
                horizontalLineTo(16f)
            }
            dot(4.5f, 7f, 1.1f)
            dot(4.5f, 12f, 1.1f)
            dot(4.5f, 17f, 1.1f)
        }
    }

    /** Vertical ellipsis. */
    val More: ImageVector by lazy {
        readerIcon("ReaderMore") {
            dot(12f, 5.5f, 1.4f)
            dot(12f, 12f, 1.4f)
            dot(12f, 18.5f, 1.4f)
        }
    }

    /** Bookmark: saving a feed item is filing it, not starring it. */
    val Save: ImageVector by lazy {
        readerIcon("ReaderSave") {
            stroked {
                moveTo(7f, 4f)
                horizontalLineToRelative(10f)
                arcToRelative(1f, 1f, 0f, false, true, 1f, 1f)
                verticalLineToRelative(15f)
                lineToRelative(-6f, -4f)
                lineToRelative(-6f, 4f)
                verticalLineTo(5f)
                arcToRelative(1f, 1f, 0f, false, true, 1f, -1f)
                close()
            }
        }
    }
}
