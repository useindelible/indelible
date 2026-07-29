package app.indelible.sidebar.ui.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.profile.ui.components.UserAvatar
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.geistMonoFontFamily

/**
 * Drawer profile header (prototype `.dw-top`): avatar, display name, and a
 * monospace subtitle line. The close button is optional — the drawer is a
 * ModalNavigationDrawer whose scrim already dismisses on tap, so the prototype
 * omits it; pass [onClose] only when an explicit affordance is wanted.
 */
@Composable
fun SidebarProfileHeader(
    displayName: String,
    modifier: Modifier = Modifier,
    subtitle: String = "",
    avatarUrl: String? = null,
    avatarBytes: ByteArray? = null,
    onClose: (() -> Unit)? = null,
) {
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .statusBarsPadding()
                .padding(
                    start = IndelibleSpacing.step20,
                    end = if (onClose != null) IndelibleSpacing.step8 else IndelibleSpacing.step20,
                    top = IndelibleSpacing.step16,
                    bottom = IndelibleSpacing.step16,
                ),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        UserAvatar(
            displayName = displayName.ifEmpty { "?" },
            size = IndelibleSpacing.step48,
            textStyle = MaterialTheme.typography.headlineSmall,
            shape = IndelibleShape.lg,
            avatarUrl = avatarUrl,
            avatarBytes = avatarBytes,
        )
        Spacer(modifier = Modifier.width(IndelibleSpacing.step12))
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = displayName.ifEmpty { "My Library" },
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            if (subtitle.isNotBlank()) {
                Text(
                    text = subtitle,
                    style = MaterialTheme.typography.labelSmall.copy(fontFamily = geistMonoFontFamily()),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
        if (onClose != null) {
            IconButton(onClick = onClose) {
                Icon(
                    imageVector = Icons.Filled.Close,
                    contentDescription = "Close menu",
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Preview
@Composable
private fun SidebarProfileHeaderPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            SidebarProfileHeader(displayName = "Samuel Ajisegiri", subtitle = "289 saved items")
        }
    }
}

@Preview
@Composable
private fun SidebarProfileHeaderPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            SidebarProfileHeader(
                displayName = "Samuel Ajisegiri",
                subtitle = "289 saved items",
                onClose = {},
            )
        }
    }
}
