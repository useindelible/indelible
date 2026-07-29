package app.indelible.auth.ui.components

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import app.indelible.ui.theme.IndelibleSpacing

@Composable
fun OAuthButton(
    providerName: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    OutlinedButton(
        onClick = onClick,
        modifier =
            modifier
                .fillMaxWidth()
                .height(IndelibleSpacing.touchTarget),
    ) {
        Text(
            text = "Continue with $providerName",
            style = MaterialTheme.typography.bodyMedium,
        )
    }
}
