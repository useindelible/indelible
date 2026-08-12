<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import {
		disconnectIntegration,
		dispatchIntegrationSync,
		loadIntegrationConnections,
		loadNotionExportItems,
		loadNotionSettings,
		refreshNotionDocumentExport,
		saveNotionExportItems,
		saveNotionSettings,
		startIntegrationAuthorization
	} from '$lib/api/integrations';
	import type {
		IntegrationConnectionDto,
		NotionExportItemDto,
		NotionSettingsDto,
		UpdateNotionSettingsRequest
	} from '$lib/api';
	import NotionStatusPanel from '$lib/components/integrations/NotionStatusPanel.svelte';
	import { deriveConnectionState } from '$lib/integrations/status';
	import NotionConnectionLoader from './components/NotionConnectionLoader.svelte';
	import NotionDisconnectSection from './components/NotionDisconnectSection.svelte';
	import NotionHero from './components/NotionHero.svelte';
	import {
		formatNotionHeroLastSync,
		getNotionDatabaseLabel,
		getNotionHeroStatus,
		getNotionWorkspaceIcon,
		getNotionWorkspaceName
	} from './notion-route-model';

	let connection = $state<IntegrationConnectionDto | undefined>(undefined);
	let loading = $state(true);
	let loadError = $state<string | null>(null);
	let settings = $state<NotionSettingsDto | null>(null);
	let settingsError = $state<string | null>(null);
	let savingSetting = $state<keyof UpdateNotionSettingsRequest | null>(null);
	let exportItems = $state<NotionExportItemDto[]>([]);
	let exportItemsLoading = $state(false);
	let exportItemsError = $state<string | null>(null);
	let exportItemsQuery = $state('');
	let exportItemsHasNext = $state(false);
	let exportItemsTotal = $state(0);
	let exportItemsFilteredCount = $state(0);
	let savingItemId = $state<string | null>(null);
	let refreshingItemId = $state<string | null>(null);
	let refreshNotice = $state<{ message: string; archivedPageUrl?: string | null } | null>(null);
	let itemSearchTimer: ReturnType<typeof setTimeout> | undefined;
	const exportItemsLimit = 25;

	let syncing = $state(false);
	let syncError = $state<string | null>(null);

	let disconnectOpen = $state(false);
	let disconnectBusy = $state(false);
	let disconnectError = $state<string | null>(null);

	const connectionState = $derived(deriveConnectionState(connection));
	const workspaceName = $derived(getNotionWorkspaceName(connection));
	const workspaceIcon = $derived(getNotionWorkspaceIcon(connection));
	const databaseLabel = $derived(getNotionDatabaseLabel(workspaceName));
	const heroDocs = $derived(exportItemsTotal);
	const formattedHeroLastSync = $derived(formatNotionHeroLastSync(connection?.last_sync_at));
	const heroStatus = $derived(getNotionHeroStatus(connectionState));

	async function refresh() {
		loading = true;
		loadError = null;
		settingsError = null;
		const result = await loadIntegrationConnections();
		if (result.success) {
			connection = result.data.connections.find((candidate) => candidate.provider === 'notion');
			const available = result.data.available_oauth_providers ?? null;
			if (!connection && available !== null && !available.includes('notion')) {
				loadError =
					'Notion is not configured on this server. An administrator must set ' +
					'NOTION_CLIENT_ID, NOTION_CLIENT_SECRET, NOTION_REDIRECT_URL and ' +
					'AUTH_CREDENTIAL_KEY to enable it.';
				loading = false;
				return;
			}
			if (connection) {
				await loadNotionDetails(connection.id, 0);
			} else {
				settings = null;
				exportItems = [];
				exportItemsTotal = 0;
				exportItemsFilteredCount = 0;
				exportItemsHasNext = false;
			}
		} else {
			loadError = result.error;
		}
		loading = false;
	}

	async function loadNotionDetails(connectionId: string, offset = 0) {
		const [settingsResult] = await Promise.all([
			loadNotionSettings(connectionId),
			loadExportItems(offset, connectionId)
		]);
		if (settingsResult.success) {
			settings = settingsResult.data;
		} else {
			settingsError = settingsResult.error;
		}
	}

	onMount(() => {
		void refresh();
	});

	async function handleSync() {
		if (!connection) return;
		syncing = true;
		syncError = null;
		const result = await dispatchIntegrationSync(connection.id);
		syncing = false;
		if (result.success) {
			void refresh();
		} else {
			syncError = result.error;
		}
	}

	async function loadExportItems(offset = 0, connectionId = connection?.id, append = false) {
		if (!connectionId) return;
		exportItemsLoading = true;
		exportItemsError = null;
		const result = await loadNotionExportItems(connectionId, {
			q: exportItemsQuery.trim() || null,
			limit: exportItemsLimit,
			offset
		});
		exportItemsLoading = false;
		if (result.success) {
			exportItems = append ? [...exportItems, ...result.data.items] : result.data.items;
			exportItemsTotal = result.data.total_count;
			exportItemsFilteredCount = result.data.filtered_count;
			exportItemsHasNext = exportItems.length < result.data.filtered_count;
		} else {
			exportItemsError = result.error;
		}
	}

	function handleExportItemsSearch(value: string) {
		exportItemsQuery = value;
		if (itemSearchTimer) clearTimeout(itemSearchTimer);
		itemSearchTimer = setTimeout(() => {
			void loadExportItems(0, undefined, false);
		}, 250);
	}

	async function handleSettingChange(key: keyof UpdateNotionSettingsRequest, value: boolean) {
		if (!connection || !settings) return;
		const previous = settings;
		settings = { ...settings, [key]: value };
		savingSetting = key;
		settingsError = null;
		const result = await saveNotionSettings(connection.id, { [key]: value });
		savingSetting = null;
		if (result.success) {
			settings = result.data;
			if (key === 'selection_enabled') {
				await loadExportItems(0, undefined, false);
			}
			void refreshConnectionOnly();
		} else {
			settings = previous;
			settingsError = result.error;
		}
	}

	async function refreshConnectionOnly() {
		const result = await loadIntegrationConnections();
		if (result.success) {
			connection = result.data.connections.find((candidate) => candidate.provider === 'notion');
		}
	}

	async function handleExportItemSelection(libraryEntryId: string, selected: boolean) {
		if (!connection) return;
		const previous = exportItems;
		exportItems = exportItems.map((item) =>
			item.library_entry_id === libraryEntryId ? { ...item, selected } : item
		);
		savingItemId = libraryEntryId;
		exportItemsError = null;
		const result = await saveNotionExportItems(connection.id, {
			selections: [{ library_entry_id: libraryEntryId, selected }]
		});
		savingItemId = null;
		if (!result.success) {
			exportItems = previous;
			exportItemsError = result.error;
		}
	}

	async function handleRefreshItem(item: NotionExportItemDto) {
		if (!connection) return;
		const confirmed = window.confirm(
			`Archive the current Notion page for "${item.title || 'Untitled'}" and queue its replacement?`
		);
		if (!confirmed) return;
		refreshingItemId = item.library_entry_id;
		exportItemsError = null;
		refreshNotice = null;
		const result = await refreshNotionDocumentExport(connection.id, item.library_entry_id);
		refreshingItemId = null;
		if (result.success) {
			refreshNotice = {
				message: `Replacement queued for ${item.title || 'Untitled'}.`,
				archivedPageUrl: result.data.archived_page_url
			};
			await loadExportItems(0, undefined, false);
			await refreshConnectionOnly();
		} else {
			exportItemsError = result.error;
		}
	}

	async function handleChangeAccount() {
		const confirmed = window.confirm(
			'Change the connected Notion account? You will be sent through Notion authorization again.'
		);
		if (!confirmed) return;
		await handleAuthorize();
	}

	async function handleAuthorize() {
		const result = await startIntegrationAuthorization('notion');
		if (result.success) {
			window.location.href = result.data.authorize_url;
		} else {
			loadError = result.error;
		}
	}

	function openDisconnect() {
		disconnectError = null;
		disconnectOpen = true;
	}

	async function confirmDisconnect() {
		if (!connection) return;
		disconnectBusy = true;
		disconnectError = null;
		const result = await disconnectIntegration(connection.id);
		disconnectBusy = false;
		if (result.success) {
			disconnectOpen = false;
			void goto(resolve('/preferences/integrations'));
		} else {
			disconnectError = result.error;
		}
	}
</script>

<div class="page">
	<NotionHero
		{heroDocs}
		{formattedHeroLastSync}
		{workspaceIcon}
		{workspaceName}
		{databaseLabel}
		{connectionState}
		{heroStatus}
		pendingJobs={connection?.pending_jobs ?? 0}
	/>

	<NotionConnectionLoader {loading} {loadError} onRetry={refresh}>
		<NotionStatusPanel
			{connection}
			{syncing}
			{syncError}
			{settings}
			{settingsError}
			{savingSetting}
			items={exportItems}
			itemsTotal={exportItemsTotal}
			itemsFilteredCount={exportItemsFilteredCount}
			itemsLoading={exportItemsLoading}
			itemsError={exportItemsError}
			itemsQuery={exportItemsQuery}
			itemsHasNext={exportItemsHasNext}
			{savingItemId}
			{refreshingItemId}
			{refreshNotice}
			onSync={handleSync}
			onReauthorize={handleAuthorize}
			onChangeAccount={handleChangeAccount}
			onDisconnect={openDisconnect}
			onSettingChange={handleSettingChange}
			onItemsSearch={handleExportItemsSearch}
			onItemsLoadMore={() => loadExportItems(exportItems.length, undefined, true)}
			onItemSelection={handleExportItemSelection}
			onRefreshItem={handleRefreshItem}
		/>
	</NotionConnectionLoader>
</div>

<NotionDisconnectSection
	open={disconnectOpen}
	busy={disconnectBusy}
	errorMessage={disconnectError}
	onCancel={() => {
		if (!disconnectBusy) disconnectOpen = false;
	}}
	onConfirm={confirmDisconnect}
/>

<style>
	.page {
		display: flex;
		flex-direction: column;
		--accent: var(--notion-accent);
		--accent-hover: var(--notion-accent-strong);
		--accent-soft: var(--notion-accent-soft);
	}
</style>
