package app.indelible.library.ui.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.library_add_url_cd
import org.jetbrains.compose.resources.stringResource

@Composable
fun LibraryFab(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        onClick = onClick,
        shape = IndelibleShape.xxl,
        color = MaterialTheme.colorScheme.primary,
        contentColor = MaterialTheme.colorScheme.onPrimary,
        shadowElevation = IndelibleSpacing.step6,
        modifier = modifier.size(IndelibleSpacing.step56),
    ) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Icon(
                imageVector = Icons.Filled.Add,
                contentDescription = stringResource(Res.string.library_add_url_cd),
            )
        }
    }
}

@Preview
@Composable
private fun LibraryFabPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            LibraryFab(onClick = {}, modifier = Modifier.padding(IndelibleSpacing.step16))
        }
    }
}

@Preview
@Composable
private fun LibraryFabPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            LibraryFab(onClick = {}, modifier = Modifier.padding(IndelibleSpacing.step16))
        }
    }
}
