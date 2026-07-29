package app.indelible.reader.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.requiredSize
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.relocation.BringIntoViewRequester
import androidx.compose.foundation.relocation.bringIntoViewRequester
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import app.indelible.reader.model.ArticleTocEntry
import app.indelible.reader.viewmodel.TocPanelState
import app.indelible.reader.viewmodel.TocSections
import app.indelible.reader.viewmodel.TocStatus
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import app.indelible.ui.theme.SerifFontFamily
import app.indelible.ui.theme.geistMonoFontFamily

internal enum class TocDotState { DONE, HERE, UPCOMING }

internal fun dotState(
    index: Int,
    activeIndex: Int,
): TocDotState =
    when {
        index == activeIndex -> TocDotState.HERE
        index < activeIndex -> TocDotState.DONE
        else -> TocDotState.UPCOMING
    }

internal fun contentsEyebrow(
    status: TocStatus,
    progressPercent: Int,
): String =
    if (status == TocStatus.READY) {
        "Contents / $progressPercent% read"
    } else {
        "Contents"
    }

/** The Contents pill only appears once an outline actually exists. */
internal fun showContentsPill(status: TocStatus): Boolean = status == TocStatus.READY

/**
 * The Contents sheet: the prototype's rail of sections — progress dot plus
 * connector per row, section title, and a per-section minute estimate.
 *
 * The host scaffold already scrolls its content column, so this must stay a
 * plain Column: a lazy list here would be measured with infinite height and
 * crash. The outline is capped at 200 entries, so eager rows are cheap.
 */
@Composable
fun ContentsPanel(
    toc: TocPanelState,
    onEntryTap: (ArticleTocEntry) -> Unit,
    modifier: Modifier = Modifier,
) {
    when (toc.status) {
        TocStatus.READY -> {
            val activeRowRequester = remember { BringIntoViewRequester() }
            Column(
                modifier = modifier.fillMaxWidth().padding(vertical = IndelibleSpacing.step8),
            ) {
                toc.entries.forEachIndexed { index, entry ->
                    ContentsRow(
                        entry = entry,
                        dot = dotState(index, toc.activeIndex),
                        isFirst = index == 0,
                        isLast = index == toc.entries.lastIndex,
                        onTap = { onEntryTap(entry) },
                        modifier =
                            if (index == toc.activeIndex) {
                                Modifier.bringIntoViewRequester(activeRowRequester)
                            } else {
                                Modifier
                            },
                    )
                }
            }
            // The sheet composes fresh on every open, so this runs on open and
            // again whenever the reading position moves to another section —
            // the active row is scrolled into view without hoisting the
            // scaffold's scroll state.
            LaunchedEffect(toc.activeIndex) {
                if (toc.activeIndex >= 0) activeRowRequester.bringIntoView()
            }
        }

        TocStatus.LOADING, TocStatus.PENDING ->
            ContentsMessage(
                title = "Building the outline",
                body = "The table of contents is being prepared for this article. It usually takes a few seconds.",
                modifier = modifier,
            )

        TocStatus.NONE, TocStatus.UNAVAILABLE ->
            ContentsMessage(
                title = "No outline here",
                body = "This article doesn't have enough sections for a table of contents.",
                modifier = modifier,
            )
    }
}

/**
 * One outline row, matching the prototype's toc-item geometry: a fixed rail
 * column carrying an unbroken hairline through opaque dots, a serif title
 * (depth indents the title only, so the rail stays a single line), and a
 * top-aligned mono minute estimate.
 */
@Composable
private fun ContentsRow(
    entry: ArticleTocEntry,
    dot: TocDotState,
    isFirst: Boolean,
    isLast: Boolean,
    onTap: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .clickable(onClick = onTap)
                .padding(horizontal = IndelibleSpacing.step16)
                .height(IntrinsicSize.Min),
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
    ) {
        RailColumn(dot = dot, isFirst = isFirst, isLast = isLast)
        Text(
            text = entry.title,
            style = MaterialTheme.typography.bodyLarge.copy(fontFamily = SerifFontFamily),
            fontWeight = if (dot == TocDotState.HERE) FontWeight.SemiBold else FontWeight.Normal,
            color =
                if (dot == TocDotState.HERE) {
                    MaterialTheme.colorScheme.onSurface
                } else {
                    MaterialTheme.colorScheme.onSurfaceVariant
                },
            modifier =
                Modifier
                    .weight(1f)
                    .padding(
                        start = IndelibleSpacing.step12 * entry.depth,
                        bottom = IndelibleSpacing.step16,
                    ),
        )
        Text(
            text = "${TocSections.sectionMinutes(entry.wordCount)} min",
            style = MaterialTheme.typography.labelSmall.copy(fontFamily = geistMonoFontFamily()),
            color = IndelibleTheme.colors.textTertiary,
            modifier = Modifier.padding(top = IndelibleSpacing.step4),
        )
    }
}

/** Vertical offset from the row top to the dot, aligning it with the title's first line. */
private val RailDotTop = IndelibleSpacing.step6

@Composable
private fun RailColumn(
    dot: TocDotState,
    isFirst: Boolean,
    isLast: Boolean,
) {
    Box(modifier = Modifier.fillMaxHeight().width(IndelibleSpacing.step10)) {
        // One hairline drawn behind the opaque dot keeps the rail visually
        // unbroken across rows; the first and last rows trim it at the dot.
        val line = MaterialTheme.colorScheme.outlineVariant
        if (!isLast) {
            Box(
                modifier =
                    Modifier
                        .align(Alignment.TopCenter)
                        .padding(top = if (isFirst) RailDotTop else IndelibleSpacing.step0)
                        .fillMaxHeight()
                        .width(IndelibleSpacing.step2)
                        .background(line),
            )
        } else if (!isFirst) {
            Box(
                modifier =
                    Modifier
                        .align(Alignment.TopCenter)
                        .height(RailDotTop + IndelibleSpacing.step4)
                        .width(IndelibleSpacing.step2)
                        .background(line),
            )
        }
        if (dot == TocDotState.HERE) {
            Box(
                modifier =
                    Modifier
                        .align(Alignment.TopCenter)
                        .padding(top = IndelibleSpacing.step2)
                        .requiredSize(IndelibleSpacing.step16)
                        .background(
                            MaterialTheme.colorScheme.primary.copy(alpha = 0.18f),
                            CircleShape,
                        ),
            )
        }
        Box(
            modifier =
                Modifier
                    .align(Alignment.TopCenter)
                    .padding(top = RailDotTop)
                    .size(IndelibleSpacing.step8)
                    .then(
                        when (dot) {
                            TocDotState.HERE ->
                                Modifier.background(MaterialTheme.colorScheme.primary, CircleShape)

                            TocDotState.DONE ->
                                Modifier.background(IndelibleTheme.colors.textTertiary, CircleShape)

                            TocDotState.UPCOMING ->
                                Modifier
                                    .background(
                                        MaterialTheme.colorScheme.surfaceContainer,
                                        CircleShape,
                                    ).border(
                                        IndelibleSpacing.step2,
                                        MaterialTheme.colorScheme.outline,
                                        CircleShape,
                                    )
                        },
                    ),
        )
    }
}

@Composable
private fun ContentsMessage(
    title: String,
    body: String,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier.fillMaxWidth().padding(IndelibleSpacing.step24),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
    ) {
        Text(
            text = title,
            style = MaterialTheme.typography.titleMedium,
            color = MaterialTheme.colorScheme.onSurface,
        )
        Text(
            text = body,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
