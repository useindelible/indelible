package app.indelible.reader.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Slider
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.reader.model.ReaderBackground
import app.indelible.reader.model.ReaderPreferences
import app.indelible.reader.model.Typeface
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import kotlin.math.abs
import kotlin.math.roundToInt

private const val DEFAULT_LINE_HEIGHT = 1.72f

// Line-spacing option menu: the values are the user-facing choices.
@Suppress("MagicNumber")
private val lineSpacingOptions = listOf(1.5f to "Tight", 1.72f to "Normal", 2.0f to "Loose")

/**
 * Display panel: live reader typography and canvas. Every change is pushed back
 * through [onPreferencesChanged] and re-emitted as WebView CSS without a reload.
 *
 * There is no artwork control: the drawing behind the masthead is chosen from the
 * document id, so an article keeps the same one every time it is opened.
 */
@Composable
fun DisplaySettingsPanel(
    preferences: ReaderPreferences,
    onPreferencesChanged: (ReaderPreferences) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier.fillMaxWidth()) {
        TextSizeSection(preferences = preferences, onPreferencesChanged = onPreferencesChanged)
        Spacer(modifier = Modifier.height(IndelibleSpacing.sectionGap))
        BackgroundSection(preferences = preferences, onPreferencesChanged = onPreferencesChanged)
        Spacer(modifier = Modifier.height(IndelibleSpacing.sectionGap))
        TypefaceSection(preferences = preferences, onPreferencesChanged = onPreferencesChanged)
        Spacer(modifier = Modifier.height(IndelibleSpacing.sectionGap))
        LineSpacingSection(preferences = preferences, onPreferencesChanged = onPreferencesChanged)
    }
}

@Composable
private fun TextSizeSection(
    preferences: ReaderPreferences,
    onPreferencesChanged: (ReaderPreferences) -> Unit,
) {
    PanelSectionLabel("Text size")
    Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step12),
    ) {
        Text(
            text = "A",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Slider(
            value = preferences.fontSize.toFloat(),
            onValueChange = { onPreferencesChanged(preferences.copy(fontSize = it.roundToInt())) },
            valueRange = 15f..24f,
            steps = 8,
            modifier = Modifier.weight(1f),
        )
        Text(
            text = "A",
            style = MaterialTheme.typography.titleLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun BackgroundSection(
    preferences: ReaderPreferences,
    onPreferencesChanged: (ReaderPreferences) -> Unit,
) {
    PanelSectionLabel("Background")
    Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
    val swatches = IndelibleTheme.colors.readerBackgroundSwatches
    Row(horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step16)) {
        ReaderBackground.entries.forEachIndexed { index, background ->
            ColorChoice(
                color = swatches[index],
                selected = preferences.background == background,
                contentDescription = background.name,
                onClick = { onPreferencesChanged(preferences.copy(background = background)) },
            )
        }
    }
}

@Composable
private fun TypefaceSection(
    preferences: ReaderPreferences,
    onPreferencesChanged: (ReaderPreferences) -> Unit,
) {
    PanelSectionLabel("Typeface")
    Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
    val typefaces = Typeface.entries.toTypedArray()
    val typefaceLabels = arrayOf("Serif", "Sans", "Mono")
    SingleChoiceSegmentedButtonRow(modifier = Modifier.fillMaxWidth()) {
        typefaces.forEachIndexed { index, typeface ->
            SegmentedButton(
                selected = preferences.typeface == typeface,
                onClick = { onPreferencesChanged(preferences.copy(typeface = typeface)) },
                shape = SegmentedButtonDefaults.itemShape(index, typefaces.size),
            ) {
                Text(text = typefaceLabels[index], style = MaterialTheme.typography.bodySmall)
            }
        }
    }
}

@Composable
private fun LineSpacingSection(
    preferences: ReaderPreferences,
    onPreferencesChanged: (ReaderPreferences) -> Unit,
) {
    PanelSectionLabel("Line spacing")
    Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
    val nearestSpacing =
        lineSpacingOptions.minByOrNull { abs(it.first - preferences.lineHeight) }?.first
            ?: DEFAULT_LINE_HEIGHT
    SingleChoiceSegmentedButtonRow(modifier = Modifier.fillMaxWidth()) {
        lineSpacingOptions.forEachIndexed { index, option ->
            val (value, label) = option
            SegmentedButton(
                selected = value == nearestSpacing,
                onClick = { onPreferencesChanged(preferences.copy(lineHeight = value)) },
                shape = SegmentedButtonDefaults.itemShape(index, lineSpacingOptions.size),
            ) {
                Text(text = label, style = MaterialTheme.typography.bodySmall)
            }
        }
    }
}

@Composable
private fun PanelSectionLabel(text: String) {
    Text(
        text = text,
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
    )
}

@Composable
private fun ColorChoice(
    color: Color,
    selected: Boolean,
    contentDescription: String,
    onClick: () -> Unit,
) {
    val ring =
        if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outlineVariant
    Box(
        modifier =
            Modifier
                .size(IndelibleSpacing.step40)
                .clip(CircleShape)
                .background(color)
                .border(IndelibleSpacing.step2, ring, CircleShape)
                .clickable(onClickLabel = contentDescription, onClick = onClick),
    )
}

@Preview
@Composable
private fun DisplaySettingsPanelPreviewLight() {
    AppTheme(darkTheme = false) {
        Surface {
            DisplaySettingsPanel(
                preferences = ReaderPreferences(),
                onPreferencesChanged = {},
                modifier = Modifier.padding(IndelibleSpacing.screenPaddingH),
            )
        }
    }
}

@Preview
@Composable
private fun DisplaySettingsPanelPreviewDark() {
    AppTheme(darkTheme = true) {
        Surface {
            DisplaySettingsPanel(
                preferences = ReaderPreferences(background = ReaderBackground.SLATE),
                onPreferencesChanged = {},
                modifier = Modifier.padding(IndelibleSpacing.screenPaddingH),
            )
        }
    }
}
