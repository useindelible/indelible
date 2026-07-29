package app.indelible.reader.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing

private const val PERCENT_MAX = 100f

/**
 * Reading position, as a hairline within the reader's bottom shelf. The track is
 * transparent so the paper shows through, and the fill is flat accent: the reader
 * carries no gradient fills.
 *
 * [progress] is a 0..100 percentage.
 */
@Composable
fun ReaderProgressBarV2(
    progress: Float,
    modifier: Modifier = Modifier,
) {
    val fraction = (progress / PERCENT_MAX).coerceIn(0f, 1f)
    Box(
        modifier =
            modifier
                .fillMaxWidth()
                .height(IndelibleSpacing.step2),
    ) {
        Box(
            modifier =
                Modifier
                    .fillMaxWidth(fraction)
                    .fillMaxHeight()
                    .background(MaterialTheme.colorScheme.primary),
        )
    }
}

@Preview
@Composable
private fun ReaderProgressBarV2LightPreview() {
    AppTheme(darkTheme = false) {
        ReaderProgressBarV2(progress = 42f)
    }
}

@Preview
@Composable
private fun ReaderProgressBarV2DarkPreview() {
    AppTheme(darkTheme = true) {
        ReaderProgressBarV2(progress = 78f)
    }
}
