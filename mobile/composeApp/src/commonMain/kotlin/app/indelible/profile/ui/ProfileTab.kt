package app.indelible.profile.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import app.indelible.auth.viewmodel.AuthState
import app.indelible.auth.viewmodel.AuthViewModel
import app.indelible.profile.ui.components.SettingsRow
import app.indelible.profile.ui.components.SettingsSection
import app.indelible.profile.ui.components.UserAvatar
import app.indelible.ui.theme.IndelibleSpacing

@Composable
fun ProfileTab(
    authViewModel: AuthViewModel,
    onNavigateToEdit: () -> Unit,
    onNavigateToPreferences: () -> Unit,
    onNavigateToAi: () -> Unit,
    onNavigateToIntegrations: () -> Unit,
    onNavigateToAccount: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val authState by authViewModel.authState.collectAsState()
    val user = (authState as? AuthState.Authenticated)?.user
    val avatarBytes by authViewModel.avatarBytes.collectAsState()

    Column(
        modifier =
            modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState()),
    ) {
        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

        Text(
            text = "Profile",
            style = MaterialTheme.typography.headlineMedium,
            modifier = Modifier.padding(horizontal = IndelibleSpacing.step16),
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

        ProfileHero(
            displayName = user?.displayName ?: "User",
            email = user?.email ?: "",
            avatarUrl = user?.avatarUrl,
            avatarBytes = avatarBytes,
            onClick = onNavigateToEdit,
        )

        HorizontalDivider()

        SettingsSection(title = "Settings") {
            SettingsRow(
                label = "Preferences",
                sublabel = "Appearance, Reader, Triage & Workflow",
                onClick = onNavigateToPreferences,
            )
            SettingsRow(
                label = "Mila & AI",
                sublabel = "Enable Mila, provider, model, prompt presets",
                onClick = onNavigateToAi,
            )
            SettingsRow(
                label = "Content",
                sublabel = "Feeds, email ingest, add to library",
                onClick = onNavigateToIntegrations,
            )
            SettingsRow(
                label = "Account",
                sublabel = "Password, Storage, Sign out",
                onClick = onNavigateToAccount,
            )
        }

        Spacer(modifier = Modifier.height(IndelibleSpacing.step32))
    }
}

@Composable
private fun ProfileHero(
    displayName: String,
    email: String,
    avatarUrl: String?,
    avatarBytes: ByteArray?,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .clickable(onClick = onClick)
                .padding(IndelibleSpacing.step16),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        UserAvatar(
            displayName = displayName,
            avatarUrl = avatarUrl,
            avatarBytes = avatarBytes,
            size = IndelibleSpacing.step64,
            textStyle = MaterialTheme.typography.headlineSmall,
        )

        Spacer(modifier = Modifier.width(IndelibleSpacing.step16))

        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = displayName,
                style = MaterialTheme.typography.headlineSmall,
            )
            Text(
                text = email,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        Icon(
            imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(IndelibleSpacing.step20),
        )
    }
}
