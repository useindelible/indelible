package app.indelible.library.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.library.viewmodel.TriageFilter
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing

@Composable
fun LibraryEmptyState(
    triageFilter: TriageFilter,
    modifier: Modifier = Modifier,
) {
    val (title, subtitle) =
        when (triageFilter) {
            TriageFilter.INBOX -> "Your inbox is empty" to "Save articles, books, and more to get started"
            TriageFilter.LATER -> "Nothing queued for later" to "Swipe right on an item to save it for later"
            TriageFilter.ARCHIVE -> "Your archive is empty" to "Items you've finished will appear here"
        }

    Column(
        modifier =
            modifier
                .fillMaxSize()
                .padding(
                    top = IndelibleSpacing.step64,
                    start = IndelibleSpacing.screenPaddingH,
                    end = IndelibleSpacing.screenPaddingH,
                ),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Top,
    ) {
        Text(
            text = title,
            style = MaterialTheme.typography.headlineMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
        )
        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
        Text(
            text = subtitle,
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
        )
    }
}

@Preview(showBackground = true)
@Composable
private fun LibraryEmptyStatePreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            LibraryEmptyState(triageFilter = TriageFilter.INBOX)
        }
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun LibraryEmptyStatePreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            LibraryEmptyState(triageFilter = TriageFilter.LATER)
        }
    }
}
