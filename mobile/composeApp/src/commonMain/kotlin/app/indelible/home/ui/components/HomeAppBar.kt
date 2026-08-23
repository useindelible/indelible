package app.indelible.home.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Menu
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.profile.ui.components.UserAvatar
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.home_open_menu_cd
import indelible.composeapp.generated.resources.home_search_cd
import org.jetbrains.compose.resources.stringResource

/**
 * Transparent home top bar: drawer menu on the left, search + profile avatar on
 * the right. Sits above the scrolling content and clears the status bar inset.
 */
@Composable
fun HomeAppBar(
    userDisplayName: String,
    onMenuClick: () -> Unit,
    onSearchClick: () -> Unit,
    onProfileClick: () -> Unit,
    modifier: Modifier = Modifier,
    avatarUrl: String? = null,
    avatarBytes: ByteArray? = null,
) {
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .statusBarsPadding()
                .padding(horizontal = IndelibleSpacing.step8, vertical = IndelibleSpacing.step6),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        IconButton(onClick = onMenuClick) {
            Icon(
                imageVector = Icons.Filled.Menu,
                contentDescription = stringResource(Res.string.home_open_menu_cd),
                tint = MaterialTheme.colorScheme.primary,
            )
        }
        Spacer(Modifier.weight(1f))
        IconButton(onClick = onSearchClick) {
            Icon(
                imageVector = Icons.Filled.Search,
                contentDescription = stringResource(Res.string.home_search_cd),
                tint = MaterialTheme.colorScheme.primary,
            )
        }
        Spacer(Modifier.width(IndelibleSpacing.step4))
        UserAvatar(
            displayName = userDisplayName,
            size = IndelibleSpacing.step32,
            textStyle = MaterialTheme.typography.labelSmall,
            avatarUrl = avatarUrl,
            avatarBytes = avatarBytes,
            modifier =
                Modifier
                    .clip(IndelibleShape.full)
                    .clickable(onClick = onProfileClick),
        )
    }
}

@Preview
@Composable
private fun HomeAppBarPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            HomeAppBar(
                userDisplayName = "Maya Lindqvist",
                onMenuClick = {},
                onSearchClick = {},
                onProfileClick = {},
            )
        }
    }
}

@Preview
@Composable
private fun HomeAppBarPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            HomeAppBar(
                userDisplayName = "Maya Lindqvist",
                onMenuClick = {},
                onSearchClick = {},
                onProfileClick = {},
            )
        }
    }
}
