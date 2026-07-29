package app.indelible.onboarding.ui.components

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import app.indelible.ui.theme.IndelibleSpacing

/**
 * Selection card used during onboarding (e.g. library source, AI preference).
 *
 * States:
 *   - Default:  surface bg, border-secondary outline (1dp)
 *   - Selected: fill-selected bg (primaryContainer), accent border (1dp), accent text
 *
 * Typography:
 *   - Title:       titleSmall → callout (14sp/600)
 *   - Description: bodyMedium → subheadline (13sp/400)
 */
@Composable
fun ProviderCard(
    title: String,
    description: String,
    isSelected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Card(
        modifier =
            modifier
                .fillMaxWidth()
                .clickable(onClick = onClick),
        shape = MaterialTheme.shapes.medium, // radius-md: 10dp
        colors =
            CardDefaults.cardColors(
                containerColor =
                    if (isSelected) {
                        MaterialTheme.colorScheme.primaryContainer // fill-selected
                    } else {
                        MaterialTheme.colorScheme.surface // bg-primary
                    },
            ),
        border =
            BorderStroke(
                width = 1.dp, // style guide max for selection borders
                color =
                    if (isSelected) {
                        MaterialTheme.colorScheme.primary // accent
                    } else {
                        MaterialTheme.colorScheme.outline // border-secondary
                    },
            ),
    ) {
        Column(
            modifier = Modifier.padding(IndelibleSpacing.step16),
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.titleSmall, // callout: 14sp/600
                color =
                    if (isSelected) {
                        MaterialTheme.colorScheme.onPrimaryContainer // accent
                    } else {
                        MaterialTheme.colorScheme.onSurface // text-primary
                    },
            )
            Spacer(modifier = Modifier.height(IndelibleSpacing.step4))
            Text(
                text = description,
                style = MaterialTheme.typography.bodyMedium, // subheadline: 13sp/400
                color =
                    if (isSelected) {
                        MaterialTheme.colorScheme.onPrimaryContainer // accent
                    } else {
                        MaterialTheme.colorScheme.onSurfaceVariant // text-secondary
                    },
            )
        }
    }
}
