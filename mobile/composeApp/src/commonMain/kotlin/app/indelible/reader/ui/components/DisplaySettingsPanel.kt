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
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.reader_background
import indelible.composeapp.generated.resources.reader_background_black
import indelible.composeapp.generated.resources.reader_background_paper
import indelible.composeapp.generated.resources.reader_background_sepia
import indelible.composeapp.generated.resources.reader_background_slate
import indelible.composeapp.generated.resources.reader_line_spacing
import indelible.composeapp.generated.resources.reader_line_spacing_loose
import indelible.composeapp.generated.resources.reader_line_spacing_normal
import indelible.composeapp.generated.resources.reader_line_spacing_tight
import indelible.composeapp.generated.resources.reader_text_size
import indelible.composeapp.generated.resources.reader_typeface
import indelible.composeapp.generated.resources.reader_typeface_mono
import indelible.composeapp.generated.resources.reader_typeface_sans
import indelible.composeapp.generated.resources.reader_typeface_serif
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource
import kotlin.math.abs
import kotlin.math.roundToInt

private const val DEFAULT_LINE_HEIGHT = 1.72f

// Line-spacing option menu: the values are the user-facing choices.
@Suppress("MagicNumber")
private val lineSpacingOptions = listOf(1.5f, 1.72f, 2.0f)

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
    PanelSectionLabel(stringResource(Res.string.reader_text_size))
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
    PanelSectionLabel(stringResource(Res.string.reader_background))
    Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
    val swatches = IndelibleTheme.colors.readerBackgroundSwatches
    Row(horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step16)) {
        ReaderBackground.entries.forEachIndexed { index, background ->
            ColorChoice(
                color = swatches[index],
                selected = preferences.background == background,
                contentDescription = stringResource(backgroundLabelRes(background)),
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
    PanelSectionLabel(stringResource(Res.string.reader_typeface))
    Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
    val typefaces = Typeface.entries.toTypedArray()
    SingleChoiceSegmentedButtonRow(modifier = Modifier.fillMaxWidth()) {
        typefaces.forEachIndexed { index, typeface ->
            SegmentedButton(
                selected = preferences.typeface == typeface,
                onClick = { onPreferencesChanged(preferences.copy(typeface = typeface)) },
                shape = SegmentedButtonDefaults.itemShape(index, typefaces.size),
            ) {
                Text(text = stringResource(typefaceLabelRes(typeface)), style = MaterialTheme.typography.bodySmall)
            }
        }
    }
}

@Composable
private fun LineSpacingSection(
    preferences: ReaderPreferences,
    onPreferencesChanged: (ReaderPreferences) -> Unit,
) {
    PanelSectionLabel(stringResource(Res.string.reader_line_spacing))
    Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
    val nearestSpacing =
        lineSpacingOptions.minByOrNull { abs(it - preferences.lineHeight) }
            ?: DEFAULT_LINE_HEIGHT
    SingleChoiceSegmentedButtonRow(modifier = Modifier.fillMaxWidth()) {
        lineSpacingOptions.forEachIndexed { index, value ->
            SegmentedButton(
                selected = value == nearestSpacing,
                onClick = { onPreferencesChanged(preferences.copy(lineHeight = value)) },
                shape = SegmentedButtonDefaults.itemShape(index, lineSpacingOptions.size),
            ) {
                Text(text = stringResource(lineSpacingLabelRes(value)), style = MaterialTheme.typography.bodySmall)
            }
        }
    }
}

private fun backgroundLabelRes(background: ReaderBackground): StringResource =
    when (background) {
        ReaderBackground.PAPER -> Res.string.reader_background_paper
        ReaderBackground.SEPIA -> Res.string.reader_background_sepia
        ReaderBackground.SLATE -> Res.string.reader_background_slate
        ReaderBackground.BLACK -> Res.string.reader_background_black
    }

private fun typefaceLabelRes(typeface: Typeface): StringResource =
    when (typeface) {
        Typeface.SERIF -> Res.string.reader_typeface_serif
        Typeface.SANS -> Res.string.reader_typeface_sans
        Typeface.MONO -> Res.string.reader_typeface_mono
    }

private fun lineSpacingLabelRes(value: Float): StringResource =
    when (value) {
        1.5f -> Res.string.reader_line_spacing_tight
        2.0f -> Res.string.reader_line_spacing_loose
        else -> Res.string.reader_line_spacing_normal
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
