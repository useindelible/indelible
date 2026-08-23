package app.indelible.library.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Link
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.SheetValue
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.core.i18n.resolve
import app.indelible.profile.viewmodel.AddLibraryUiState
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleTextField
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_close
import indelible.composeapp.generated.resources.library_add_url_body
import indelible.composeapp.generated.resources.library_add_url_hint
import indelible.composeapp.generated.resources.library_add_url_label
import indelible.composeapp.generated.resources.library_add_url_submit
import indelible.composeapp.generated.resources.library_add_url_title
import indelible.composeapp.generated.resources.library_add_url_waiting
import org.jetbrains.compose.resources.stringResource

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AddUrlBottomSheet(
    uiState: AddLibraryUiState,
    onSubmit: (String) -> Unit,
    onInputChanged: () -> Unit,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
) {
    var url by rememberSaveable { mutableStateOf("") }
    val isSubmitting by rememberUpdatedState(uiState.isSubmitting)
    val sheetState =
        rememberModalBottomSheetState(
            skipPartiallyExpanded = true,
            confirmValueChange = { target ->
                target != SheetValue.Hidden || !isSubmitting
            },
        )

    ModalBottomSheet(
        onDismissRequest = {
            if (!uiState.isSubmitting) onDismiss()
        },
        sheetState = sheetState,
        sheetGesturesEnabled = !uiState.isSubmitting,
        modifier = modifier,
    ) {
        AddUrlSheetContent(
            url = url,
            uiState = uiState,
            onUrlChange = {
                url = it
                onInputChanged()
            },
            onSubmit = { onSubmit(url) },
            onDismiss = onDismiss,
        )
    }
}

@Composable
private fun AddUrlSheetContent(
    url: String,
    uiState: AddLibraryUiState,
    onUrlChange: (String) -> Unit,
    onSubmit: () -> Unit,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier =
            modifier
                .fillMaxWidth()
                .navigationBarsPadding()
                .imePadding()
                .padding(horizontal = IndelibleSpacing.screenPaddingH)
                .padding(bottom = IndelibleSpacing.screenPaddingV),
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
            verticalAlignment = Alignment.Top,
        ) {
            Surface(
                shape = IndelibleShape.md,
                color = MaterialTheme.colorScheme.primaryContainer,
                contentColor = MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(IndelibleSpacing.step48),
            ) {
                Box(contentAlignment = Alignment.Center) {
                    Icon(
                        imageVector = Icons.Filled.Link,
                        contentDescription = null,
                        modifier = Modifier.size(IndelibleSpacing.step24),
                    )
                }
            }

            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = stringResource(Res.string.library_add_url_title),
                    style = MaterialTheme.typography.titleLarge,
                )
                Spacer(modifier = Modifier.height(IndelibleSpacing.step4))
                Text(
                    text = stringResource(Res.string.library_add_url_body),
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            IconButton(
                onClick = onDismiss,
                enabled = !uiState.isSubmitting,
                modifier = Modifier.size(IndelibleSpacing.step48),
            ) {
                Icon(
                    imageVector = Icons.Filled.Close,
                    contentDescription = stringResource(Res.string.common_close),
                )
            }
        }

        Spacer(modifier = Modifier.height(IndelibleSpacing.step24))

        IndelibleTextField(
            value = url,
            onValueChange = onUrlChange,
            label = stringResource(Res.string.library_add_url_label),
            error = uiState.errorMessage?.resolve(),
            enabled = !uiState.isSubmitting,
            keyboardType = KeyboardType.Uri,
            imeAction = ImeAction.Done,
            onImeAction = {
                if (url.isNotBlank() && !uiState.isSubmitting) onSubmit()
            },
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step8))

        Text(
            text =
                if (uiState.isSubmitting) {
                    stringResource(Res.string.library_add_url_waiting)
                } else {
                    stringResource(Res.string.library_add_url_hint)
                },
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )

        Spacer(modifier = Modifier.height(IndelibleSpacing.step20))

        IndelibleButton(
            text = stringResource(Res.string.library_add_url_submit),
            onClick = onSubmit,
            isLoading = uiState.isSubmitting,
            enabled = url.isNotBlank(),
        )
    }
}

@Preview(showBackground = true)
@Composable
private fun AddUrlSheetContentPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            AddUrlSheetContent(
                url = "https://example.com/article",
                uiState = AddLibraryUiState(),
                onUrlChange = {},
                onSubmit = {},
                onDismiss = {},
                modifier = Modifier.padding(top = IndelibleSpacing.step24),
            )
        }
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun AddUrlSheetContentPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            AddUrlSheetContent(
                url = "https://example.com/article",
                uiState = AddLibraryUiState(isSubmitting = true),
                onUrlChange = {},
                onSubmit = {},
                onDismiss = {},
                modifier = Modifier.padding(top = IndelibleSpacing.step24),
            )
        }
    }
}
