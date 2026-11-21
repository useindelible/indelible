package app.indelible.profile.ui

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_cancel
import indelible.composeapp.generated.resources.profile_sign_out
import indelible.composeapp.generated.resources.profile_sign_out_confirm
import indelible.composeapp.generated.resources.profile_sign_out_waiting
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource

/**
 * Signing out purges the device's queue, so [waitingChanges] above zero turns the question into a warning,
 * and a count not yet known (null) holds the confirmation until it is.
 */
@Composable
fun SignOutDialog(
    waitingChanges: Int?,
    onConfirm: () -> Unit,
    onDismiss: () -> Unit,
) {
    val body =
        if (waitingChanges != null && waitingChanges > 0) {
            pluralStringResource(Res.plurals.profile_sign_out_waiting, waitingChanges, waitingChanges)
        } else {
            stringResource(Res.string.profile_sign_out_confirm)
        }
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(stringResource(Res.string.profile_sign_out)) },
        text = { Text(body) },
        confirmButton = {
            TextButton(onClick = onConfirm, enabled = waitingChanges != null) {
                Text(stringResource(Res.string.profile_sign_out))
            }
        },
        dismissButton = { TextButton(onClick = onDismiss) { Text(stringResource(Res.string.common_cancel)) } },
    )
}
