package app.indelible.library.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.library.viewmodel.TriageFilter
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing

@Composable
fun TriageSegmentedControl(
    selected: TriageFilter,
    onSelect: (TriageFilter) -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .padding(horizontal = IndelibleSpacing.step16)
                .background(
                    color = MaterialTheme.colorScheme.surfaceVariant,
                    shape = MaterialTheme.shapes.small,
                ).padding(IndelibleSpacing.step2),
    ) {
        TriageFilter.entries.forEach { filter ->
            val isActive = filter == selected
            Box(
                modifier =
                    Modifier
                        .weight(1f)
                        .then(
                            if (isActive) {
                                Modifier
                                    .shadow(
                                        elevation = IndelibleSpacing.step2,
                                        shape = IndelibleShape.sm,
                                        ambientColor = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.08f),
                                        spotColor = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.04f),
                                    ).background(
                                        color = MaterialTheme.colorScheme.surface,
                                        shape = IndelibleShape.sm,
                                    )
                            } else {
                                Modifier
                            },
                        ).clickable { onSelect(filter) }
                        .padding(
                            horizontal = IndelibleSpacing.step16,
                            vertical = IndelibleSpacing.step6,
                        ),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = filter.name,
                    style = MaterialTheme.typography.bodyMedium,
                    color =
                        if (isActive) {
                            MaterialTheme.colorScheme.onSurface
                        } else {
                            MaterialTheme.colorScheme.onSurfaceVariant
                        },
                )
            }
        }
    }
}

@Preview(showBackground = true)
@Composable
private fun TriageSegmentedControlPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            TriageSegmentedControl(
                selected = TriageFilter.INBOX,
                onSelect = {},
                modifier = Modifier.height(IndelibleSpacing.touchTarget),
            )
        }
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun TriageSegmentedControlPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            TriageSegmentedControl(
                selected = TriageFilter.LATER,
                onSelect = {},
                modifier = Modifier.height(IndelibleSpacing.touchTarget),
            )
        }
    }
}
