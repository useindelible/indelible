package app.indelible.reader.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.Dp
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing

/**
 * Shared search/create field used by [TagEditor] and [HighlightTagSheetContent].
 * [containerShape] and [iconSize] differ between the two call sites and are exposed
 * as parameters; defaults match the [TagEditor] usage.
 * [focusRequester] is optional — pass one when the field should auto-focus on appear.
 * [onDone] is called with the current query when the keyboard Done action fires.
 */
@Composable
internal fun TagSearchField(
    query: String,
    onQueryChange: (String) -> Unit,
    onDone: (String) -> Unit,
    modifier: Modifier = Modifier,
    containerShape: Shape = IndelibleShape.sm,
    iconSize: Dp = IndelibleSpacing.step20,
    focusRequester: FocusRequester? = null,
) {
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .clip(containerShape)
                .background(MaterialTheme.colorScheme.surfaceVariant)
                .padding(horizontal = IndelibleSpacing.step12, vertical = IndelibleSpacing.step10),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
    ) {
        Icon(
            Icons.Filled.Search,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(iconSize),
        )
        BasicTextField(
            value = query,
            onValueChange = onQueryChange,
            singleLine = true,
            textStyle =
                MaterialTheme.typography.bodyMedium.copy(
                    color = MaterialTheme.colorScheme.onSurface,
                ),
            cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
            keyboardActions =
                KeyboardActions(
                    onDone = {
                        val trimmed = query.trim()
                        if (trimmed.isNotEmpty()) onDone(trimmed)
                    },
                ),
            modifier =
                Modifier
                    .weight(1f)
                    .then(if (focusRequester != null) Modifier.focusRequester(focusRequester) else Modifier),
            decorationBox = { innerTextField ->
                if (query.isEmpty()) {
                    Text(
                        text = "Search or create tag…",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                innerTextField()
            },
        )
    }
}
