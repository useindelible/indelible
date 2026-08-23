<script lang="ts">
	import type {
		IntegrationConnectionDto,
		NotionExportItemDto,
		NotionSettingsDto,
		UpdateNotionSettingsRequest
	} from '$lib/api';
	import NotionConnectionSummary from './notion-status/NotionConnectionSummary.svelte';
	import NotionDangerZone from './notion-status/NotionDangerZone.svelte';
	import NotionEmptyState from './notion-status/NotionEmptyState.svelte';
	import NotionItemSelectionTable from './notion-status/NotionItemSelectionTable.svelte';
	import NotionSettingsControls from './notion-status/NotionSettingsControls.svelte';
	import NotionSyncSummary from './notion-status/NotionSyncSummary.svelte';
	import {
		notionConnectionDetails,
		notionExportItemsMeta,
		notionStatusSummary,
		selectedExportItemCount
	} from './notion-status/notion-status-model';
	import { t } from '$lib/i18n';

	interface Props {
		connection: IntegrationConnectionDto | undefined;
		syncing?: boolean;
		syncError?: string | null;
		settings?: NotionSettingsDto | null;
		settingsError?: string | null;
		savingSetting?: keyof UpdateNotionSettingsRequest | null;
		items?: NotionExportItemDto[];
		itemsTotal?: number;
		itemsFilteredCount?: number;
		itemsLoading?: boolean;
		itemsError?: string | null;
		itemsQuery?: string;
		itemsHasNext?: boolean;
		savingItemId?: string | null;
		refreshingItemId?: string | null;
		refreshNotice?: { message: string; archivedPageUrl?: string | null } | null;
		onSync: () => void;
		onReauthorize: () => void;
		onChangeAccount: () => void;
		onDisconnect: () => void;
		onSettingChange: (key: keyof UpdateNotionSettingsRequest, value: boolean) => void;
		onItemsSearch: (query: string) => void;
		onItemsLoadMore?: () => void;
		onItemSelection: (itemId: string, selected: boolean) => void;
		onRefreshItem: (item: NotionExportItemDto) => void;
	}

	let {
		connection,
		syncing = false,
		syncError = null,
		settings = null,
		settingsError = null,
		savingSetting = null,
		items = [],
		itemsTotal = 0,
		itemsFilteredCount = 0,
		itemsLoading = false,
		itemsError = null,
		itemsQuery = '',
		itemsHasNext = false,
		savingItemId = null,
		refreshingItemId = null,
		refreshNotice = null,
		onSync,
		onReauthorize,
		onChangeAccount,
		onDisconnect,
		onSettingChange,
		onItemsSearch,
		onItemsLoadMore = () => {},
		onItemSelection,
		onRefreshItem
	}: Props = $props();

	const details = $derived(notionConnectionDetails(connection));
	const summary = $derived(notionStatusSummary(connection, $t));
	const selectedCount = $derived(selectedExportItemCount(items));
	const exportItemsMeta = $derived(
		notionExportItemsMeta(items.length, itemsTotal, itemsFilteredCount, itemsQuery, $t)
	);
</script>

{#if !connection}
	<NotionEmptyState {onReauthorize} />
{:else}
	<div class="panel" data-testid="notion-status-panel" data-state={summary.connectionState}>
		<NotionConnectionSummary {details} {summary} {onReauthorize} {onChangeAccount} />
		<NotionSyncSummary {connection} {summary} {syncing} {syncError} {onSync} {onReauthorize} />
		<NotionSettingsControls {settings} {savingSetting} {onSettingChange} />
		<NotionItemSelectionTable
			visible={settings?.selection_enabled}
			{items}
			{itemsLoading}
			{itemsError}
			{itemsQuery}
			{itemsHasNext}
			{savingItemId}
			{refreshingItemId}
			{refreshNotice}
			{selectedCount}
			{exportItemsMeta}
			{onItemsSearch}
			{onItemsLoadMore}
			{onItemSelection}
			{onRefreshItem}
		/>
		<NotionDangerZone {onDisconnect} />

		{#if settingsError}
			<div class="settings-error" role="alert">{settingsError}</div>
		{/if}
	</div>
{/if}

<style>
	.panel {
		display: flex;
		flex-direction: column;
	}

	.settings-error {
		margin-top: -12px;
		font-size: 12px;
		color: var(--destructive);
	}
</style>
