<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import { loadArchivalSettings, saveArchivalSettings } from '$lib/api/settings';
	import SavePill from '$lib/components/settings/SavePill.svelte';
	import type { ArchivalSettingsResponse } from '$lib/api';
	import { t } from '$lib/i18n';
	import {
		actionFromApi,
		buildArchivalSettingsPayload,
		canToggleArchiveFormat,
		createArchivalSnapshot,
		DEFAULT_ARCHIVAL_SETTINGS,
		sensitivityFromApi,
		type ArchiveFormatToggleId,
		type DuplicateAction,
		type DuplicateSensitivity,
		type FormatId
	} from './archival-model';
	import ArchivalHero from './components/ArchivalHero.svelte';
	import ArchiveFormatsSection from './components/ArchiveFormatsSection.svelte';
	import DuplicateDetectionSection from './components/DuplicateDetectionSection.svelte';
	import ProxySection from './components/ProxySection.svelte';
	import './components/archival-shared.css';

	let formats = $state(structuredClone(DEFAULT_ARCHIVAL_SETTINGS.formats));
	let dupEnabled = $state(DEFAULT_ARCHIVAL_SETTINGS.dupEnabled);
	let dupSensitivity = $state<DuplicateSensitivity>(DEFAULT_ARCHIVAL_SETTINGS.dupSensitivity);
	let dupAction = $state<DuplicateAction>(DEFAULT_ARCHIVAL_SETTINGS.dupAction);
	let proxyUrl = $state(DEFAULT_ARCHIVAL_SETTINGS.proxyUrl);
	let proxyAll = $state(DEFAULT_ARCHIVAL_SETTINGS.proxyAll);
	let serverData = $state<ArchivalSettingsResponse | null>(null);
	let loading = $state(true);
	let loadError = $state('');
	let saveError = $state('');

	const proxyConfigured = $derived(proxyUrl.trim().length > 0);

	$effect(() => {
		if (!proxyConfigured && proxyAll) {
			proxyAll = false;
		}
	});

	function snapshot() {
		return createArchivalSnapshot({
			formats,
			dupEnabled,
			dupSensitivity,
			dupAction,
			proxyUrl,
			proxyAll
		});
	}

	let savedSnapshot = $state('');
	const isDirty = $derived(savedSnapshot !== '' && snapshot() !== savedSnapshot);
	let saving = $state(false);
	let showSaved = $state(false);
	let savedTimer: ReturnType<typeof setTimeout> | null = null;

	function applySettings(data: ArchivalSettingsResponse) {
		serverData = data;
		formats = {
			monolith: data.archive_formats.monolith,
			pdf: data.archive_formats.pdf,
			screenshot: data.archive_formats.screenshot
		};
		dupEnabled = data.duplicate_detection.enabled;
		dupSensitivity = sensitivityFromApi(data.duplicate_detection.sensitivity);
		dupAction = actionFromApi(data.duplicate_detection.on_duplicate);
		proxyUrl = data.proxy.url ?? '';
		proxyAll = data.proxy.all_requests && proxyUrl.trim().length > 0;
		saveError = '';
		untrack(() => {
			savedSnapshot = snapshot();
		});
	}

	async function loadSettings() {
		const result = await loadArchivalSettings();
		if (result.success) {
			applySettings(result.data);
		} else {
			loadError = result.error;
		}
		loading = false;
	}

	onMount(() => {
		void loadSettings();
		return () => {
			if (savedTimer) clearTimeout(savedTimer);
		};
	});

	function toggleFormat(id: FormatId) {
		if (!canToggleArchiveFormat(id)) return;
		formats[id as ArchiveFormatToggleId] = !formats[id as ArchiveFormatToggleId];
	}

	function discard() {
		if (serverData) {
			applySettings(serverData);
		}
		saveError = '';
	}

	async function save() {
		saving = true;
		saveError = '';
		const body = buildArchivalSettingsPayload({
			serverData,
			formats,
			dupEnabled,
			dupSensitivity,
			dupAction,
			proxyUrl,
			proxyAll
		});
		const result = await saveArchivalSettings(body);
		if (!result.success) {
			saveError = result.error;
			saving = false;
			return;
		}

		applySettings(result.data);
		saving = false;
		showSaved = true;
		if (savedTimer) clearTimeout(savedTimer);
		savedTimer = setTimeout(() => {
			showSaved = false;
		}, 1800);
	}
</script>

{#if loading}
	<div class="archival-loading">{$t('archival_loading')}</div>
{:else if loadError}
	<p class="archival-load-error">{loadError}</p>
{:else}
	<div class="page archival-page">
		<ArchivalHero {formats} />

		<div class="body-area">
			<ArchiveFormatsSection {formats} onToggleFormat={toggleFormat} />

			<DuplicateDetectionSection
				{dupEnabled}
				{dupSensitivity}
				{dupAction}
				onEnabledChange={(enabled) => {
					dupEnabled = enabled;
				}}
				onSensitivityChange={(sensitivity) => {
					dupSensitivity = sensitivity;
				}}
				onActionChange={(action) => {
					dupAction = action;
				}}
			/>

			<ProxySection {proxyUrl} {proxyAll} />

			{#if saveError}
				<p class="save-error">{saveError}</p>
			{/if}
			<SavePill {isDirty} {saving} {showSaved} onSave={save} onDiscard={discard} />
		</div>
	</div>
{/if}
