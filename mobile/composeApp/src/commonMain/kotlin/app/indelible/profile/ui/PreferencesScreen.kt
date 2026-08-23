package app.indelible.profile.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import app.indelible.core.i18n.AppLanguage
import app.indelible.core.i18n.AppLanguageSettings
import app.indelible.core.preferences.DefaultViewPreference
import app.indelible.core.preferences.ReaderFontFamilyPreference
import app.indelible.core.preferences.ReaderFontSizePreference
import app.indelible.core.preferences.ReaderLineHeightPreference
import app.indelible.core.preferences.ThemePreference
import app.indelible.core.preferences.TriageModePreference
import app.indelible.profile.ui.components.PreferenceDropdownRow
import app.indelible.profile.ui.components.SettingsRow
import app.indelible.profile.ui.components.SettingsSection
import app.indelible.profile.ui.components.ToggleRow
import app.indelible.profile.viewmodel.UserPreferencesViewModel
import app.indelible.ui.theme.AccentBlue
import app.indelible.ui.theme.AccentGreen
import app.indelible.ui.theme.AccentOrange
import app.indelible.ui.theme.AccentPink
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_back
import indelible.composeapp.generated.resources.prefs_accent_color
import indelible.composeapp.generated.resources.prefs_appearance
import indelible.composeapp.generated.resources.prefs_auto_advance
import indelible.composeapp.generated.resources.prefs_auto_advance_description
import indelible.composeapp.generated.resources.prefs_default_view
import indelible.composeapp.generated.resources.prefs_default_view_description
import indelible.composeapp.generated.resources.prefs_font
import indelible.composeapp.generated.resources.prefs_font_size
import indelible.composeapp.generated.resources.prefs_language
import indelible.composeapp.generated.resources.prefs_language_description
import indelible.composeapp.generated.resources.prefs_layout_navigation
import indelible.composeapp.generated.resources.prefs_line_height
import indelible.composeapp.generated.resources.prefs_reader
import indelible.composeapp.generated.resources.prefs_theme
import indelible.composeapp.generated.resources.prefs_theme_description
import indelible.composeapp.generated.resources.prefs_title
import indelible.composeapp.generated.resources.prefs_triage_mode
import indelible.composeapp.generated.resources.prefs_triage_mode_description
import indelible.composeapp.generated.resources.prefs_triage_workflow
import org.jetbrains.compose.resources.stringResource

private val accentSwatches =
    listOf(
        AccentBlue,
        AccentPink,
        AccentGreen,
        AccentOrange,
    )

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PreferencesScreen(
    viewModel: UserPreferencesViewModel,
    appLanguageSettings: AppLanguageSettings?,
    onNavigateBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val theme by viewModel.theme.collectAsState()
    val defaultView by viewModel.defaultView.collectAsState()
    val fontFamily by viewModel.fontFamily.collectAsState()
    val fontSize by viewModel.fontSize.collectAsState()
    val lineHeight by viewModel.lineHeight.collectAsState()
    val triageMode by viewModel.triageMode.collectAsState()
    val autoAdvance by viewModel.autoAdvance.collectAsState()

    var selectedAccent by remember { mutableStateOf(accentSwatches[0]) }

    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = stringResource(Res.string.prefs_title),
                        style = MaterialTheme.typography.headlineSmall,
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = stringResource(Res.string.common_back),
                        )
                    }
                },
            )
        },
    ) { paddingValues ->
        Column(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(top = paddingValues.calculateTopPadding())
                    .verticalScroll(rememberScrollState()),
        ) {
            SettingsSection(title = stringResource(Res.string.prefs_appearance)) {
                PreferenceDropdownRow(
                    label = stringResource(Res.string.prefs_theme),
                    sublabel = stringResource(Res.string.prefs_theme_description),
                    currentValue = theme,
                    displayName = { stringResource(it.labelRes) },
                    options = ThemePreference.entries,
                    onSelected = { viewModel.setTheme(it) },
                )
                if (appLanguageSettings != null) {
                    LanguageSettingsRow(appLanguageSettings)
                }
                AccentColorRow(
                    swatches = accentSwatches,
                    selected = selectedAccent,
                    onSelected = { selectedAccent = it },
                )
            }

            SettingsSection(title = stringResource(Res.string.prefs_layout_navigation)) {
                PreferenceDropdownRow(
                    label = stringResource(Res.string.prefs_default_view),
                    sublabel = stringResource(Res.string.prefs_default_view_description),
                    currentValue = defaultView,
                    displayName = { stringResource(it.labelRes) },
                    options = DefaultViewPreference.entries,
                    onSelected = { viewModel.setDefaultView(it) },
                )
            }

            SettingsSection(title = stringResource(Res.string.prefs_triage_workflow)) {
                PreferenceDropdownRow(
                    label = stringResource(Res.string.prefs_triage_mode),
                    sublabel = stringResource(Res.string.prefs_triage_mode_description),
                    currentValue = triageMode,
                    displayName = { stringResource(it.labelRes) },
                    options = TriageModePreference.entries,
                    onSelected = { viewModel.setTriageMode(it) },
                )
                ToggleRow(
                    label = stringResource(Res.string.prefs_auto_advance),
                    sublabel = stringResource(Res.string.prefs_auto_advance_description),
                    checked = autoAdvance,
                    onCheckedChange = { viewModel.setAutoAdvance(it) },
                )
            }

            SettingsSection(title = stringResource(Res.string.prefs_reader)) {
                PreferenceDropdownRow(
                    label = stringResource(Res.string.prefs_font),
                    sublabel = stringResource(fontFamily.descriptionRes),
                    currentValue = fontFamily,
                    displayName = { stringResource(it.labelRes) },
                    options = ReaderFontFamilyPreference.entries,
                    onSelected = { viewModel.setFontFamily(it) },
                )
                PreferenceDropdownRow(
                    label = stringResource(Res.string.prefs_font_size),
                    currentValue = fontSize,
                    displayName = { stringResource(it.labelRes) },
                    options = ReaderFontSizePreference.entries,
                    onSelected = { viewModel.setFontSize(it) },
                )
                PreferenceDropdownRow(
                    label = stringResource(Res.string.prefs_line_height),
                    currentValue = lineHeight,
                    displayName = { stringResource(it.labelRes) },
                    options = ReaderLineHeightPreference.entries,
                    onSelected = { viewModel.setLineHeight(it) },
                )
            }

            Spacer(modifier = Modifier.height(IndelibleSpacing.step32))
        }
    }
}

@Composable
internal fun LanguageSettingsRow(
    settings: AppLanguageSettings,
    modifier: Modifier = Modifier,
) {
    when (settings) {
        is AppLanguageSettings.Selectable ->
            PreferenceDropdownRow(
                label = stringResource(Res.string.prefs_language),
                sublabel = stringResource(Res.string.prefs_language_description),
                currentValue = settings.language,
                displayName = { stringResource(it.labelRes) },
                options = AppLanguage.entries,
                onSelected = settings.onSelected,
                modifier = modifier,
            )
        is AppLanguageSettings.SystemManaged ->
            SettingsRow(
                label = stringResource(Res.string.prefs_language),
                sublabel = stringResource(Res.string.prefs_language_description),
                value = stringResource(settings.language.labelRes),
                onClick = settings.onOpenSettings,
                modifier = modifier,
            )
    }
}

@Composable
private fun AccentColorRow(
    swatches: List<Color>,
    selected: Color,
    onSelected: (Color) -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .padding(horizontal = IndelibleSpacing.rowPaddingH, vertical = IndelibleSpacing.rowPaddingV),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = stringResource(Res.string.prefs_accent_color),
            style = MaterialTheme.typography.bodyMedium,
            modifier = Modifier.weight(1f),
        )
        Row(horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8)) {
            swatches.forEach { color ->
                val isSelected = color == selected
                Box(
                    modifier =
                        Modifier
                            .size(IndelibleSpacing.step24)
                            .clip(CircleShape)
                            .background(color)
                            .then(
                                if (isSelected) {
                                    Modifier.border(
                                        width = IndelibleSpacing.step2,
                                        color = MaterialTheme.colorScheme.onSurface,
                                        shape = CircleShape,
                                    )
                                } else {
                                    Modifier
                                },
                            ).clickable { onSelected(color) },
                )
            }
        }
    }
}
