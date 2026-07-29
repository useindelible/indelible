use chrono::{DateTime, Utc};
use ind_domain::{
    AccentColor, AiPreferenceSettings, AppearanceSettings, ArchivalProcessingSettings,
    ArchivalSettings, ArchiveFormatPreferences, DefaultView, DuplicateAction,
    DuplicateDetectionSettings, DuplicateSensitivity, LayoutSettings, ListDensity,
    NotificationPreferences, PreferencesSettings, ProxySettings, ReaderFontFamily, ReaderFontSize,
    ReaderLineHeight, ReaderOpenMode, ReaderSettings, SidePanelMode, SidebarMode, Theme,
    TriageMode, WorkflowSettings,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ThemeDto {
    Light,
    Dark,
    System,
}

impl From<Theme> for ThemeDto {
    fn from(value: Theme) -> Self {
        match value {
            Theme::Light => Self::Light,
            Theme::Dark => Self::Dark,
            Theme::System => Self::System,
        }
    }
}

impl From<ThemeDto> for Theme {
    fn from(value: ThemeDto) -> Self {
        match value {
            ThemeDto::Light => Self::Light,
            ThemeDto::Dark => Self::Dark,
            ThemeDto::System => Self::System,
        }
    }
}

macro_rules! dto_enum {
    ($name:ident => $domain:ty { $($variant:ident),+ $(,)? }) => {
        #[derive(Debug, Serialize, Deserialize, ToSchema)]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($variant),+ }

        impl From<$domain> for $name {
            fn from(value: $domain) -> Self {
                match value {
                    $(<$domain>::$variant => Self::$variant,)+
                }
            }
        }

        impl From<$name> for $domain {
            fn from(value: $name) -> Self {
                match value {
                    $( $name::$variant => <$domain>::$variant, )+
                }
            }
        }
    };
}

dto_enum!(AccentColorDto => AccentColor { Blue, Green, Orange, Rose });
dto_enum!(SidebarModeDto => SidebarMode { Expanded, Collapsed, Auto });
dto_enum!(DefaultViewDto => DefaultView { Library, Feed, Search });
dto_enum!(ListDensityDto => ListDensity { Comfortable, Compact });
dto_enum!(SidePanelModeDto => SidePanelMode { Auto, Open, Closed });
dto_enum!(TriageModeDto => TriageMode { Manual, Focus });
dto_enum!(ReaderFontFamilyDto => ReaderFontFamily { Serif, Sans, Mono });
dto_enum!(ReaderFontSizeDto => ReaderFontSize { Small, Medium, Large });
dto_enum!(ReaderLineHeightDto => ReaderLineHeight { Compact, Relaxed });
dto_enum!(ReaderOpenModeDto => ReaderOpenMode { Reader, Original });
dto_enum!(DuplicateSensitivityDto => DuplicateSensitivity { Low, Medium, High });
dto_enum!(DuplicateActionDto => DuplicateAction { NotifyMe, SkipSilently, MergeWithExisting });

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AppearanceSettingsDto {
    pub accent_color: AccentColorDto,
}

impl From<AppearanceSettings> for AppearanceSettingsDto {
    fn from(value: AppearanceSettings) -> Self {
        Self {
            accent_color: value.accent_color.into(),
        }
    }
}

impl From<AppearanceSettingsDto> for AppearanceSettings {
    fn from(value: AppearanceSettingsDto) -> Self {
        Self {
            accent_color: value.accent_color.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LayoutSettingsDto {
    pub sidebar_mode: SidebarModeDto,
    pub default_view: DefaultViewDto,
    pub list_density: ListDensityDto,
    pub side_panel: SidePanelModeDto,
}

impl From<LayoutSettings> for LayoutSettingsDto {
    fn from(value: LayoutSettings) -> Self {
        Self {
            sidebar_mode: value.sidebar_mode.into(),
            default_view: value.default_view.into(),
            list_density: value.list_density.into(),
            side_panel: value.side_panel.into(),
        }
    }
}

impl From<LayoutSettingsDto> for LayoutSettings {
    fn from(value: LayoutSettingsDto) -> Self {
        Self {
            sidebar_mode: value.sidebar_mode.into(),
            default_view: value.default_view.into(),
            list_density: value.list_density.into(),
            side_panel: value.side_panel.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WorkflowSettingsDto {
    pub triage_mode: TriageModeDto,
    pub auto_advance: bool,
}

impl From<WorkflowSettings> for WorkflowSettingsDto {
    fn from(value: WorkflowSettings) -> Self {
        Self {
            triage_mode: value.triage_mode.into(),
            auto_advance: value.auto_advance,
        }
    }
}

impl From<WorkflowSettingsDto> for WorkflowSettings {
    fn from(value: WorkflowSettingsDto) -> Self {
        Self {
            triage_mode: value.triage_mode.into(),
            auto_advance: value.auto_advance,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ReaderSettingsDto {
    pub font_family: ReaderFontFamilyDto,
    pub font_size: ReaderFontSizeDto,
    pub line_height: ReaderLineHeightDto,
    pub email_open_mode: ReaderOpenModeDto,
}

impl From<ReaderSettings> for ReaderSettingsDto {
    fn from(value: ReaderSettings) -> Self {
        Self {
            font_family: value.font_family.into(),
            font_size: value.font_size.into(),
            line_height: value.line_height.into(),
            email_open_mode: value.email_open_mode.into(),
        }
    }
}

impl From<ReaderSettingsDto> for ReaderSettings {
    fn from(value: ReaderSettingsDto) -> Self {
        Self {
            font_family: value.font_family.into(),
            font_size: value.font_size.into(),
            line_height: value.line_height.into(),
            email_open_mode: value.email_open_mode.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AiPreferenceSettingsDto {
    pub mila_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = true)]
    pub custom_prompt: Option<String>,
}

impl From<AiPreferenceSettings> for AiPreferenceSettingsDto {
    fn from(value: AiPreferenceSettings) -> Self {
        Self {
            mila_enabled: value.mila_enabled,
            custom_prompt: value.custom_prompt,
        }
    }
}

impl From<AiPreferenceSettingsDto> for AiPreferenceSettings {
    fn from(value: AiPreferenceSettingsDto) -> Self {
        Self {
            mila_enabled: value.mila_enabled,
            custom_prompt: value.custom_prompt,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PreferencesSettingsResponse {
    pub theme: ThemeDto,
    pub appearance: AppearanceSettingsDto,
    pub layout: LayoutSettingsDto,
    pub workflow: WorkflowSettingsDto,
    pub reader: ReaderSettingsDto,
    pub ai: AiPreferenceSettingsDto,
}

pub type UpdatePreferencesRequest = PreferencesSettingsResponse;

impl From<ind_application::PreferencesSection> for PreferencesSettingsResponse {
    fn from(value: ind_application::PreferencesSection) -> Self {
        let settings = value.settings;
        Self {
            theme: value.theme.into(),
            appearance: settings.appearance.into(),
            layout: settings.layout.into(),
            workflow: settings.workflow.into(),
            reader: settings.reader.into(),
            ai: settings.ai.into(),
        }
    }
}

impl UpdatePreferencesRequest {
    pub fn into_domain(self) -> ind_application::PreferencesSection {
        ind_application::PreferencesSection {
            theme: self.theme.into(),
            settings: PreferencesSettings {
                appearance: self.appearance.into(),
                layout: self.layout.into(),
                workflow: self.workflow.into(),
                reader: self.reader.into(),
                ai: self.ai.into(),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NotificationsSettingsResponse {
    pub daily_review_reminder_enabled: bool,
    pub daily_review_reminder_time: String,
    pub weekly_digest_enabled: bool,
    pub new_highlights_sync: bool,
    pub feed_updates: bool,
    pub marketing_emails: bool,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

pub type UpdateNotificationsRequest = NotificationsSettingsResponse;

impl From<NotificationPreferences> for NotificationsSettingsResponse {
    fn from(value: NotificationPreferences) -> Self {
        Self {
            daily_review_reminder_enabled: value.daily_review_reminder_enabled,
            daily_review_reminder_time: value.daily_review_reminder_time,
            weekly_digest_enabled: value.weekly_digest_enabled,
            new_highlights_sync: value.new_highlights_sync,
            feed_updates: value.feed_updates,
            marketing_emails: value.marketing_emails,
            updated_at: value.updated_at,
        }
    }
}

impl UpdateNotificationsRequest {
    pub fn into_domain(self, user_id: ind_domain::UserId) -> NotificationPreferences {
        NotificationPreferences {
            user_id,
            daily_review_reminder_enabled: self.daily_review_reminder_enabled,
            daily_review_reminder_time: self.daily_review_reminder_time,
            weekly_digest_enabled: self.weekly_digest_enabled,
            new_highlights_sync: self.new_highlights_sync,
            feed_updates: self.feed_updates,
            marketing_emails: self.marketing_emails,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ArchiveFormatPreferencesDto {
    pub readable_html: bool,
    pub monolith: bool,
    pub pdf: bool,
    pub screenshot: bool,
    pub warc: bool,
}

impl From<ArchiveFormatPreferences> for ArchiveFormatPreferencesDto {
    fn from(value: ArchiveFormatPreferences) -> Self {
        Self {
            readable_html: value.readable_html,
            monolith: value.monolith,
            pdf: value.pdf,
            screenshot: value.screenshot,
            warc: value.warc,
        }
    }
}

impl From<ArchiveFormatPreferencesDto> for ArchiveFormatPreferences {
    fn from(value: ArchiveFormatPreferencesDto) -> Self {
        Self {
            readable_html: value.readable_html,
            monolith: value.monolith,
            pdf: value.pdf,
            screenshot: value.screenshot,
            warc: value.warc,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DuplicateDetectionSettingsDto {
    pub enabled: bool,
    pub sensitivity: DuplicateSensitivityDto,
    pub on_duplicate: DuplicateActionDto,
}

impl From<DuplicateDetectionSettings> for DuplicateDetectionSettingsDto {
    fn from(value: DuplicateDetectionSettings) -> Self {
        Self {
            enabled: value.enabled,
            sensitivity: value.sensitivity.into(),
            on_duplicate: value.on_duplicate.into(),
        }
    }
}

impl From<DuplicateDetectionSettingsDto> for DuplicateDetectionSettings {
    fn from(value: DuplicateDetectionSettingsDto) -> Self {
        Self {
            enabled: value.enabled,
            sensitivity: value.sensitivity.into(),
            on_duplicate: value.on_duplicate.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ArchivalProcessingSettingsDto {
    pub browser_timeout_secs: u32,
    pub max_concurrent_archives: u32,
    pub ai_auto_processing: bool,
}

impl From<ArchivalProcessingSettings> for ArchivalProcessingSettingsDto {
    fn from(value: ArchivalProcessingSettings) -> Self {
        Self {
            browser_timeout_secs: value.browser_timeout_secs,
            max_concurrent_archives: value.max_concurrent_archives,
            ai_auto_processing: value.ai_auto_processing,
        }
    }
}

impl From<ArchivalProcessingSettingsDto> for ArchivalProcessingSettings {
    fn from(value: ArchivalProcessingSettingsDto) -> Self {
        Self {
            browser_timeout_secs: value.browser_timeout_secs,
            max_concurrent_archives: value.max_concurrent_archives,
            ai_auto_processing: value.ai_auto_processing,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProxySettingsDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = true)]
    pub url: Option<String>,
    pub all_requests: bool,
}

impl From<ProxySettings> for ProxySettingsDto {
    fn from(value: ProxySettings) -> Self {
        Self {
            url: value.url,
            all_requests: value.all_requests,
        }
    }
}

impl From<ProxySettingsDto> for ProxySettings {
    fn from(value: ProxySettingsDto) -> Self {
        Self {
            url: value.url,
            all_requests: value.all_requests,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ArchivalSettingsResponse {
    pub archive_formats: ArchiveFormatPreferencesDto,
    pub duplicate_detection: DuplicateDetectionSettingsDto,
    pub processing: ArchivalProcessingSettingsDto,
    pub proxy: ProxySettingsDto,
}

pub type UpdateArchivalRequest = ArchivalSettingsResponse;

impl From<ArchivalSettings> for ArchivalSettingsResponse {
    fn from(value: ArchivalSettings) -> Self {
        Self {
            archive_formats: value.archive_formats.into(),
            duplicate_detection: value.duplicate_detection.into(),
            processing: value.processing.into(),
            proxy: value.proxy.into(),
        }
    }
}

impl From<UpdateArchivalRequest> for ArchivalSettings {
    fn from(value: UpdateArchivalRequest) -> Self {
        Self {
            archive_formats: value.archive_formats.into(),
            duplicate_detection: value.duplicate_detection.into(),
            processing: value.processing.into(),
            proxy: value.proxy.into(),
        }
    }
}
