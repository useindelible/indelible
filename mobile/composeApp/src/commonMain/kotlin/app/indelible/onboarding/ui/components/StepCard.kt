package app.indelible.onboarding.ui.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import app.indelible.ui.theme.IndelibleSpacing

/**
 * Full-screen layout card used by every onboarding step.
 *
 * Typography:
 *   - title:    headlineLarge → title-1 (28sp/700)
 *   - subtitle: bodyLarge     → body (15sp/400)
 *   - colours:  onBackground / onSurfaceVariant
 *
 * Spacing: all values from [IndelibleSpacing] grid.
 */
@Composable
fun StepCard(
    title: String,
    subtitle: String? = null,
    modifier: Modifier = Modifier,
    content: @Composable ColumnScope.() -> Unit,
) {
    Column(
        modifier =
            modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(
                    horizontal = IndelibleSpacing.screenPaddingH,
                    vertical = IndelibleSpacing.screenPaddingV,
                ),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(
            text = title,
            style = MaterialTheme.typography.headlineLarge, // title-1: 28sp/700
            color = MaterialTheme.colorScheme.onBackground,
            textAlign = TextAlign.Center,
            modifier = Modifier.fillMaxWidth(),
        )

        if (subtitle != null) {
            Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
            Text(
                text = subtitle,
                style = MaterialTheme.typography.bodyLarge, // body: 15sp/400
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center,
                modifier = Modifier.fillMaxWidth(),
            )
        }

        Spacer(modifier = Modifier.height(IndelibleSpacing.step32))

        content()
    }
}
