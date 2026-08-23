package app.indelible.reader.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_coming_contents_body
import indelible.composeapp.generated.resources.reader_coming_contents_title
import indelible.composeapp.generated.resources.reader_coming_epub_body
import indelible.composeapp.generated.resources.reader_coming_epub_title
import indelible.composeapp.generated.resources.reader_coming_pdf_body
import indelible.composeapp.generated.resources.reader_coming_pdf_title
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource

/** Which not-yet-supported surface the reader is standing in for. */
enum class ReaderComingSoonFormat { PDF, EPUB, CONTENTS }

private data class ComingSoonCopy(
    val icon: ImageVector,
    val titleRes: StringResource,
    val bodyRes: StringResource,
)

private fun copyFor(format: ReaderComingSoonFormat): ComingSoonCopy =
    when (format) {
        ReaderComingSoonFormat.PDF ->
            ComingSoonCopy(
                icon = IndelibleIcons.Pdf,
                titleRes = Res.string.reader_coming_pdf_title,
                bodyRes = Res.string.reader_coming_pdf_body,
            )

        ReaderComingSoonFormat.EPUB ->
            ComingSoonCopy(
                icon = IndelibleIcons.Book,
                titleRes = Res.string.reader_coming_epub_title,
                bodyRes = Res.string.reader_coming_epub_body,
            )

        ReaderComingSoonFormat.CONTENTS ->
            ComingSoonCopy(
                icon = IndelibleIcons.SmartList,
                titleRes = Res.string.reader_coming_contents_title,
                bodyRes = Res.string.reader_coming_contents_body,
            )
    }

/**
 * Placeholder for reader surfaces that have not shipped yet: the PDF and EPUB
 * readers, and the table of contents.
 */
@Composable
fun ReaderComingSoonContent(
    format: ReaderComingSoonFormat,
    modifier: Modifier = Modifier,
) {
    val copy = copyFor(format)

    Box(
        modifier = modifier.padding(IndelibleSpacing.step32),
        contentAlignment = Alignment.Center,
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
        ) {
            Icon(
                imageVector = copy.icon,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(IndelibleSpacing.step48),
            )
            Text(
                text = stringResource(copy.titleRes),
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onSurface,
                textAlign = TextAlign.Center,
            )
            Text(
                text = stringResource(copy.bodyRes),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center,
            )
        }
    }
}

@Preview
@Composable
private fun ReaderComingSoonContentPdfLightPreview() {
    AppTheme(darkTheme = false) {
        Surface {
            ReaderComingSoonContent(
                format = ReaderComingSoonFormat.PDF,
                modifier = Modifier.fillMaxSize(),
            )
        }
    }
}

@Preview
@Composable
private fun ReaderComingSoonContentEpubDarkPreview() {
    AppTheme(darkTheme = true) {
        Surface {
            ReaderComingSoonContent(
                format = ReaderComingSoonFormat.EPUB,
                modifier = Modifier.fillMaxSize(),
            )
        }
    }
}
