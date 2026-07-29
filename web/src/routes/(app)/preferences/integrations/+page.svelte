<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import IntegrationCallbackBanner from '$lib/components/integrations/IntegrationCallbackBanner.svelte';
	import IntegrationDisconnectDialog from '$lib/components/integrations/IntegrationDisconnectDialog.svelte';
	import ImportRollbackDialog from '$lib/components/imports/ImportRollbackDialog.svelte';
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import {
		disconnectIntegration,
		dispatchIntegrationSync,
		loadIntegrationConnections,
		startIntegrationAuthorization
	} from '$lib/api/integrations';
	import {
		fetchImportJob,
		fetchRecentImports,
		rollbackImportJob,
		uploadReadwiseImportFiles,
		validateReadwiseCsv,
		type ImportJobLookupResult,
		type ReadwiseImportFiles
	} from '$lib/api/imports';
	import type { IntegrationConnectionDto, ImportJobStatusResponse } from '$lib/api';
	import { isPollingStatus, isTerminalImportStatus } from '$lib/integrations/status';
	import { parseIntegrationCallback, type IntegrationCallback } from '$lib/integrations/callback';
	import { findProvider } from '$lib/integrations/providers';
	import { getAuth } from '$lib/stores/auth.svelte';
	import { createPoll, type PollHandle } from '$lib/utils/polling';
	import ConnectionsSection from './components/ConnectionsSection.svelte';
	import ImportHistoryTable from './components/ImportHistoryTable.svelte';
	import IntegrationsHero from './components/IntegrationsHero.svelte';
	import ReadwiseImportSection from './components/ReadwiseImportSection.svelte';
	import {
		browserStoreLink,
		connectionRingCounts,
		connectionRingDash,
		formatUploadLimit,
		isOauthProviderAvailable,
		notionHubStatus,
		obsidianHubStatus,
		sevenDayDelta,
		sevenDayItems,
		type ImportSlot,
		type SyncState
	} from './integrations-hub-model';

	const auth = getAuth();
	const inboxAddress = $derived(auth.user?.ingest_library_email ?? '');
	const feedAddress = $derived(auth.user?.ingest_email ?? '');
	const readwiseUploadLimit = formatUploadLimit(findProvider('readwise')?.maxBytes);
	const CSV_HEADER_PROBE_BYTES = 4096;

	let copiedInbox = $state(false);
	let copiedFeed = $state(false);
	let connections = $state<IntegrationConnectionDto[]>([]);
	let availableOauthProviders = $state<string[] | null>(null);
	let connectionsLoading = $state(true);
	let connectionsError = $state<string | null>(null);
	let syncStateByConnection = $state<Record<string, SyncState>>({});
	let syncErrorByConnection = $state<Record<string, string>>({});
	let notionConnectError = $state<string | null>(null);
	let disconnectTarget = $state<IntegrationConnectionDto | null>(null);
	let disconnectBusy = $state(false);
	let disconnectError = $state<string | null>(null);
	let callback = $state<IntegrationCallback | null>(null);
	let activeJob = $state<ImportJobStatusResponse | null>(null);
	let activeSlot = $state<ImportSlot | null>(null);
	let busySlot = $state<ImportSlot | null>(null);
	let uploadError = $state<string | null>(null);
	let uploadErrorSlot = $state<ImportSlot | null>(null);
	let pollError = $state<string | null>(null);
	let dropHover = $state<ImportSlot | null>(null);
	let rollbackOpen = $state(false);
	let rollbackBusy = $state(false);
	let rollbackError = $state<string | null>(null);
	let rollbackNotice = $state<string | null>(null);
	let rollbackTargetJobId = $state<string | null>(null);
	let history = $state<ImportJobStatusResponse[]>([]);
	let pollHandle: PollHandle | null = null;

	const notionConnection = $derived(findConnection('notion'));
	const notionAvailable = $derived(isOauthProviderAvailable(availableOauthProviders, 'notion'));
	const obsidianConnection = $derived(findConnection('obsidian'));
	const notionStatus = $derived(notionHubStatus(notionConnection));
	const obsidianStatus = $derived(obsidianHubStatus(obsidianConnection));
	const ringCounts = $derived(connectionRingCounts(connections));
	const ringDash = $derived(connectionRingDash(ringCounts));
	const heroState = $derived<'populated' | 'empty'>(
		connections.length === 0 && history.length === 0 ? 'empty' : 'populated'
	);
	const extStore = $derived(
		browserStoreLink(typeof navigator === 'undefined' ? undefined : navigator.userAgent)
	);
	const activeIsTerminal = $derived(activeJob ? isTerminalImportStatus(activeJob.status) : false);
	const sevenDayItemCount = $derived(sevenDayItems(history));
	const sevenDayDeltaValue = $derived(sevenDayDelta(history));

	onMount(() => {
		consumeCallbackParams();
		void refreshConnections();
		void refreshHistory();
		const jobId = page.url.searchParams.get('job');
		if (jobId) void hydrateJob(jobId);
	});

	onDestroy(() => {
		pollHandle?.stop();
		pollHandle = null;
	});

	function consumeCallbackParams() {
		const url = page.url;
		const parsed = parseIntegrationCallback(url);
		if (!parsed) return;
		callback = parsed;
		const cleared = new URL(url);
		cleared.searchParams.delete('connected');
		cleared.searchParams.delete('integration_error');
		cleared.searchParams.delete('provider');
		const tail = `${cleared.search}${cleared.hash}`;
		// eslint-disable-next-line svelte/no-navigation-without-resolve -- URL uses resolve() for the route and appends current query/hash.
		void goto(`${resolve('/preferences/integrations')}${tail}`, {
			replaceState: true,
			noScroll: true,
			keepFocus: true
		});
	}

	function dismissCallback() {
		callback = null;
	}

	function handleCallbackAction(cb: IntegrationCallback) {
		if (cb.kind === 'success' && cb.provider === 'notion') {
			void goto(resolve('/preferences/integrations/notion'));
		}
	}

	async function refreshConnections() {
		connectionsLoading = true;
		connectionsError = null;
		const result = await loadIntegrationConnections();
		connectionsLoading = false;
		if (result.success) {
			connections = result.data.connections;
			availableOauthProviders = result.data.available_oauth_providers ?? null;
		} else {
			connectionsError = result.error;
		}
	}

	async function refreshHistory() {
		const result = await fetchRecentImports(25);
		if (result.success) history = result.data;
	}

	function findConnection(providerId: string): IntegrationConnectionDto | undefined {
		return connections.find((connection) => connection.provider === providerId);
	}

	async function startNotionAuthorization() {
		if (!notionAvailable) return;
		notionConnectError = null;
		const result = await startIntegrationAuthorization('notion');
		if (result.success) window.location.href = result.data.authorize_url;
		else notionConnectError = result.error;
	}

	function openNotionDetail() {
		void goto(resolve('/preferences/integrations/notion'));
	}

	function openObsidianDetail() {
		void goto(resolve('/preferences/integrations/obsidian'));
	}

	async function handleSync(connectionId: string) {
		syncStateByConnection = { ...syncStateByConnection, [connectionId]: 'pending' };
		syncErrorByConnection = { ...syncErrorByConnection, [connectionId]: '' };
		const result = await dispatchIntegrationSync(connectionId);
		if (result.success) {
			syncStateByConnection = { ...syncStateByConnection, [connectionId]: 'success' };
			void refreshConnections();
		} else {
			syncStateByConnection = { ...syncStateByConnection, [connectionId]: 'error' };
			syncErrorByConnection = { ...syncErrorByConnection, [connectionId]: result.error };
		}
	}

	function openDisconnectDialog(connection: IntegrationConnectionDto) {
		disconnectTarget = connection;
		disconnectError = null;
	}

	function closeDisconnectDialog() {
		if (disconnectBusy) return;
		disconnectTarget = null;
		disconnectError = null;
	}

	async function confirmDisconnect() {
		const target = disconnectTarget;
		if (!target) return;
		disconnectBusy = true;
		disconnectError = null;
		const result = await disconnectIntegration(target.id);
		disconnectBusy = false;
		if (result.success) {
			disconnectTarget = null;
			void refreshConnections();
		} else {
			disconnectError = result.error;
		}
	}

	function disconnectProviderName(connection: IntegrationConnectionDto): string {
		const map: Record<string, string> = {
			notion: 'Notion',
			obsidian: 'Obsidian',
			email_ingest: 'Email Forwarding'
		};
		return map[connection.provider] ?? connection.provider;
	}

	async function copyAddress(address: string, which: 'inbox' | 'feed') {
		if (!address) return;
		try {
			await navigator.clipboard.writeText(address);
			if (which === 'inbox') {
				copiedInbox = true;
				setTimeout(() => (copiedInbox = false), 2000);
			} else {
				copiedFeed = true;
				setTimeout(() => (copiedFeed = false), 2000);
			}
		} catch {
			// Clipboard is optional in non-browser test environments.
		}
	}

	function pollFetcher(jobId: string): () => Promise<{
		value?: ImportJobStatusResponse;
		error?: { httpStatus: number; message: string };
		terminal?: boolean;
	}> {
		return async () => {
			const result: ImportJobLookupResult = await fetchImportJob(jobId);
			if (result.status === 'ok') return { value: result.data };
			if (result.status === 'not_found') {
				return {
					error: { httpStatus: 404, message: 'Import job not found' },
					terminal: true
				};
			}
			return { error: { httpStatus: result.httpStatus, message: result.message } };
		};
	}

	function startPolling(jobId: string) {
		pollHandle?.stop();
		pollError = null;
		pollHandle = createPoll<ImportJobStatusResponse>({
			fetcher: pollFetcher(jobId),
			intervalMs: 2000,
			shouldStop: (result) => Boolean(result.value && !isPollingStatus(result.value.status)),
			onUpdate: (result) => {
				if (result.value) {
					activeJob = result.value;
					pollError = null;
				}
			},
			onError: (error) => {
				pollError = error.message;
			},
			onStop: () => {
				if (pollError) {
					pollError = `Lost contact with import job: ${pollError}. Refresh the page to retry.`;
				} else {
					void refreshHistory();
				}
			}
		});
		pollHandle.start();
	}

	async function hydrateJob(jobId: string, slot?: ImportSlot) {
		const lookup = await fetchImportJob(jobId);
		if (lookup.status !== 'ok') {
			pollError = lookup.status === 'not_found' ? 'Import job not found.' : lookup.message;
			return;
		}
		activeJob = lookup.data;
		activeSlot = slot ?? 'readwise';
		void refreshHistory();
		if (isPollingStatus(lookup.data.status)) startPolling(lookup.data.id);
	}

	async function pickReadwiseFiles(files: FileList) {
		const bundle: ReadwiseImportFiles = {};
		for (const file of Array.from(files)) {
			const lower = file.name.toLowerCase();
			if (lower.endsWith('.csv')) {
				if (bundle.libraryCsv)
					return setUploadError('Pick only one CSV — Readwise exports a single library CSV.');
				const text = await file
					.slice(0, CSV_HEADER_PROBE_BYTES)
					.text()
					.catch(() => '');
				const validation = validateReadwiseCsv(text);
				if (validation) return setUploadError(validation);
				bundle.libraryCsv = file;
			} else if (lower.endsWith('.zip')) {
				if (bundle.archiveZip) return setUploadError('Pick only one ZIP archive.');
				bundle.archiveZip = file;
			} else if (lower.endsWith('.opml') || lower.endsWith('.xml')) {
				if (bundle.feedsOpml) return setUploadError('Pick only one OPML/XML file.');
				bundle.feedsOpml = file;
			} else {
				return setUploadError(`Unsupported file: ${file.name}. Allowed: .csv, .zip, .opml, .xml`);
			}
		}
		if (!bundle.libraryCsv && !bundle.archiveZip && !bundle.feedsOpml) {
			return setUploadError('Pick at least one file.');
		}
		busySlot = 'readwise';
		uploadError = null;
		uploadErrorSlot = null;
		rollbackNotice = null;
		const result = await uploadReadwiseImportFiles(bundle);
		busySlot = null;
		if (!result.success) return setUploadError(result.error);
		await hydrateJob(result.data.import_job_id, 'readwise');
	}

	function setUploadError(message: string) {
		uploadError = message;
		uploadErrorSlot = 'readwise';
	}

	function handleReadwiseFileChange(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const files = input.files;
		if (!files?.[0]) return;
		void pickReadwiseFiles(files);
		input.value = '';
	}

	function handleReadwiseDrop(event: DragEvent) {
		event.preventDefault();
		dropHover = null;
		const files = event.dataTransfer?.files;
		if (!files?.[0]) return;
		void pickReadwiseFiles(files);
	}

	function clearActiveJob() {
		pollHandle?.stop();
		pollHandle = null;
		activeJob = null;
		activeSlot = null;
		pollError = null;
	}

	function openRollback(jobId: string) {
		rollbackTargetJobId = jobId;
		rollbackError = null;
		rollbackOpen = true;
	}

	async function confirmRollback() {
		const jobId = rollbackTargetJobId;
		if (!jobId) return;
		rollbackBusy = true;
		rollbackError = null;
		const result = await rollbackImportJob(jobId);
		rollbackBusy = false;
		if (!result.success) {
			rollbackError = result.error;
			return;
		}
		rollbackOpen = false;
		rollbackNotice = 'Rollback completed. Imported items were moved to Trash.';
		void refreshHistory();
		const lookup = await fetchImportJob(jobId);
		if (lookup.status === 'ok' && activeJob?.id === jobId) activeJob = lookup.data;
	}
</script>

<div class="settings-content">
	<IntegrationCallbackBanner
		{callback}
		onDismiss={dismissCallback}
		onAction={handleCallbackAction}
	/>

	<IntegrationsHero
		{heroState}
		{ringCounts}
		{ringDash}
		sevenDayItems={sevenDayItemCount}
		sevenDayDelta={sevenDayDeltaValue}
		onCopyInbox={() => copyAddress(inboxAddress, 'inbox')}
		onStartNotion={startNotionAuthorization}
	/>

	<div class="body-area">
		<ConnectionsSection
			{connectionsLoading}
			{connectionsError}
			{inboxAddress}
			{feedAddress}
			{copiedInbox}
			{copiedFeed}
			{extStore}
			{notionConnection}
			{obsidianConnection}
			{notionStatus}
			{obsidianStatus}
			{syncStateByConnection}
			{syncErrorByConnection}
			{notionConnectError}
			{notionAvailable}
			onCopyAddress={copyAddress}
			onStartNotion={startNotionAuthorization}
			onOpenNotion={openNotionDetail}
			onOpenObsidian={openObsidianDetail}
			onSync={handleSync}
			onDisconnect={openDisconnectDialog}
		/>

		<ReadwiseImportSection
			{activeJob}
			{activeSlot}
			{busySlot}
			{uploadError}
			{uploadErrorSlot}
			{pollError}
			{rollbackNotice}
			{dropHover}
			{readwiseUploadLimit}
			{activeIsTerminal}
			onDropHover={(slot) => (dropHover = slot)}
			onFileChange={handleReadwiseFileChange}
			onDrop={handleReadwiseDrop}
			onOpenRollback={openRollback}
			onClearActiveJob={clearActiveJob}
		/>

		<SettingsGroup title="Import history" meta="Most recent jobs">
			<ImportHistoryTable {history} onRollback={openRollback} />
		</SettingsGroup>
	</div>
</div>

<IntegrationDisconnectDialog
	open={disconnectTarget !== null}
	providerName={disconnectTarget ? disconnectProviderName(disconnectTarget) : ''}
	busy={disconnectBusy}
	errorMessage={disconnectError}
	onConfirm={confirmDisconnect}
	onCancel={closeDisconnectDialog}
/>

<ImportRollbackDialog
	open={rollbackOpen}
	busy={rollbackBusy}
	errorMessage={rollbackError}
	onCancel={() => {
		if (!rollbackBusy) {
			rollbackOpen = false;
			rollbackError = null;
			rollbackTargetJobId = null;
		}
	}}
	onConfirm={confirmRollback}
/>

<style>
	.settings-content {
		display: flex;
		flex-direction: column;
	}

	.body-area {
		padding: 32px 56px 48px;
		display: flex;
		flex-direction: column;
		max-width: 1080px;
		width: 100%;
		align-self: center;
		margin: 0 auto;
	}

	@media (max-width: 599px) {
		.body-area {
			padding: 20px 16px 32px;
		}
	}
</style>
