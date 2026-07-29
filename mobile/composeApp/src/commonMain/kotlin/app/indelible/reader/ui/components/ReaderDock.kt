package app.indelible.reader.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.sp
import app.indelible.reader.model.DataPanel
import app.indelible.ui.platform.rememberHapticTick
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.ReaderIcons
import app.indelible.ui.theme.SerifFontFamily

// An open tool reads as a filled wash. The previous 4dp dot under the glyph was too
// quiet to answer "which panel is open" at a glance.
private const val ACTIVE_WASH_ALPHA = 0.10f

/**
 * The reader's tool dock.
 *
 * Grouped, not evenly spaced: annotate (highlight, note), then organise (move, type),
 * then the assistant, then Listen. Buttons inside a group sit tight together and the
 * groups are separated by a hairline, so the arrangement carries the meaning without
 * labels. Listen is the primary action and says so — a labelled accent pill rather
 * than a sixth identical square.
 */
@Composable
fun ReaderDock(
    activePanel: DataPanel,
    onPanelSelected: (DataPanel) -> Unit,
    modifier: Modifier = Modifier,
) {
    val hapticTick = rememberHapticTick()
    val select: (DataPanel) -> Unit = { panel ->
        hapticTick()
        onPanelSelected(panel)
    }
    Surface(
        modifier = modifier,
        shape = IndelibleShape.xxl,
        color = MaterialTheme.colorScheme.surfaceContainer,
        shadowElevation = IndelibleSpacing.step8,
    ) {
        Row(
            modifier = Modifier.padding(IndelibleSpacing.step6),
            horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step6),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            DockZone {
                DockButton(
                    active = activePanel == DataPanel.HL,
                    contentDescription = "Highlight",
                    onClick = { select(DataPanel.HL) },
                ) { DockIcon(ReaderIcons.Highlight) }
                DockButton(
                    active = activePanel == DataPanel.NOTE,
                    contentDescription = "Note",
                    onClick = { select(DataPanel.NOTE) },
                ) { DockIcon(ReaderIcons.Note) }
            }

            DockRule()

            DockZone {
                DockButton(
                    active = activePanel == DataPanel.MOVE,
                    contentDescription = "Move",
                    onClick = { select(DataPanel.MOVE) },
                ) { DockIcon(ReaderIcons.Move) }
                DockRule()
                DockButton(
                    active = activePanel == DataPanel.AA,
                    contentDescription = "Text options",
                    onClick = { select(DataPanel.AA) },
                ) {
                    Text(
                        text = "Aa",
                        fontFamily = SerifFontFamily,
                        fontSize = 18.sp,
                        fontWeight = FontWeight.SemiBold,
                    )
                }
            }

            // Mila stands outside the zones: it is the one control here that is not a
            // reading tool, so it carries the accent at rest and no group of its own.
            DockButton(
                active = activePanel == DataPanel.MILA,
                contentDescription = "Ask Mila",
                accented = true,
                onClick = { select(DataPanel.MILA) },
            ) { DockIcon(ReaderIcons.Mila) }

            DockRule()

            DockListenButton(onClick = { select(DataPanel.LISTEN) })
        }
    }
}

@Composable
private fun DockZone(content: @Composable () -> Unit) {
    Row(
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step2),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        content()
    }
}

@Composable
private fun DockIcon(icon: ImageVector) {
    Icon(
        imageVector = icon,
        contentDescription = null,
        modifier = Modifier.size(IndelibleSpacing.step20),
    )
}

@Composable
private fun DockRule() {
    Box(
        modifier =
            Modifier
                .width(IndelibleSpacing.hairline)
                .height(IndelibleSpacing.step24)
                .background(MaterialTheme.colorScheme.outlineVariant),
    )
}

@Composable
private fun DockButton(
    active: Boolean,
    contentDescription: String,
    onClick: () -> Unit,
    accented: Boolean = false,
    content: @Composable () -> Unit,
) {
    val foreground =
        when {
            active || accented -> MaterialTheme.colorScheme.primary
            else -> MaterialTheme.colorScheme.onSurfaceVariant
        }
    Box(
        modifier =
            Modifier
                .size(IndelibleSpacing.step40)
                .clip(IndelibleShape.sm)
                .background(
                    if (active) {
                        MaterialTheme.colorScheme.primary.copy(alpha = ACTIVE_WASH_ALPHA)
                    } else {
                        Color.Transparent
                    },
                )
                .clickable(onClickLabel = contentDescription, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        CompositionLocalProvider(LocalContentColor provides foreground) {
            content()
        }
    }
}

@Composable
private fun DockListenButton(onClick: () -> Unit) {
    Row(
        modifier =
            Modifier
                .height(IndelibleSpacing.step40)
                .clip(IndelibleShape.sm)
                .background(MaterialTheme.colorScheme.primary)
                .clickable(onClickLabel = "Listen", onClick = onClick)
                .padding(start = IndelibleSpacing.step12, end = IndelibleSpacing.step14),
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            imageVector = ReaderIcons.Listen,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onPrimary,
            modifier = Modifier.size(IndelibleSpacing.step16),
        )
        Text(
            text = "Listen",
            fontSize = 13.5.sp,
            fontWeight = FontWeight.SemiBold,
            color = MaterialTheme.colorScheme.onPrimary,
        )
    }
}

@Preview
@Composable
private fun ReaderDockPreviewLight() {
    AppTheme(darkTheme = false) {
        ReaderDock(
            activePanel = DataPanel.HL,
            onPanelSelected = {},
            modifier = Modifier.padding(IndelibleSpacing.step16),
        )
    }
}

@Preview
@Composable
private fun ReaderDockPreviewDark() {
    AppTheme(darkTheme = true) {
        ReaderDock(
            activePanel = DataPanel.NONE,
            onPanelSelected = {},
            modifier = Modifier.padding(IndelibleSpacing.step16),
        )
    }
}
