package app.indelible.sidebar.ui.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.em
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.geistMonoFontFamily

/**
 * Uppercase monospace group heading in the drawer (prototype `.dw-grouplab`):
 * wide tracking, tertiary colour. Tracking is set via `.copy` on `labelSmall`,
 * which the type-scale doc sanctions for section labels.
 */
@Composable
fun SidebarGroupLabel(
    text: String,
    modifier: Modifier = Modifier,
) {
    Text(
        text = text.uppercase(),
        style =
            MaterialTheme.typography.labelSmall.copy(
                fontFamily = geistMonoFontFamily(),
                fontWeight = FontWeight.SemiBold,
                letterSpacing = 0.14.em,
            ),
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier =
            modifier.padding(
                start = IndelibleSpacing.step12,
                end = IndelibleSpacing.step12,
                top = IndelibleSpacing.step14,
                bottom = IndelibleSpacing.step6,
            ),
    )
}

@Preview
@Composable
private fun SidebarGroupLabelPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            Column {
                SidebarGroupLabel("Library")
                SidebarGroupLabel("Collections")
            }
        }
    }
}

@Preview
@Composable
private fun SidebarGroupLabelPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            SidebarGroupLabel("Smart Lists")
        }
    }
}
