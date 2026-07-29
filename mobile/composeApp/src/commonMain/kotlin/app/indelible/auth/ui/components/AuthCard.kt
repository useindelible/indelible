package app.indelible.auth.ui.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import app.indelible.ui.theme.IndelibleSpacing

/**
 * Full-screen layout wrapper used by all auth screens.
 *
 * The scrollable Box centres a Card (surface) in the middle of the screen,
 * matching the prototype's white/elevated card on the auth background.
 *
 *   - Shape:     MaterialTheme.shapes.extraLarge → radius-xl (14dp)
 *   - BG:        MaterialTheme.colorScheme.surface → bg-primary
 *   - Elevation: 2dp (elevation-2 per style guide: app icon / logo badge level)
 *   - Padding:   24dp horizontal, 32dp vertical (screen padding grid values)
 */
@Composable
fun AuthCard(
    modifier: Modifier = Modifier,
    content: @Composable ColumnScope.() -> Unit,
) {
    Box(
        modifier =
            modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState()),
        contentAlignment = Alignment.Center,
    ) {
        Card(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(
                        horizontal = IndelibleSpacing.screenPaddingH,
                        vertical = IndelibleSpacing.screenPaddingV,
                    ),
            shape = MaterialTheme.shapes.extraLarge,
            colors =
                CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.surface,
                ),
            elevation = CardDefaults.cardElevation(defaultElevation = IndelibleSpacing.step2),
        ) {
            Column(
                modifier = Modifier.padding(IndelibleSpacing.screenPaddingH),
                content = content,
            )
        }
    }
}
