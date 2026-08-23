package app.indelible.profile.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp
import coil3.compose.AsyncImagePainter
import coil3.compose.SubcomposeAsyncImage
import coil3.compose.SubcomposeAsyncImageContent

@Composable
fun UserAvatar(
    displayName: String,
    size: Dp,
    textStyle: TextStyle,
    modifier: Modifier = Modifier,
    shape: Shape = CircleShape,
    avatarUrl: String? = null,
    avatarBytes: ByteArray? = null,
) {
    // avatarBytes (auth-fetched) takes priority; fall back to direct URL (presigned / external)
    val imageModel: Any? = avatarBytes ?: avatarUrl

    Box(
        modifier =
            modifier
                .size(size)
                .clip(shape)
                .background(MaterialTheme.colorScheme.primaryContainer),
        contentAlignment = Alignment.Center,
    ) {
        // Initial letter is always the base layer — shows on load, error, or no image
        Text(
            text = displayName.firstOrNull()?.uppercase() ?: "?", // i18n-ignore: user-provided name initial
            style = textStyle,
            color = MaterialTheme.colorScheme.onPrimaryContainer,
            textAlign = TextAlign.Center,
        )

        // Image overlays only on a successful load, transparent otherwise
        if (imageModel != null) {
            SubcomposeAsyncImage(
                model = imageModel,
                contentDescription = null,
                contentScale = ContentScale.Crop,
                modifier = Modifier.matchParentSize(),
            ) {
                val state by painter.state.collectAsState()
                if (state is AsyncImagePainter.State.Success) {
                    SubcomposeAsyncImageContent()
                }
            }
        }
    }
}
