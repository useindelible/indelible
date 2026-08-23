<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import {
		createPromptPreset,
		deletePromptPreset,
		getConfig,
		getStatus,
		listPromptPresets,
		testConfig,
		updatePromptPreset,
		upsertConfig,
		reindexConfig
	} from '$lib/api';
	import type {
		MilaConfigResponse,
		MilaPromptPresetResponse,
		MilaPromptPresetsResponse,
		MilaStatusResponse
	} from '$lib/api';
	import SavePill from '$lib/components/settings/SavePill.svelte';
	import { t } from '$lib/i18n';
	import MilaHero from './components/MilaHero.svelte';
	import MilaIndexingStatus from './components/MilaIndexingStatus.svelte';
	import MilaProviderSettings from './components/MilaProviderSettings.svelte';
	import PromptPresetSection from './components/PromptPresetSection.svelte';
	import {
		ACTIONS,
		applyMilaConfig,
		buildMilaSaveBody,
		buildMilaTestBody,
		createPresetEditor,
		editPresetEditor,
		emptyMilaDraft,
		milaConfigSnapshot,
		type ActionKey,
		type MilaConfigDraft,
		type PresetEditorState,
		type TestState
	} from './mila-settings-model';

	let config = $state<MilaConfigResponse | null>(null);
	let presets = $state<MilaPromptPresetsResponse | null>(null);
	let draft = $state<MilaConfigDraft>(emptyMilaDraft());
	let loading = $state(true);
	let loadError = $state('');
	let savedSnapshot = $state('');
	let saving = $state(false);
	let showSaved = $state(false);
	let saveError = $state('');
	let testState = $state<TestState>('idle');
	let testMessage = $state('');
	let expandedPresetId = $state<string | null>(null);
	let editorState = $state<PresetEditorState | null>(null);
	let editorSaving = $state(false);
	let indexingStatus = $state<MilaStatusResponse | null>(null);
	let indexingRetrying = $state(false);
	let indexingStatusError = $state('');

	const isDirty = $derived(milaConfigSnapshot(draft) !== savedSnapshot);

	onMount(() => {
		loadAll();
	});

	async function loadAll() {
		loading = true;
		loadError = '';
		try {
			const [cfgRes, prRes] = await Promise.all([getConfig(), listPromptPresets()]);
			if (cfgRes.data) {
				config = cfgRes.data;
				draft = applyMilaConfig(cfgRes.data);
			}
			presets = prRes.data ?? null;
			untrack(() => {
				savedSnapshot = milaConfigSnapshot(draft);
			});
			await refreshIndexingStatus();
		} catch {
			loadError = $t('prefs_ai_error_load');
		} finally {
			loading = false;
		}
	}

	async function refreshIndexingStatus() {
		try {
			const { data, error } = await getStatus();
			if (error || !data) throw new Error('Status unavailable');
			indexingStatus = data;
			indexingStatusError = '';
		} catch {
			indexingStatusError = $t('prefs_ai_status_unavailable_hint');
		}
	}

	async function retryIndexing() {
		indexingRetrying = true;
		saveError = '';
		try {
			const { data, error } = await reindexConfig({ body: buildMilaSaveBody(draft) });
			if (error || !data) throw new Error('Retry failed');
			config = data;
			await refreshIndexingStatus();
		} catch {
			saveError = $t('prefs_ai_error_restart_indexing');
		} finally {
			indexingRetrying = false;
		}
	}

	$effect(() => {
		if (!indexingStatus?.is_indexing) return;
		const timer = setTimeout(() => void refreshIndexingStatus(), 2000);
		return () => clearTimeout(timer);
	});

	function updateDraft(patch: Partial<MilaConfigDraft>) {
		draft = { ...draft, ...patch };
	}

	async function testConnection() {
		testState = 'testing';
		testMessage = '';
		try {
			const { data } = await testConfig({ body: buildMilaTestBody(draft) });
			if (data?.success) {
				testState = 'success';
				testMessage = '';
			} else {
				testState = 'error';
				testMessage = data?.error ?? $t('prefs_ai_test_failed');
			}
		} catch {
			testState = 'error';
			testMessage = $t('prefs_ai_request_failed');
		}
	}

	async function save() {
		saving = true;
		saveError = '';
		try {
			const body = buildMilaSaveBody(draft);
			const { data } = await upsertConfig({ body });
			if (data) {
				config = data;
				draft = applyMilaConfig(data);
				await refreshIndexingStatus();
				untrack(() => {
					savedSnapshot = milaConfigSnapshot(draft);
				});
				showSaved = true;
				setTimeout(() => {
					showSaved = false;
				}, 2000);
			}
		} catch {
			saveError = $t('prefs_ai_error_save_config');
		} finally {
			saving = false;
		}
	}

	function discard() {
		if (config) draft = applyMilaConfig(config);
		saveError = '';
		untrack(() => {
			savedSnapshot = milaConfigSnapshot(draft);
		});
	}

	async function reloadPresets() {
		const { data } = await listPromptPresets();
		presets = data ?? null;
	}

	function startAddPreset(action: ActionKey) {
		editorState = createPresetEditor(action);
	}

	function startEditPreset(action: ActionKey, preset: MilaPromptPresetResponse) {
		editorState = editPresetEditor(action, preset);
	}

	function cancelEditor() {
		editorState = null;
	}

	function updateEditor(patch: Partial<PresetEditorState>) {
		if (!editorState) return;
		editorState = { ...editorState, ...patch };
	}

	async function saveEditor() {
		if (!editorState) return;
		editorSaving = true;
		try {
			if (editorState.mode === 'add') {
				await createPromptPreset({
					body: {
						action: editorState.action,
						name: editorState.name,
						system_prompt: editorState.system_prompt,
						is_default: editorState.is_default
					}
				});
			} else if (editorState.id) {
				await updatePromptPreset({
					path: { preset_id: editorState.id },
					body: {
						name: editorState.name,
						system_prompt: editorState.system_prompt,
						is_default: editorState.is_default
					}
				});
			}
			editorState = null;
			await reloadPresets();
		} catch {
			saveError = $t('prefs_ai_error_save_preset');
		} finally {
			editorSaving = false;
		}
	}

	async function handleDeletePreset(id: string) {
		try {
			await deletePromptPreset({ path: { preset_id: id } });
			if (expandedPresetId === id) expandedPresetId = null;
			await reloadPresets();
		} catch {
			saveError = $t('prefs_ai_error_delete_preset');
		}
	}

	function togglePreset(id: string | null | undefined) {
		if (!id) return;
		expandedPresetId = expandedPresetId === id ? null : id;
	}
</script>

<div class="page">
	<MilaHero
		enabled={draft.enabled}
		onToggleEnabled={() => updateDraft({ enabled: !draft.enabled })}
	/>

	<div class="body-area">
		{#if loading}
			<p class="loading-text">{$t('common_loading')}</p>
		{:else if loadError}
			<p class="save-error">{loadError}</p>
		{:else}
			<MilaProviderSettings
				{config}
				{draft}
				{testMessage}
				{testState}
				onChange={updateDraft}
				onTestConnection={testConnection}
			/>

			{#if indexingStatus || indexingStatusError}
				<MilaIndexingStatus
					status={indexingStatus}
					error={indexingStatusError}
					embeddingModel={config?.byo_enabled
						? config.embedding_model
						: config
							? $t('prefs_ai_platform_default')
							: draft.embeddingModel}
					retrying={indexingRetrying}
					onRetry={retryIndexing}
					onRefresh={refreshIndexingStatus}
				/>
			{/if}

			<PromptPresetSection
				actions={ACTIONS}
				{editorSaving}
				{editorState}
				{expandedPresetId}
				{presets}
				onAdd={startAddPreset}
				onCancelEditor={cancelEditor}
				onDelete={handleDeletePreset}
				onEdit={startEditPreset}
				onEditorChange={updateEditor}
				onSaveEditor={saveEditor}
				onTogglePreset={togglePreset}
			/>

			{#if saveError}
				<p class="save-error">{saveError}</p>
			{/if}

			<SavePill {isDirty} {saving} {showSaved} onSave={save} onDiscard={discard} />
		{/if}
	</div>
</div>

<style>
	.page {
		display: flex;
		flex-direction: column;
	}
	.body-area {
		padding: 32px 56px 48px;
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 24px;
		max-width: 1080px;
		width: 100%;
		align-self: center;
		margin: 0 auto;
		box-sizing: border-box;
	}
	.loading-text {
		font-size: 13px;
		color: var(--text-tertiary);
		text-align: center;
		padding: 40px;
	}
	.save-error {
		font-size: 13px;
		color: var(--destructive);
		margin: 0;
	}
	@media (max-width: 720px) {
		.body-area {
			padding: 24px 20px 40px;
		}
	}
</style>
