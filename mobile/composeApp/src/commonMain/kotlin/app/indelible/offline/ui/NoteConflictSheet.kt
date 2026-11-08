package app.indelible.offline.ui

import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.core.i18n.relativeTimeText
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.offline_conflict_body
import indelible.composeapp.generated.resources.offline_conflict_keep_mine
import indelible.composeapp.generated.resources.offline_conflict_keep_other
import indelible.composeapp.generated.resources.offline_conflict_other_device
import indelible.composeapp.generated.resources.offline_conflict_this_device
import indelible.composeapp.generated.resources.offline_failure_conflict_title
import kotlinx.datetime.Clock
import kotlinx.datetime.Instant
import org.jetbrains.compose.resources.stringResource

/** Both versions of a note that changed on two devices; the one kept replaces the other everywhere. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NoteConflictSheet(
    mine: String,
    theirs: String,
    theirsAt: Instant,
    onKeepMine: () -> Unit,
    onKeepOther: () -> Unit,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
) {
    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
        modifier = modifier,
    ) {
        NoteConflictContent(mine, theirs, theirsAt, Clock.System.now(), onKeepMine, onKeepOther)
    }
}

@Composable
internal fun NoteConflictContent(
    mine: String,
    theirs: String,
    theirsAt: Instant,
    now: Instant,
    onKeepMine: () -> Unit,
    onKeepOther: () -> Unit,
) {
    Column(
        modifier =
            Modifier.padding(
                start = IndelibleSpacing.rowPaddingH,
                end = IndelibleSpacing.rowPaddingH,
                bottom = IndelibleSpacing.step32,
            ),
    ) {
        Text(
            text = stringResource(Res.string.offline_failure_conflict_title),
            style = MaterialTheme.typography.titleLarge,
            color = MaterialTheme.colorScheme.onSurface,
        )
        Text(
            text = stringResource(Res.string.offline_conflict_body),
            style = MaterialTheme.typography.bodyMedium,
            color = IndelibleTheme.colors.textTertiary,
            modifier = Modifier.padding(top = IndelibleSpacing.step6, bottom = IndelibleSpacing.step16),
        )
        VersionCard(
            origin = stringResource(Res.string.offline_conflict_this_device),
            body = mine,
            keep = stringResource(Res.string.offline_conflict_keep_mine),
            onKeep = onKeepMine,
        )
        VersionCard(
            origin = stringResource(Res.string.offline_conflict_other_device, relativeTimeText(theirsAt, now)),
            body = theirs,
            keep = stringResource(Res.string.offline_conflict_keep_other),
            onKeep = onKeepOther,
            modifier = Modifier.padding(top = IndelibleSpacing.step12),
        )
    }
}

@Composable
private fun VersionCard(
    origin: String,
    body: String,
    keep: String,
    onKeep: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier =
            modifier
                .fillMaxWidth()
                .border(HAIRLINE, MaterialTheme.colorScheme.outlineVariant, IndelibleShape.lg)
                .padding(IndelibleSpacing.step14),
    ) {
        Text(
            text = origin,
            style = storageMono(TRACK_META, FontWeight.Normal),
            color = IndelibleTheme.colors.textTertiary,
        )
        Text(
            text = body,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.padding(top = IndelibleSpacing.step8),
        )
        IndelibleButton(
            text = keep,
            onClick = onKeep,
            style = IndelibleButtonStyle.Secondary,
            compact = true,
            modifier = Modifier.padding(top = IndelibleSpacing.step12),
        )
    }
}

private const val PREVIEW_NOW_MS = 1_767_225_600_000L
private const val PREVIEW_THEIRS_MS = 1_767_225_300_000L
private const val UI_MODE_NIGHT = 0x20

@Composable
private fun NoteConflictPreview() {
    NoteConflictContent(
        mine = "Read the Dewey chapter again before the seminar.",
        theirs = "Read the Dewey chapter again before the seminar, and bring the Rorty essay.",
        theirsAt = Instant.fromEpochMilliseconds(PREVIEW_THEIRS_MS),
        now = Instant.fromEpochMilliseconds(PREVIEW_NOW_MS),
        onKeepMine = {},
        onKeepOther = {},
    )
}

@Preview(showBackground = true)
@Composable
private fun NoteConflictPreviewLight() {
    AppTheme(darkTheme = false) { NoteConflictPreview() }
}

@Preview(showBackground = true, uiMode = UI_MODE_NIGHT)
@Composable
private fun NoteConflictPreviewDark() {
    AppTheme(darkTheme = true) { NoteConflictPreview() }
}
