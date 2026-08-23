import type {
	AccentColorDto,
	DefaultViewDto,
	ListDensityDto,
	PreferencesSettingsResponse,
	ReaderFontFamilyDto,
	ReaderFontSizeDto,
	ReaderLineHeightDto,
	ReaderOpenModeDto,
	SidePanelModeDto,
	SidebarModeDto,
	ThemeDto,
	TriageModeDto
} from '$lib/api';
import type { MessageKey, Translate } from '$lib/i18n';
import { matchLocale, systemLocale } from '$lib/i18n';

export interface ReadingAppearanceDraft {
	theme: ThemeDto;
	accentColor: AccentColorDto;
	sidebarMode: SidebarModeDto;
	defaultView: DefaultViewDto;
	listDensity: ListDensityDto;
	sidePanel: SidePanelModeDto;
	triageMode: TriageModeDto;
	autoAdvance: boolean;
	fontFamily: ReaderFontFamilyDto;
	fontSize: ReaderFontSizeDto;
	lineHeight: ReaderLineHeightDto;
	emailOpenMode: ReaderOpenModeDto;
	locale: string;
}

export interface ReaderPreviewStyles {
	fontFamily: string;
	fontSize: string;
	lineHeight: string;
}

export const ACCENT_SWATCHES: { value: AccentColorDto; labelKey: MessageKey }[] = [
	{ value: 'blue', labelKey: 'prefs_reading_color_blue' },
	{ value: 'green', labelKey: 'prefs_reading_color_green' },
	{ value: 'rose', labelKey: 'prefs_reading_color_rose' },
	{ value: 'orange', labelKey: 'prefs_reading_color_orange' }
];

export const FONT_SIZES = [
	'small',
	'medium',
	'large'
] as const satisfies readonly ReaderFontSizeDto[];

export const FONT_SIZE_LABEL: Record<ReaderFontSizeDto, MessageKey> = {
	small: 'prefs_reading_size_small',
	medium: 'prefs_reading_size_medium',
	large: 'prefs_reading_size_large'
};

export const SYSTEM_LOCALE_OPTION = '';

const LANGUAGE_LABEL_KEYS: Record<'en' | 'fr', MessageKey> = {
	en: 'prefs_reading_language_english',
	fr: 'prefs_reading_language_french'
};

export function localeOptions(
	translate: Translate,
	navigatorLanguages?: readonly string[]
): { value: string; label: string }[] {
	const detected = systemLocale(navigatorLanguages);
	return [
		{
			value: SYSTEM_LOCALE_OPTION,
			label: translate('prefs_reading_language_system', {
				values: { language: translate(LANGUAGE_LABEL_KEYS[detected as 'en' | 'fr']) }
			})
		},
		...(['en', 'fr'] as const).map((value) => ({
			value,
			label: translate(LANGUAGE_LABEL_KEYS[value])
		}))
	];
}

export function selectedLocaleValue(profileLocale: string | null | undefined): string {
	return matchLocale(profileLocale) ?? SYSTEM_LOCALE_OPTION;
}

export function draftFromPreferences(
	data: PreferencesSettingsResponse,
	locale: string
): ReadingAppearanceDraft {
	return {
		theme: data.theme,
		accentColor: data.appearance.accent_color,
		sidebarMode: data.layout.sidebar_mode,
		defaultView: data.layout.default_view,
		listDensity: data.layout.list_density,
		sidePanel: data.layout.side_panel,
		triageMode: data.workflow.triage_mode,
		autoAdvance: data.workflow.auto_advance,
		fontFamily: data.reader.font_family,
		fontSize: data.reader.font_size,
		lineHeight: data.reader.line_height,
		emailOpenMode: data.reader.email_open_mode ?? 'reader',
		locale
	};
}

export function readingAppearanceSnapshot(draft: ReadingAppearanceDraft): string {
	return JSON.stringify({
		theme: draft.theme,
		accentColor: draft.accentColor,
		sidebarMode: draft.sidebarMode,
		defaultView: draft.defaultView,
		listDensity: draft.listDensity,
		sidePanel: draft.sidePanel,
		triageMode: draft.triageMode,
		autoAdvance: draft.autoAdvance,
		fontFamily: draft.fontFamily,
		fontSize: draft.fontSize,
		lineHeight: draft.lineHeight,
		emailOpenMode: draft.emailOpenMode,
		locale: draft.locale
	});
}

export function readingPreferencesSnapshot(draft: ReadingAppearanceDraft): string {
	return JSON.stringify({
		theme: draft.theme,
		accentColor: draft.accentColor,
		sidebarMode: draft.sidebarMode,
		defaultView: draft.defaultView,
		listDensity: draft.listDensity,
		sidePanel: draft.sidePanel,
		triageMode: draft.triageMode,
		autoAdvance: draft.autoAdvance,
		fontFamily: draft.fontFamily,
		fontSize: draft.fontSize,
		lineHeight: draft.lineHeight,
		emailOpenMode: draft.emailOpenMode
	});
}

export function buildPreferencesSaveBody(
	draft: ReadingAppearanceDraft,
	serverData: PreferencesSettingsResponse | null
): PreferencesSettingsResponse {
	return {
		theme: draft.theme,
		appearance: { accent_color: draft.accentColor },
		layout: {
			sidebar_mode: draft.sidebarMode,
			default_view: draft.defaultView,
			list_density: draft.listDensity,
			side_panel: draft.sidePanel
		},
		workflow: { triage_mode: draft.triageMode, auto_advance: draft.autoAdvance },
		reader: {
			font_family: draft.fontFamily,
			font_size: draft.fontSize,
			line_height: draft.lineHeight,
			email_open_mode: draft.emailOpenMode
		},
		ai: serverData?.ai ?? { mila_enabled: true, custom_prompt: null }
	};
}

export function bumpFontSize(fontSize: ReaderFontSizeDto, delta: 1 | -1): ReaderFontSizeDto {
	const index = FONT_SIZES.indexOf(fontSize);
	const next = Math.max(0, Math.min(FONT_SIZES.length - 1, index + delta));
	return FONT_SIZES[next] ?? fontSize;
}

export function readerPreviewStyles(
	fontFamily: ReaderFontFamilyDto,
	fontSize: ReaderFontSizeDto,
	lineHeight: ReaderLineHeightDto
): ReaderPreviewStyles {
	return {
		fontFamily:
			fontFamily === 'serif'
				? "'New York', 'Iowan Old Style', Georgia, 'Times New Roman', serif"
				: fontFamily === 'mono'
					? "'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace"
					: "-apple-system, 'SF Pro Display', 'Helvetica Neue', sans-serif",
		fontSize: fontSize === 'small' ? '14px' : fontSize === 'large' ? '17px' : '15.5px',
		lineHeight: lineHeight === 'compact' ? '1.42' : '1.6'
	};
}
