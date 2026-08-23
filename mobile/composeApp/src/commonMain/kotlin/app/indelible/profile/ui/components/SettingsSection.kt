package app.indelible.profile.ui.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.em
import app.indelible.ui.theme.IndelibleSpacing

@Composable
fun SettingsSection(
    title: String,
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit,
) {
    Column(
        modifier = modifier.fillMaxWidth(),
    ) {
        Text(
            text = title,
            // caption-1: 11sp / medium weight / wide tracking — section labels
            style =
                MaterialTheme.typography.labelSmall.copy(
                    fontWeight = FontWeight.Medium,
                    letterSpacing = 0.06.em,
                ),
            color = MaterialTheme.colorScheme.onSurfaceVariant, // text-secondary
            modifier =
                Modifier.padding(
                    start = IndelibleSpacing.step16,
                    end = IndelibleSpacing.step16,
                    top = IndelibleSpacing.sectionGap,
                    bottom = IndelibleSpacing.step8,
                ),
        )
        content()
    }
}
