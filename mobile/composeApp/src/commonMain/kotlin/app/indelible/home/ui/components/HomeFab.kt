package app.indelible.home.ui.components

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

/**
 * The home save action: a rounded-square accent button that opens the save flow.
 * Squircle (xxl) rather than fully circular to echo the reimagined surfaces.
 */
@Composable
fun HomeFab(
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
            Icon(imageVector = Icons.Filled.Add, contentDescription = "Save new item")
        }
    }
}

@Preview
@Composable
private fun HomeFabPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            HomeFab(onClick = {}, modifier = Modifier.padding(IndelibleSpacing.step16))
        }
    }
}

@Preview
@Composable
private fun HomeFabPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            HomeFab(onClick = {}, modifier = Modifier.padding(IndelibleSpacing.step16))
        }
    }
}
