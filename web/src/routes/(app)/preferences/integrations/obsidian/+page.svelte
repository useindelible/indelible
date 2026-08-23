<script lang="ts">
	import { onMount } from 'svelte';
	import SavePill from '$lib/components/settings/SavePill.svelte';
	import {
		loadIntegrationConnections,
		loadObsidianSettings,
		previewObsidianSettings,
		saveObsidianSettings,
		setupObsidianExportConnection
	} from '$lib/api/integrations';
	import { deriveConnectionState } from '$lib/integrations/status';
	import { t } from '$lib/i18n';
	import { renderMarkdown } from '$lib/utils/markdown';
	import type {
		IntegrationConnectionDto,
		ObsidianPreviewResponse,
		ObsidianSettingsDto
	} from '$lib/api';
	import ObsidianBehaviorSection from './components/ObsidianBehaviorSection.svelte';
	import ObsidianHero from './components/ObsidianHero.svelte';
	import ObsidianPathsSection from './components/ObsidianPathsSection.svelte';
	import ObsidianPreviewPanel from './components/ObsidianPreviewPanel.svelte';
	import ObsidianSyncSection from './components/ObsidianSyncSection.svelte';
	import ObsidianTemplatesSection from './components/ObsidianTemplatesSection.svelte';
	import {
		buildObsidianSaveBody,
		formatObsidianLastSync,
		obsidianHeroState,
		obsidianHeroStatusLabel,
		previewBody,
		previewFilePath,
		previewMissingSummary,
		serializeForCompare,
		snapshotObsidianSettings,
		type PreviewView
	} from './obsidian-model';

	type TemplateKey =
		| 'properties_template'
		| 'page_title_template'
		| 'metadata_template'
		| 'highlight_header_template'
		| 'highlight_template'
		| 'sync_notification_template';

	let connection = $state<IntegrationConnectionDto | undefined>(undefined);
	let settings = $state<ObsidianSettingsDto | null>(null);
	let baseline = $state<ObsidianSettingsDto | null>(null);
	let loading = $state(true);
	let loadError = $state<string | null>(null);
	let saving = $state(false);
	let saveError = $state<string | null>(null);
	let savedFlash = $state(false);
	let previewing = $state(false);
	let preview = $state<ObsidianPreviewResponse | null>(null);
	let previewError = $state<string | null>(null);
	let varsOpen = $state(false);
	let previewViewLocal = $state<PreviewView>('note');
	let setupRunning = $state(false);
	let setupError = $state<string | null>(null);
	let validatedPreviewSignature = $state('');
	let lastPreviewSignature = '';
	let previewDebounce: ReturnType<typeof setTimeout> | null = null;

	const connectionState = $derived(deriveConnectionState(connection));
	const heroStateAttr = $derived(obsidianHeroState(connection, connectionState));
	const heroStatusLabel = $derived($t(obsidianHeroStatusLabel(heroStateAttr)));
	const lastSyncLabel = $derived(formatObsidianLastSync(connection, $t));
	const currentPreviewPath = $derived(previewFilePath(preview, previewViewLocal));
	const currentPreviewBody = $derived(previewBody(preview, previewViewLocal));
	const currentPreviewBodyHtml = $derived(
		currentPreviewBody && previewViewLocal === 'note' ? renderMarkdown(currentPreviewBody) : ''
	);
	const currentPreviewMissingSummary = $derived(previewMissingSummary(preview, previewViewLocal));
	const fullTextOffAndMissing = $derived(
		previewViewLocal === 'full' &&
			!!preview &&
			!preview.full_document_text &&
			!!settings &&
			!settings.export_all_reader_documents
	);
	const fullTextMissing = $derived(
		previewViewLocal === 'full' &&
			!!preview &&
			!preview.full_document_text &&
			!!settings &&
			settings.export_all_reader_documents
	);
	const currentSettingsSignature = $derived(
		settings ? JSON.stringify(serializeForCompare(settings)) : ''
	);
	const saveDisabled = $derived(
		previewing ||
			!currentSettingsSignature ||
			validatedPreviewSignature !== currentSettingsSignature
	);
	const isDirty = $derived.by(() => {
		if (!settings || !baseline) return false;
		return (
			JSON.stringify(serializeForCompare(settings)) !==
			JSON.stringify(serializeForCompare(baseline))
		);
	});

	onMount(() => {
		void refresh();
	});

	$effect(() => {
		if (!settings || !connection) return;
		const signature = currentSettingsSignature;
		if (signature === lastPreviewSignature) return;
		const isFirst = lastPreviewSignature === '';
		lastPreviewSignature = signature;
		if (previewDebounce) {
			clearTimeout(previewDebounce);
			previewDebounce = null;
		}
		if (isFirst) {
			void renderPreview(signature);
		} else {
			previewDebounce = setTimeout(() => {
				previewDebounce = null;
				void renderPreview(signature);
			}, 500);
		}
	});

	async function refresh() {
		loading = true;
		loadError = null;
		const result = await loadIntegrationConnections();
		if (!result.success) {
			loadError = result.error;
			loading = false;
			return;
		}
		connection = result.data.connections.find((item) => item.provider === 'obsidian');
		if (!connection) {
			settings = null;
			baseline = null;
			loading = false;
			return;
		}
		const settingsResult = await loadObsidianSettings(connection.id);
		if (settingsResult.success) {
			settings = settingsResult.data;
			baseline = snapshotObsidianSettings(settingsResult.data);
		} else {
			loadError = settingsResult.error;
		}
		loading = false;
	}

	async function handleSetup() {
		if (setupRunning) return;
		setupRunning = true;
		setupError = null;
		const result = await setupObsidianExportConnection();
		if (!result.success) {
			setupError = result.error;
			setupRunning = false;
			return;
		}
		await refresh();
		setupRunning = false;
	}

	async function handleSave() {
		if (!connection || !settings || saveDisabled) return;
		saving = true;
		saveError = null;
		savedFlash = false;
		const result = await saveObsidianSettings(connection.id, buildObsidianSaveBody(settings));
		saving = false;
		if (result.success) {
			settings = result.data;
			baseline = snapshotObsidianSettings(result.data);
			savedFlash = true;
			setTimeout(() => {
				savedFlash = false;
			}, 2000);
		} else {
			saveError = result.error;
		}
	}

	function handleDiscard() {
		if (!baseline) return;
		settings = snapshotObsidianSettings(baseline);
		saveError = null;
	}

	async function renderPreview(expectedSignature?: string) {
		if (!connection || !settings) return;
		const signature = expectedSignature ?? currentSettingsSignature;
		if (signature !== currentSettingsSignature) return;
		const requestedSettings = snapshotObsidianSettings(settings);
		previewing = true;
		previewError = null;
		const result = await previewObsidianSettings(connection.id, { settings: requestedSettings });
		if (signature !== currentSettingsSignature) return;
		previewing = false;
		if (result.success) {
			preview = result.data;
			validatedPreviewSignature = signature;
		} else {
			validatedPreviewSignature = '';
			previewError = result.error;
		}
	}

	function updateSettings(patch: Partial<ObsidianSettingsDto>) {
		if (!settings) return;
		settings = { ...settings, ...patch };
	}

	function setFolderTemplate(key: string, value: string) {
		if (!settings) return;
		settings = {
			...settings,
			category_folder_templates: {
				...settings.category_folder_templates,
				[key]: value
			}
		};
	}

	function setTemplate(key: TemplateKey, value: string) {
		if (!settings) return;
		settings = { ...settings, [key]: value };
	}
</script>

<svelte:head>
	<title>Obsidian — Indelible</title>
</svelte:head>

<div class="obs-page" data-page-state={heroStateAttr}>
	<ObsidianHero
		{connection}
		heroState={heroStateAttr}
		statusLabel={heroStatusLabel}
		{lastSyncLabel}
		{setupRunning}
		{setupError}
		onSetup={handleSetup}
	/>

	<div class="body-area">
		{#if loading}
			<p class="muted body-msg">{$t('integrations_obsidian_loading_settings')}</p>
		{:else if loadError}
			<p class="error body-msg">{loadError}</p>
		{:else if connection && settings !== null}
			<div class="settings-body">
				<ObsidianSyncSection {connection} heroState={heroStateAttr} />
				<ObsidianBehaviorSection {settings} onChange={updateSettings} />
				<ObsidianPathsSection
					{settings}
					onChange={updateSettings}
					onFolderTemplateChange={setFolderTemplate}
				/>
				<ObsidianTemplatesSection
					{settings}
					{varsOpen}
					{saveError}
					onToggleVars={() => (varsOpen = !varsOpen)}
					onTemplateChange={setTemplate}
				/>
				<ObsidianPreviewPanel
					previewView={previewViewLocal}
					{previewing}
					previewFilePath={currentPreviewPath}
					previewBody={currentPreviewBody}
					previewBodyHtml={currentPreviewBodyHtml}
					{previewError}
					{fullTextOffAndMissing}
					{fullTextMissing}
					previewMissingSummary={currentPreviewMissingSummary}
					onEnableFullText={() => updateSettings({ export_all_reader_documents: true })}
					onRenderPreview={() => void renderPreview()}
					onSetPreviewView={(view) => (previewViewLocal = view)}
				/>
			</div>

			<SavePill
				{isDirty}
				{saving}
				{saveDisabled}
				showSaved={savedFlash}
				onSave={handleSave}
				onDiscard={handleDiscard}
			/>
		{/if}
	</div>
</div>

<style>
	.obs-page {
		display: flex;
		flex-direction: column;
		min-height: 100%;
	}
	.obs-page :global(.hero-inner) {
		max-width: none;
		width: 100%;
	}
	.body-area {
		flex: 1;
		display: flex;
		flex-direction: column;
		background: var(--obs-body-bleed);
		position: relative;
	}
	.body-msg {
		padding: 28px 36px;
	}
	.muted {
		color: var(--text-tertiary);
	}
	.error {
		color: var(--destructive);
	}
	.settings-body {
		flex: 1;
		padding: 28px 36px 80px;
		max-width: 940px;
		margin: 0 auto;
		width: 100%;
		display: flex;
		flex-direction: column;
		min-height: 0;
	}

	@media (max-width: 599px) {
		.settings-body {
			padding: 20px 16px 56px;
		}

		.body-msg {
			padding: 20px 16px;
		}
	}
</style>
