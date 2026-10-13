package app.indelible.reader.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.reader.viewmodel.EpubStatus
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_retry
import indelible.composeapp.generated.resources.reader_epub_connection_body
import indelible.composeapp.generated.resources.reader_epub_connection_title
import indelible.composeapp.generated.resources.reader_epub_empty_body
import indelible.composeapp.generated.resources.reader_epub_empty_title
import indelible.composeapp.generated.resources.reader_epub_list_pending_body
import indelible.composeapp.generated.resources.reader_epub_list_pending_title
import indelible.composeapp.generated.resources.reader_epub_needs_reprocess_body
import indelible.composeapp.generated.resources.reader_epub_needs_reprocess_title
import indelible.composeapp.generated.resources.reader_epub_needs_reupload_body
import indelible.composeapp.generated.resources.reader_epub_needs_reupload_title
import indelible.composeapp.generated.resources.reader_epub_reprocess
import indelible.composeapp.generated.resources.reader_epub_server_too_old_body
import indelible.composeapp.generated.resources.reader_epub_server_too_old_title
import indelible.composeapp.generated.resources.reader_epub_unavailable_body
import indelible.composeapp.generated.resources.reader_epub_unavailable_title
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource

private enum class EpubAction { NONE, RETRY, REPROCESS }

private class EpubStatusCopy(
    val title: StringResource,
    val body: StringResource,
    val action: EpubAction,
)

private fun copyFor(status: EpubStatus): EpubStatusCopy? =
    when (status) {
        EpubStatus.LOADING, EpubStatus.READY -> null
        EpubStatus.CHAPTER_UNAVAILABLE ->
            EpubStatusCopy(
                Res.string.reader_epub_unavailable_title,
                Res.string.reader_epub_unavailable_body,
                EpubAction.RETRY,
            )
        EpubStatus.CONNECTION_FAILED ->
            EpubStatusCopy(
                Res.string.reader_epub_connection_title,
                Res.string.reader_epub_connection_body,
                EpubAction.RETRY,
            )
        EpubStatus.LIST_PENDING ->
            EpubStatusCopy(
                Res.string.reader_epub_list_pending_title,
                Res.string.reader_epub_list_pending_body,
                EpubAction.RETRY,
            )
        EpubStatus.NEEDS_REPROCESS ->
            EpubStatusCopy(
                Res.string.reader_epub_needs_reprocess_title,
                Res.string.reader_epub_needs_reprocess_body,
                EpubAction.REPROCESS,
            )
        EpubStatus.NEEDS_REUPLOAD ->
            EpubStatusCopy(
                Res.string.reader_epub_needs_reupload_title,
                Res.string.reader_epub_needs_reupload_body,
                EpubAction.NONE,
            )
        EpubStatus.SERVER_TOO_OLD ->
            EpubStatusCopy(
                Res.string.reader_epub_server_too_old_title,
                Res.string.reader_epub_server_too_old_body,
                EpubAction.NONE,
            )
        EpubStatus.EMPTY ->
            EpubStatusCopy(Res.string.reader_epub_empty_title, Res.string.reader_epub_empty_body, EpubAction.NONE)
    }

/** Why a book or its chapter cannot be shown, with the action that can fix it; nothing while loading or ready. */
@Composable
fun EpubStatusContent(
    status: EpubStatus,
    onRetry: () -> Unit,
    onReprocess: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val copy = copyFor(status) ?: return
    Box(modifier = modifier.padding(IndelibleSpacing.step32), contentAlignment = Alignment.Center) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
        ) {
            Text(
                text = stringResource(copy.title),
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onSurface,
                textAlign = TextAlign.Center,
            )
            Text(
                text = stringResource(copy.body),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center,
            )
            when (copy.action) {
                EpubAction.RETRY ->
                    IndelibleButton(
                        text = stringResource(Res.string.common_retry),
                        onClick = onRetry,
                        style = IndelibleButtonStyle.Secondary,
                        compact = true,
                    )
                EpubAction.REPROCESS ->
                    IndelibleButton(
                        text = stringResource(Res.string.reader_epub_reprocess),
                        onClick = onReprocess,
                        style = IndelibleButtonStyle.Secondary,
                        compact = true,
                    )
                EpubAction.NONE -> Unit
            }
        }
    }
}

@Preview
@Composable
private fun EpubStatusContentLightPreview() {
    AppTheme(darkTheme = false) {
        Surface {
            EpubStatusContent(
                status = EpubStatus.CONNECTION_FAILED,
                onRetry = {},
                onReprocess = {},
                modifier = Modifier.fillMaxSize(),
            )
        }
    }
}

@Preview
@Composable
private fun EpubStatusContentDarkPreview() {
    AppTheme(darkTheme = true) {
        Surface {
            EpubStatusContent(
                status = EpubStatus.NEEDS_REPROCESS,
                onRetry = {},
                onReprocess = {},
                modifier = Modifier.fillMaxSize(),
            )
        }
    }
}
