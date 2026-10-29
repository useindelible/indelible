package app.indelible.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_retry
import indelible.composeapp.generated.resources.offline_last_loaded
import org.jetbrains.compose.resources.stringResource

@Composable
fun LastLoadedNotice(
    onRetry: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier.fillMaxWidth().padding(horizontal = IndelibleSpacing.step16),
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = stringResource(Res.string.offline_last_loaded),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.weight(1f),
        )
        IndelibleButton(
            text = stringResource(Res.string.common_retry),
            onClick = onRetry,
            style = IndelibleButtonStyle.Text,
            compact = true,
        )
    }
}

@Preview(showBackground = true)
@Composable
private fun LastLoadedNoticePreviewLight() {
    AppTheme(darkTheme = false) {
        Surface { LastLoadedNotice(onRetry = {}) }
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun LastLoadedNoticePreviewDark() {
    AppTheme(darkTheme = true) {
        Surface { LastLoadedNotice(onRetry = {}) }
    }
}
