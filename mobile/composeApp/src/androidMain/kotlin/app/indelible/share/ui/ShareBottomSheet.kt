package app.indelible.share.ui

import android.content.res.Configuration.UI_MODE_NIGHT_YES
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Bookmark
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Error
import androidx.compose.material.icons.filled.WifiOff
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.share.viewmodel.ShareUiState
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ShareBottomSheet(
    url: String,
    uiState: ShareUiState,
    onSave: () -> Unit,
    onDismiss: () -> Unit,
    onSignIn: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val sheetState = rememberModalBottomSheetState()

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        shape = MaterialTheme.shapes.large,
        modifier = modifier,
    ) {
        Column(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = IndelibleSpacing.screenPaddingH)
                    .padding(bottom = IndelibleSpacing.screenPaddingV),
        ) {
            ShareSheetHeader(url = url)

            Spacer(modifier = Modifier.height(IndelibleSpacing.step24))

            ShareStateContent(uiState = uiState, onSave = onSave, onSignIn = onSignIn)
        }
    }
}

@Composable
private fun ShareSheetHeader(url: String) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            imageVector = Icons.Default.Bookmark,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.primary,
            modifier = Modifier.size(IndelibleSpacing.step20),
        )
        Spacer(modifier = Modifier.width(IndelibleSpacing.step8))
        Text(
            text = "Save to Indelible",
            style = MaterialTheme.typography.headlineMedium,
            color = MaterialTheme.colorScheme.onSurface,
        )
    }

    Spacer(modifier = Modifier.height(IndelibleSpacing.step16))
    HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
    Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

    Text(
        text = url,
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        maxLines = 2,
    )

    Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

    Text(
        text = "Inbox (default)",
        style = MaterialTheme.typography.bodySmall,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
    )
}

@Composable
private fun ShareStatusRow(
    icon: ImageVector,
    iconTint: Color,
    text: String,
    textColor: Color = MaterialTheme.colorScheme.onSurface,
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier.fillMaxWidth(),
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = iconTint,
            modifier = Modifier.size(IndelibleSpacing.step20),
        )
        Spacer(modifier = Modifier.width(IndelibleSpacing.step8))
        Text(
            text = text,
            style = MaterialTheme.typography.bodyLarge,
            color = textColor,
        )
    }
}

@Composable
private fun ShareStateContent(
    uiState: ShareUiState,
    onSave: () -> Unit,
    onSignIn: () -> Unit,
) {
    when (uiState) {
        is ShareUiState.Idle, is ShareUiState.Saving -> {
            IndelibleButton(
                text = "Save",
                onClick = onSave,
                isLoading = uiState is ShareUiState.Saving,
            )
        }
        is ShareUiState.Success, is ShareUiState.AlreadySaved -> {
            ShareStatusRow(
                icon = Icons.Default.CheckCircle,
                iconTint = MaterialTheme.colorScheme.primary,
                text = if (uiState is ShareUiState.AlreadySaved) "Already saved" else "Saved!",
            )
        }
        is ShareUiState.Queued -> {
            ShareStatusRow(
                icon = Icons.Default.WifiOff,
                iconTint = MaterialTheme.colorScheme.onSurfaceVariant,
                text = "Saved offline — will sync when online",
            )
        }
        is ShareUiState.AuthRequired -> {
            Text(
                text = "Sign in to Indelible first",
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.error,
            )
            Spacer(modifier = Modifier.height(IndelibleSpacing.step16))
            IndelibleButton(
                text = "Open Indelible",
                onClick = onSignIn,
            )
        }
        is ShareUiState.InvalidUrl -> {
            ShareStatusRow(
                icon = Icons.Default.Error,
                iconTint = MaterialTheme.colorScheme.error,
                text = "This doesn't look like a valid URL",
                textColor = MaterialTheme.colorScheme.error,
            )
        }
        is ShareUiState.Error -> {
            ShareStatusRow(
                icon = Icons.Default.Error,
                iconTint = MaterialTheme.colorScheme.error,
                text = uiState.message,
                textColor = MaterialTheme.colorScheme.error,
            )
        }
    }
}

@Preview(showBackground = true)
@Composable
private fun ShareBottomSheetPreviewLight() {
    AppTheme(darkTheme = false) {
        ShareBottomSheet(
            url = "https://example.com/an-interesting-article",
            uiState = ShareUiState.Idle,
            onSave = {},
            onDismiss = {},
            onSignIn = {},
        )
    }
}

@Preview(showBackground = true, uiMode = UI_MODE_NIGHT_YES)
@Composable
private fun ShareBottomSheetPreviewDark() {
    AppTheme(darkTheme = true) {
        ShareBottomSheet(
            url = "https://example.com/an-interesting-article",
            uiState = ShareUiState.Idle,
            onSave = {},
            onDismiss = {},
            onSignIn = {},
        )
    }
}

@Preview(showBackground = true)
@Composable
private fun ShareBottomSheetSuccessPreview() {
    AppTheme(darkTheme = false) {
        ShareBottomSheet(
            url = "https://example.com/an-interesting-article",
            uiState = ShareUiState.Queued,
            onSave = {},
            onDismiss = {},
            onSignIn = {},
        )
    }
}
