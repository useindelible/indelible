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
		milaEmbeddingIdentityChanged,
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
	let testMessage = $state('Not tested yet');
	let expandedPresetId = $state<string | null>(null);
	let editorState = $state<PresetEditorState | null>(null);
	let editorSaving = $state(false);
	let indexingStatus = $state<MilaStatusResponse | null>(null);
	let indexingRetrying = $state(false);
	let indexingStatusError = $state('');

	const isDirty = $derived(milaConfigSnapshot(draft) !== savedSnapshot);
	const willReindex = $derived(milaEmbeddingIdentityChanged(config, draft));

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
			loadError = 'Failed to load AI configuration.';
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
			indexingStatusError = 'The status service did not respond.';
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
			saveError = 'Failed to restart Mila indexing.';
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
		testMessage = 'Checking embeddings and chat…';
		try {
			const { data } = await testConfig({ body: buildMilaTestBody(draft) });
			if (data?.success) {
				testState = 'success';
				testMessage = 'Connection live · embedding and chat responded.';
			} else {
				testState = 'error';
				testMessage = data?.error ?? 'Test failed';
			}
		} catch {
			testState = 'error';
			testMessage = 'Request failed';
		}
	}

	async function save() {
		saving = true;
		saveError = '';
		try {
			const body = buildMilaSaveBody(draft);
			// The rebuild cost is disclosed inline before saving, so the save itself
			// does not stop for a confirm dialog.
			const needsReindex = willReindex;
			const { data } = needsReindex ? await reindexConfig({ body }) : await upsertConfig({ body });
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
			saveError = 'Failed to save configuration.';
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
			saveError = 'Failed to save preset.';
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
			saveError = 'Failed to delete preset.';
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
			<p class="loading-text">Loading…</p>
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
					embeddingModel={config?.embedding_model ?? draft.embeddingModel}
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

			{#if willReindex}
				<div class="reindex-notice" role="status">
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<path d="M12 9v4" />
						<path
							d="M10.3 3.9L2.6 17.3A1.6 1.6 0 0 0 4 19.7h16a1.6 1.6 0 0 0 1.4-2.4L13.7 3.9a1.6 1.6 0 0 0-2.8 0z"
						/>
						<circle cx="12" cy="16.6" r="0.6" />
					</svg>
					<span>
						Saving rebuilds embeddings for
						{#if indexingStatus}<strong>{indexingStatus.eligible_items} items</strong>{:else}every
							saved item{/if}. Mila keeps answering from the current index until the new one
						finishes.
					</span>
				</div>
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
	/* Shown before saving rather than as a confirm dialog, so the cost of the
	   rebuild is visible while the endpoint is still being edited. */
	.reindex-notice {
		display: flex;
		align-items: flex-start;
		gap: 9px;
		padding: 12px 14px;
		border-radius: 12px;
		background: var(--mila-status-warn-bg);
		box-shadow: inset 0 0 0 0.5px var(--mila-status-warn-bg);
		color: var(--mila-status-warn-text);
		font-size: 12.5px;
		line-height: 1.45;
		letter-spacing: -0.005em;
	}
	.reindex-notice svg {
		width: 14px;
		height: 14px;
		flex-shrink: 0;
		margin-top: 1px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.8;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
	.reindex-notice strong {
		font-weight: 600;
	}
	@media (max-width: 720px) {
		.body-area {
			padding: 24px 20px 40px;
		}
	}
</style>
