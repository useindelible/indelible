package app.indelible.offline.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import app.indelible.offline.viewmodel.StorageUiState
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.offline_cap_title
import indelible.composeapp.generated.resources.offline_items
import indelible.composeapp.generated.resources.offline_key_cached
import indelible.composeapp.generated.resources.offline_key_free
import indelible.composeapp.generated.resources.offline_key_kept
import indelible.composeapp.generated.resources.offline_manage_kept
import indelible.composeapp.generated.resources.offline_manage_kept_body
import indelible.composeapp.generated.resources.offline_used_line
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource

private const val MB = 1024L * 1024
private const val HALF_GB_IN_MB = 500L
private const val ONE_GB_IN_MB = 1024L
private const val TWO_GB_IN_MB = 2048L
private const val FIVE_GB_IN_MB = 5120L

/** The caps the segmented control offers; the stored default is one of them. */
internal val capChoices =
    listOf(HALF_GB_IN_MB * MB, ONE_GB_IN_MB * MB, TWO_GB_IN_MB * MB, FIVE_GB_IN_MB * MB)

private val METER_HEIGHT = 6.dp
private val KEY_DOT = 6.dp
private const val CACHED_ALPHA = 0.42f
private val TRACK_KEY = 0.04.em
private val TRACK_USED = 0.06.em
private val TRACK_USED_NUMBER = -0.02.em

@Composable
internal fun UsageSection(
    state: StorageUiState,
    onCapChange: (Long) -> Unit,
    onManageKept: () -> Unit,
) {
    Column(
        modifier =
            Modifier
                .fillMaxWidth()
                .padding(
                    start = IndelibleSpacing.rowPaddingH,
                    end = IndelibleSpacing.rowPaddingH,
                    top = IndelibleSpacing.step20,
                ),
    ) {
        SectionLabel(stringResource(Res.string.offline_cap_title))
        StorageSegmented(
            segments = capChoices.map { Segment(byteSize(it)) },
            selected = capChoices.indexOf(state.capBytes),
            onSelect = { onCapChange(capChoices[it]) },
            modifier = Modifier.padding(top = IndelibleSpacing.step12),
        )
        UsedLine(state, Modifier.padding(top = IndelibleSpacing.step16))
        UsageMeter(state, Modifier.padding(top = IndelibleSpacing.step16))
        MeterKey(state, Modifier.padding(top = IndelibleSpacing.step12))
        if (state.keptOverCap) {
            Text(
                text = stringResource(Res.string.offline_manage_kept_body),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.error,
                modifier = Modifier.padding(top = IndelibleSpacing.step12),
            )
            IndelibleButton(
                text = stringResource(Res.string.offline_manage_kept),
                onClick = onManageKept,
                style = IndelibleButtonStyle.Secondary,
                modifier = Modifier.padding(top = IndelibleSpacing.step8),
            )
        }
    }
}

/** The used size set large inside the one translated line, so the sentence stays whole. */
@Composable
private fun UsedLine(
    state: StorageUiState,
    modifier: Modifier,
) {
    val used = byteSize(state.usedBytes)
    val line =
        stringResource(
            Res.string.offline_used_line,
            used,
            byteSize(state.capBytes),
            pluralStringResource(Res.plurals.offline_items, state.itemCount, state.itemCount),
        )
    val start = line.indexOf(used)
    val number =
        SpanStyle(
            fontSize = MaterialTheme.typography.headlineMedium.fontSize,
            letterSpacing = TRACK_USED_NUMBER,
            color = MaterialTheme.colorScheme.onSurface,
        )
    Text(
        text =
            buildAnnotatedString {
                append(line)
                if (start >= 0) addStyle(number, start, start + used.length)
            },
        style = storageMono(TRACK_USED, FontWeight.Normal),
        color = IndelibleTheme.colors.textTertiary,
        modifier = modifier,
    )
}

/** Kept, cached and free as separate bars, so the cap shows where the space went. */
@Composable
private fun UsageMeter(
    state: StorageUiState,
    modifier: Modifier,
) {
    val total = maxOf(state.capBytes, state.usedBytes).coerceAtLeast(1)
    val free = (state.capBytes - state.usedBytes).coerceAtLeast(0)
    Row(
        modifier = modifier.fillMaxWidth().height(METER_HEIGHT),
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step2),
    ) {
        MeterSegment(state.keptBytes, total, keptInk())
        MeterSegment(state.cachedBytes, total, cachedInk())
        MeterSegment(free, total, MaterialTheme.colorScheme.outline)
    }
}

@Composable
private fun keptInk() = MaterialTheme.colorScheme.primary

@Composable
private fun cachedInk() = MaterialTheme.colorScheme.primary.copy(alpha = CACHED_ALPHA)

@Composable
private fun RowScope.MeterSegment(
    bytes: Long,
    total: Long,
    color: Color,
) {
    if (bytes <= 0) return
    Box(
        modifier =
            Modifier
                .weight(bytes.toFloat() / total)
                .height(METER_HEIGHT)
                .clip(IndelibleShape.xs)
                .background(color),
    )
}

/** Only the parts that hold something; an empty device needs no key at all. */
@Composable
private fun MeterKey(
    state: StorageUiState,
    modifier: Modifier,
) {
    if (state.usedBytes == 0L) return
    Row(modifier = modifier, horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step14)) {
        if (state.keptBytes > 0) {
            KeyEntry(keptInk(), stringResource(Res.string.offline_key_kept, byteSize(state.keptBytes)))
        }
        if (state.cachedBytes > 0) {
            KeyEntry(cachedInk(), stringResource(Res.string.offline_key_cached, byteSize(state.cachedBytes)))
        }
        KeyEntry(
            MaterialTheme.colorScheme.outline,
            stringResource(Res.string.offline_key_free, byteSize((state.capBytes - state.usedBytes).coerceAtLeast(0))),
        )
    }
}

@Composable
private fun KeyEntry(
    color: Color,
    label: String,
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step6),
    ) {
        Box(
            modifier =
                Modifier
                    .size(KEY_DOT)
                    .clip(IndelibleShape.xs)
                    .background(color),
        )
        Text(
            text = label,
            style = storageMono(TRACK_KEY, FontWeight.Medium),
            color = IndelibleTheme.colors.textTertiary,
        )
    }
}
