package app.indelible.reader.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.reader.model.PdfStatus
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_retry
import indelible.composeapp.generated.resources.offline_manage_storage
import indelible.composeapp.generated.resources.reader_pdf_connection_body
import indelible.composeapp.generated.resources.reader_pdf_connection_title
import indelible.composeapp.generated.resources.reader_pdf_copy_removed
import indelible.composeapp.generated.resources.reader_pdf_download_failed_body
import indelible.composeapp.generated.resources.reader_pdf_download_failed_title
import indelible.composeapp.generated.resources.reader_pdf_no_space_body
import indelible.composeapp.generated.resources.reader_pdf_no_space_title
import indelible.composeapp.generated.resources.reader_pdf_unreadable_body
import indelible.composeapp.generated.resources.reader_pdf_unreadable_title
import indelible.composeapp.generated.resources.reader_pdf_waiting_body
import indelible.composeapp.generated.resources.reader_pdf_waiting_title
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource

private class PdfStatusCopy(
    val title: StringResource,
    val body: StringResource?,
    val retry: Boolean = false,
    val storage: Boolean = false,
    val progress: Boolean = false,
)

private fun copyFor(status: PdfStatus): PdfStatusCopy? =
    when (status) {
        PdfStatus.LOADING, PdfStatus.READY -> null
        PdfStatus.WAITING_FOR_COPY ->
            PdfStatusCopy(Res.string.reader_pdf_waiting_title, Res.string.reader_pdf_waiting_body, progress = true)
        PdfStatus.CONNECTION_FAILED ->
            PdfStatusCopy(Res.string.reader_pdf_connection_title, Res.string.reader_pdf_connection_body, retry = true)
        PdfStatus.NO_SPACE ->
            PdfStatusCopy(
                Res.string.reader_pdf_no_space_title,
                Res.string.reader_pdf_no_space_body,
                retry = true,
                storage = true,
            )
        PdfStatus.DOWNLOAD_FAILED ->
            PdfStatusCopy(
                Res.string.reader_pdf_download_failed_title,
                Res.string.reader_pdf_download_failed_body,
                retry = true,
            )
        PdfStatus.UNREADABLE ->
            PdfStatusCopy(Res.string.reader_pdf_unreadable_title, Res.string.reader_pdf_unreadable_body)
        PdfStatus.COPY_REMOVED -> PdfStatusCopy(Res.string.reader_pdf_copy_removed, body = null)
    }

/** Why the PDF is not on screen yet, with the download's progress or the action that can fix it. */
@Composable
fun PdfStatusContent(
    status: PdfStatus,
    downloadFraction: Float?,
    onRetry: () -> Unit,
    onManageStorage: () -> Unit,
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
            copy.body?.let {
                Text(
                    text = stringResource(it),
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    textAlign = TextAlign.Center,
                )
            }
            if (copy.progress) DownloadProgress(downloadFraction)
            Row(horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12)) {
                if (copy.storage) {
                    IndelibleButton(
                        text = stringResource(Res.string.offline_manage_storage),
                        onClick = onManageStorage,
                        style = IndelibleButtonStyle.Secondary,
                        compact = true,
                    )
                }
                if (copy.retry) {
                    IndelibleButton(
                        text = stringResource(Res.string.common_retry),
                        onClick = onRetry,
                        style = IndelibleButtonStyle.Secondary,
                        compact = true,
                    )
                }
            }
        }
    }
}

@Composable
private fun DownloadProgress(fraction: Float?) {
    val modifier = Modifier.fillMaxWidth().padding(top = IndelibleSpacing.step12)
    if (fraction == null) {
        LinearProgressIndicator(modifier = modifier)
    } else {
        LinearProgressIndicator(progress = { fraction }, modifier = modifier)
    }
}

@Preview
@Composable
private fun PdfStatusContentLightPreview() {
    AppTheme(darkTheme = false) {
        Surface {
            PdfStatusContent(
                status = PdfStatus.WAITING_FOR_COPY,
                downloadFraction = 0.4f,
                onRetry = {},
                onManageStorage = {},
                modifier = Modifier.fillMaxSize(),
            )
        }
    }
}

@Preview
@Composable
private fun PdfStatusContentDarkPreview() {
    AppTheme(darkTheme = true) {
        Surface {
            PdfStatusContent(
                status = PdfStatus.NO_SPACE,
                downloadFraction = null,
                onRetry = {},
                onManageStorage = {},
                modifier = Modifier.fillMaxSize(),
            )
        }
    }
}
