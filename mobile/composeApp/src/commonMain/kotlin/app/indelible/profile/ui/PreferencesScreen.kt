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
import app.indelible.core.preferences.DefaultViewPreference
import app.indelible.core.preferences.ReaderFontFamilyPreference
import app.indelible.core.preferences.ReaderFontSizePreference
import app.indelible.core.preferences.ReaderLineHeightPreference
import app.indelible.core.preferences.ThemePreference
import app.indelible.core.preferences.TriageModePreference
import app.indelible.profile.ui.components.PreferenceDropdownRow
import app.indelible.profile.ui.components.SettingsSection
import app.indelible.profile.ui.components.ToggleRow
import app.indelible.profile.viewmodel.UserPreferencesViewModel
import app.indelible.ui.theme.AccentBlue
import app.indelible.ui.theme.AccentGreen
import app.indelible.ui.theme.AccentOrange
import app.indelible.ui.theme.AccentPink
import app.indelible.ui.theme.IndelibleSpacing

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
                        text = "Preferences",
                        style = MaterialTheme.typography.headlineSmall,
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = "Back",
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
            // ── Appearance ────────────────────────────────────────────────
            SettingsSection(title = "Appearance") {
                PreferenceDropdownRow(
                    label = "Theme",
                    sublabel = "Light, Dark, or System",
                    currentValue = theme,
                    displayName = { it.displayName },
                    options = ThemePreference.entries,
                    onSelected = { viewModel.setTheme(it) },
                )
                AccentColorRow(
                    swatches = accentSwatches,
                    selected = selectedAccent,
                    onSelected = { selectedAccent = it },
                )
            }

            // ── Layout & Navigation ───────────────────────────────────────
            SettingsSection(title = "Layout & Navigation") {
                PreferenceDropdownRow(
                    label = "Default View",
                    sublabel = "Screen shown on launch",
                    currentValue = defaultView,
                    displayName = { it.displayName },
                    options = DefaultViewPreference.entries,
                    onSelected = { viewModel.setDefaultView(it) },
                )
            }

            // ── Triage & Workflow ─────────────────────────────────────────
            SettingsSection(title = "Triage & Workflow") {
                PreferenceDropdownRow(
                    label = "Triage Mode",
                    sublabel = "Manual or Focus-assisted triage",
                    currentValue = triageMode,
                    displayName = { it.displayName },
                    options = TriageModePreference.entries,
                    onSelected = { viewModel.setTriageMode(it) },
                )
                ToggleRow(
                    label = "Auto-Advance",
                    sublabel = "Move to next item after triage action",
                    checked = autoAdvance,
                    onCheckedChange = { viewModel.setAutoAdvance(it) },
                )
            }

            // ── Reader ────────────────────────────────────────────────────
            SettingsSection(title = "Reader") {
                PreferenceDropdownRow(
                    label = "Font",
                    sublabel = fontFamily.description,
                    currentValue = fontFamily,
                    displayName = { it.displayName },
                    options = ReaderFontFamilyPreference.entries,
                    onSelected = { viewModel.setFontFamily(it) },
                )
                PreferenceDropdownRow(
                    label = "Font Size",
                    currentValue = fontSize,
                    displayName = { it.displayName },
                    options = ReaderFontSizePreference.entries,
                    onSelected = { viewModel.setFontSize(it) },
                )
                PreferenceDropdownRow(
                    label = "Line Height",
                    currentValue = lineHeight,
                    displayName = { it.displayName },
                    options = ReaderLineHeightPreference.entries,
                    onSelected = { viewModel.setLineHeight(it) },
                )
            }

            Spacer(modifier = Modifier.height(IndelibleSpacing.step32))
        }
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
            text = "Accent Color",
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
