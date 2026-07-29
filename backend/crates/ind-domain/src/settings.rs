use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccentColor {
    Blue,
    Green,
    Orange,
    Rose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SidebarMode {
    Expanded,
    Collapsed,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefaultView {
    Library,
    Feed,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListDensity {
    Comfortable,
    Compact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SidePanelMode {
    Auto,
    Open,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriageMode {
    Manual,
    Focus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReaderFontFamily {
    Serif,
    Sans,
    Mono,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReaderFontSize {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReaderLineHeight {
    Compact,
    Relaxed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReaderOpenMode {
    Reader,
    Original,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppearanceSettings {
    pub accent_color: AccentColor,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            accent_color: AccentColor::Blue,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutSettings {
    pub sidebar_mode: SidebarMode,
    pub default_view: DefaultView,
    pub list_density: ListDensity,
    pub side_panel: SidePanelMode,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            sidebar_mode: SidebarMode::Expanded,
            default_view: DefaultView::Library,
            list_density: ListDensity::Comfortable,
            side_panel: SidePanelMode::Auto,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowSettings {
    pub triage_mode: TriageMode,
    pub auto_advance: bool,
}

impl Default for WorkflowSettings {
    fn default() -> Self {
        Self {
            triage_mode: TriageMode::Focus,
            auto_advance: true,
        }
    }
}

#[cfg(test)]
mod workflow_tests {
    use super::{TriageMode, WorkflowSettings};

    #[test]
    fn defaults_to_focus_triage() {
        assert_eq!(WorkflowSettings::default().triage_mode, TriageMode::Focus);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReaderSettings {
    pub font_family: ReaderFontFamily,
    pub font_size: ReaderFontSize,
    pub line_height: ReaderLineHeight,
    pub email_open_mode: ReaderOpenMode,
}

impl Default for ReaderSettings {
    fn default() -> Self {
        Self {
            font_family: ReaderFontFamily::Serif,
            font_size: ReaderFontSize::Medium,
            line_height: ReaderLineHeight::Relaxed,
            email_open_mode: ReaderOpenMode::Reader,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiPreferenceSettings {
    pub mila_enabled: bool,
    pub custom_prompt: Option<String>,
}

impl Default for AiPreferenceSettings {
    fn default() -> Self {
        Self {
            mila_enabled: true,
            custom_prompt: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PreferencesSettings {
    pub appearance: AppearanceSettings,
    pub layout: LayoutSettings,
    pub workflow: WorkflowSettings,
    pub reader: ReaderSettings,
    pub ai: AiPreferenceSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveFormatPreferences {
    pub readable_html: bool,
    pub monolith: bool,
    pub pdf: bool,
    pub screenshot: bool,
    pub warc: bool,
}

impl Default for ArchiveFormatPreferences {
    fn default() -> Self {
        Self {
            readable_html: true,
            monolith: true,
            pdf: false,
            screenshot: true,
            warc: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DuplicateSensitivity {
    Low,
    Medium,
    High,
}

impl DuplicateSensitivity {
    pub fn simhash_threshold(self) -> u32 {
        match self {
            Self::Low => 4,
            Self::Medium => 6,
            Self::High => 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DuplicateAction {
    NotifyMe,
    SkipSilently,
    MergeWithExisting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuplicateDetectionSettings {
    pub enabled: bool,
    pub sensitivity: DuplicateSensitivity,
    pub on_duplicate: DuplicateAction,
}

impl Default for DuplicateDetectionSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            sensitivity: DuplicateSensitivity::Medium,
            on_duplicate: DuplicateAction::NotifyMe,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchivalProcessingSettings {
    pub browser_timeout_secs: u32,
    pub max_concurrent_archives: u32,
    pub ai_auto_processing: bool,
}

impl Default for ArchivalProcessingSettings {
    fn default() -> Self {
        Self {
            browser_timeout_secs: 90,
            max_concurrent_archives: 3,
            ai_auto_processing: false,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxySettings {
    pub url: Option<String>,
    pub all_requests: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ArchivalSettings {
    pub archive_formats: ArchiveFormatPreferences,
    pub duplicate_detection: DuplicateDetectionSettings,
    pub processing: ArchivalProcessingSettings,
    pub proxy: ProxySettings,
}
