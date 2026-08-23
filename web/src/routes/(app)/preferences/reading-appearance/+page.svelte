<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import SavePill from '$lib/components/settings/SavePill.svelte';
	import { loadPreferencesSettings, savePreferencesSettings } from '$lib/api/settings';
	import { applyTheme, saveTheme } from '$lib/styles/theme';
	import { getAuth } from '$lib/stores/auth.svelte';
	import { getLibrary } from '$lib/stores/library.svelte';
	import { getAppPreferences } from '$lib/stores/app-preferences.svelte';
	import { applyProfileLocale, t } from '$lib/i18n';
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
	import EmailOpenModeSection from './components/EmailOpenModeSection.svelte';
	import KeyboardShortcutsSection from './components/KeyboardShortcutsSection.svelte';
	import LayoutSection from './components/LayoutSection.svelte';
	import LocaleSection from './components/LocaleSection.svelte';
	import ReaderDefaultsSection from './components/ReaderDefaultsSection.svelte';
	import ReadingHeroPreview from './components/ReadingHeroPreview.svelte';
	import ThemeSection from './components/ThemeSection.svelte';
	import WorkflowSection from './components/WorkflowSection.svelte';
	import {
		ACCENT_SWATCHES,
		buildPreferencesSaveBody,
		draftFromPreferences,
		FONT_SIZE_LABEL,
		localeOptions,
		readerPreviewStyles,
		readingPreferencesSnapshot,
		selectedLocaleValue,
		type ReadingAppearanceDraft
	} from './reading-appearance-model';

	const auth = getAuth();
	const lib = getLibrary();
	const appPrefs = getAppPreferences();

	let theme = $state<ThemeDto>('system');
	let accentColor = $state<AccentColorDto>('blue');

	let sidebarMode = $state<SidebarModeDto>('expanded');
	let defaultView = $state<DefaultViewDto>('library');
	let listDensity = $state<ListDensityDto>('compact');
	let sidePanel = $state<SidePanelModeDto>('open');

	let triageMode = $state<TriageModeDto>('focus');
	let autoAdvance = $state(true);

	let fontFamily = $state<ReaderFontFamilyDto>('serif');
	let fontSize = $state<ReaderFontSizeDto>('medium');
	let lineHeight = $state<ReaderLineHeightDto>('relaxed');
	let emailOpenMode = $state<ReaderOpenModeDto>('reader');

	let locale = $state(selectedLocaleValue(auth.user?.locale));
	let serverLocale = $state(selectedLocaleValue(auth.user?.locale));
	let serverData = $state<PreferencesSettingsResponse | null>(null);
	let loading = $state(true);
	let loadError = $state('');
	let saving = $state(false);
	let showSaved = $state(false);
	let saveError = $state('');
	let savedPreferencesSnapshot = $state('');

	const preview = $derived(readerPreviewStyles(fontFamily, fontSize, lineHeight));
	const locales = $derived(localeOptions($t));
	const preferencesDirty = $derived(
		savedPreferencesSnapshot !== '' && preferencesSnapshot() !== savedPreferencesSnapshot
	);
	const localeDirty = $derived(locale !== serverLocale);
	const isDirty = $derived(preferencesDirty || localeDirty);

	$effect(() => {
		if (savedPreferencesSnapshot !== '') applyTheme(theme);
	});

	function currentDraft(): ReadingAppearanceDraft {
		const draftTheme = theme;
		const draftAccentColor = accentColor;
		const draftSidebarMode = sidebarMode;
		const draftDefaultView = defaultView;
		const draftListDensity = listDensity;
		const draftSidePanel = sidePanel;
		const draftTriageMode = triageMode;
		const draftAutoAdvance = autoAdvance;
		const draftFontFamily = fontFamily;
		const draftFontSize = fontSize;
		const draftLineHeight = lineHeight;
		const draftEmailOpenMode = emailOpenMode;
		const draftLocale = locale;

		return {
			theme: draftTheme,
			accentColor: draftAccentColor,
			sidebarMode: draftSidebarMode,
			defaultView: draftDefaultView,
			listDensity: draftListDensity,
			sidePanel: draftSidePanel,
			triageMode: draftTriageMode,
			autoAdvance: draftAutoAdvance,
			fontFamily: draftFontFamily,
			fontSize: draftFontSize,
			lineHeight: draftLineHeight,
			emailOpenMode: draftEmailOpenMode,
			locale: draftLocale
		};
	}

	function preferencesSnapshot() {
		return readingPreferencesSnapshot(currentDraft());
	}

	function setDraft(draft: ReadingAppearanceDraft) {
		theme = draft.theme;
		accentColor = draft.accentColor;
		sidebarMode = draft.sidebarMode;
		defaultView = draft.defaultView;
		listDensity = draft.listDensity;
		sidePanel = draft.sidePanel;
		triageMode = draft.triageMode;
		autoAdvance = draft.autoAdvance;
		fontFamily = draft.fontFamily;
		fontSize = draft.fontSize;
		lineHeight = draft.lineHeight;
		emailOpenMode = draft.emailOpenMode;
		locale = draft.locale;
	}

	function applySettings(data: PreferencesSettingsResponse, nextLocale: string) {
		serverData = data;
		setDraft(draftFromPreferences(data, nextLocale));
		serverLocale = nextLocale;
		untrack(() => {
			savedPreferencesSnapshot = preferencesSnapshot();
		});
	}

	function applyPersistedPreferences(data: PreferencesSettingsResponse) {
		serverData = data;
		setDraft(draftFromPreferences(data, locale));
		untrack(() => {
			savedPreferencesSnapshot = preferencesSnapshot();
		});
		saveTheme(data.theme);
		lib.applyPreferences(data);
		appPrefs.setDefaultView(data.layout.default_view);
	}

	onMount(async () => {
		const result = await loadPreferencesSettings();
		if (result.success) {
			const initialLocale = selectedLocaleValue(auth.user?.locale) || locale;
			applySettings(result.data, initialLocale);
		} else {
			loadError = result.error;
		}
		loading = false;
	});

	function discard() {
		if (serverData) applySettings(serverData, serverLocale);
		saveError = '';
	}

	async function save() {
		saving = true;
		saveError = '';

		if (preferencesDirty) {
			const result = await savePreferencesSettings(
				buildPreferencesSaveBody(currentDraft(), serverData)
			);
			if (!result.success) {
				saveError = result.error;
				saving = false;
				return;
			}
			applyPersistedPreferences(result.data);
		}

		if (localeDirty) {
			const profileLocale = locale || null;
			const profileResult = await auth.updateProfile({ locale: profileLocale });
			if (!profileResult.success) {
				saveError = profileResult.error ?? $t('prefs_reading_locale_update_failed');
				saving = false;
				return;
			}
			await applyProfileLocale(profileLocale);
			serverLocale = locale;
		}

		showSaved = true;
		setTimeout(() => {
			showSaved = false;
		}, 2000);
		saving = false;
	}
</script>

{#if loading}
	<div class="loading-state">{$t('prefs_reading_loading')}</div>
{:else if loadError}
	<p class="load-error">{loadError}</p>
{:else}
	<ReadingHeroPreview {preview} />

	<div class="body-area">
		<ThemeSection
			{theme}
			{accentColor}
			accentSwatches={ACCENT_SWATCHES}
			onThemeChange={(value) => (theme = value)}
			onAccentColorChange={(value) => (accentColor = value)}
		/>

		<LayoutSection
			{sidebarMode}
			{defaultView}
			{listDensity}
			{sidePanel}
			onSidebarModeChange={(value) => (sidebarMode = value)}
			onDefaultViewChange={(value) => (defaultView = value)}
			onListDensityChange={(value) => (listDensity = value)}
			onSidePanelChange={(value) => (sidePanel = value)}
		/>

		<WorkflowSection
			{triageMode}
			{autoAdvance}
			onTriageModeChange={(value) => (triageMode = value)}
			onAutoAdvanceChange={(value) => (autoAdvance = value)}
		/>

		<ReaderDefaultsSection
			{fontFamily}
			{fontSize}
			{lineHeight}
			fontSizeLabel={FONT_SIZE_LABEL}
			onFontFamilyChange={(value) => (fontFamily = value)}
			onFontSizeChange={(value) => (fontSize = value)}
			onLineHeightChange={(value) => (lineHeight = value)}
		/>

		<EmailOpenModeSection
			{emailOpenMode}
			onEmailOpenModeChange={(value) => (emailOpenMode = value)}
		/>

		<LocaleSection {locale} {locales} onLocaleChange={(value) => (locale = value)} />

		<KeyboardShortcutsSection />

		{#if saveError}
			<p class="save-error">{saveError}</p>
		{/if}

		<SavePill {isDirty} {saving} {showSaved} onSave={save} onDiscard={discard} />
	</div>
{/if}

<style>
	.loading-state,
	.load-error {
		font-family: var(--font-sans);
		font-size: 14px;
		padding: 32px 56px;
	}

	.load-error,
	.save-error {
		color: var(--destructive);
	}

	.body-area {
		padding: 32px 56px 16px;
		flex: 1;
		display: flex;
		flex-direction: column;
		max-width: 920px;
		width: 100%;
		align-self: center;
		margin: 0 auto;
		box-sizing: border-box;
	}

	.save-error {
		font-family: var(--font-sans);
		font-size: 12px;
		margin: 12px 4px 0;
	}

	@media (max-width: 599px) {
		.body-area {
			padding: 24px 20px 16px;
		}
	}
</style>
