<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import FeedSubscriptionsTable from './FeedSubscriptionsTable.svelte';
	import type { Feed, FeedStats, FilterChip } from '../feed-model';

	interface Props {
		loading: boolean;
		feeds: Feed[];
		filteredFeeds: Feed[];
		stats: FeedStats;
		activeFilter: FilterChip;
		searchQuery: string;
		openKebabId: string | null;
		onAddFeed: () => void;
		onSearch: (query: string) => void;
		onFilter: (filter: FilterChip) => void;
		onToggleAutoSave: (id: string) => void;
		onToggleFeed: (id: string) => void;
		onToggleMenu: (id: string, event: MouseEvent) => void;
		onCloseMenu: () => void;
		onEdit: (id: string) => void;
		onRetry: (id: string) => void;
		onDelete: (id: string) => void;
	}

	let {
		loading,
		feeds,
		filteredFeeds,
		stats,
		activeFilter,
		searchQuery,
		openKebabId,
		onAddFeed,
		onSearch,
		onFilter,
		onToggleAutoSave,
		onToggleFeed,
		onToggleMenu,
		onCloseMenu,
		onEdit,
		onRetry,
		onDelete
	}: Props = $props();

	const filters: { value: FilterChip; label: string; count: () => number }[] = [
		{ value: 'all', label: 'All', count: () => stats.total },
		{ value: 'active', label: 'Active', count: () => stats.active },
		{ value: 'paused', label: 'Paused', count: () => stats.paused },
		{ value: 'error', label: 'Error', count: () => stats.error }
	];
</script>

<SettingsGroup
	title="All feeds"
	meta={loading
		? 'Loading…'
		: `${stats.total} source${stats.total === 1 ? '' : 's'} · sorted by most recent`}
>
	{#if loading}
		<div class="loading-state">
			<span class="loading-text">Loading subscriptions…</span>
		</div>
	{:else if feeds.length === 0}
		<div class="empty-state">
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<path d="M4 11a9 9 0 0 1 9 9" />
				<path d="M4 4a16 16 0 0 1 16 16" />
				<circle cx="5" cy="19" r="1" fill="currentColor" stroke="none" />
			</svg>
			<span class="es-title">No feeds yet</span>
			<span class="es-desc">
				Subscribe to RSS feeds to automatically receive new content in your library.
			</span>
			<button type="button" class="hero-btn primary" onclick={onAddFeed}>Add your first feed</button
			>
		</div>
	{:else}
		<div class="filter-bar">
			<div class="search-input-wrap">
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<circle cx="11" cy="11" r="7" />
					<path d="M21 21l-4.35-4.35" />
				</svg>
				<input
					class="search-input"
					type="search"
					placeholder="Search feeds by name or domain…"
					value={searchQuery}
					aria-label="Search feeds"
					oninput={(event) => onSearch(event.currentTarget.value)}
				/>
			</div>
			<div class="chip-row" role="tablist" aria-label="Filter feeds">
				{#each filters as filter (filter.value)}
					<button
						type="button"
						class="chip"
						class:active={activeFilter === filter.value}
						role="tab"
						aria-selected={activeFilter === filter.value}
						onclick={() => onFilter(filter.value)}
					>
						{filter.label} <span class="count">{filter.count()}</span>
					</button>
				{/each}
			</div>
		</div>

		{#if filteredFeeds.length === 0}
			<div class="search-empty">
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<circle cx="11" cy="11" r="7" />
					<path d="M21 21l-4.35-4.35" />
				</svg>
				<span class="search-empty-title">
					{searchQuery ? `No feeds match "${searchQuery}"` : `No ${activeFilter} feeds`}
				</span>
				<span class="search-empty-sub">Try a different filter or search term</span>
			</div>
		{:else}
			<FeedSubscriptionsTable
				feeds={filteredFeeds}
				{openKebabId}
				{onToggleAutoSave}
				{onToggleFeed}
				{onToggleMenu}
				{onCloseMenu}
				{onEdit}
				{onRetry}
				{onDelete}
			/>
		{/if}
	{/if}
</SettingsGroup>

<style>
	.filter-bar {
		display: flex;
		align-items: center;
		gap: 14px;
		background: var(--bg-elevated);
		padding: 10px 12px;
		border-radius: 14px;
		box-shadow: var(--feed-card-shadow);
		margin-bottom: 12px;
	}

	.search-input-wrap {
		position: relative;
		flex: 0 0 280px;
	}

	.search-input-wrap svg {
		position: absolute;
		left: 12px;
		top: 50%;
		transform: translateY(-50%);
		width: 14px;
		height: 14px;
		stroke: var(--text-tertiary);
		fill: none;
		stroke-width: 1.8;
		stroke-linecap: round;
		stroke-linejoin: round;
		pointer-events: none;
	}

	.search-input {
		width: 100%;
		padding: 8px 12px 8px 34px;
		border-radius: 10px;
		background: var(--bg-secondary);
		border: 0;
		box-shadow: var(--feed-input-shadow);
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-primary);
		outline: none;
		transition:
			box-shadow 150ms,
			background 150ms;
	}

	.search-input::-webkit-search-cancel-button {
		display: none;
	}

	.search-input:focus {
		box-shadow:
			inset 0 0 0 1.5px var(--feed-amber),
			0 0 0 4px var(--feed-amber-soft);
		background: var(--bg-primary);
	}

	.search-input::placeholder {
		color: var(--text-tertiary);
	}

	.chip-row {
		display: flex;
		gap: 6px;
		flex: 1;
		flex-wrap: wrap;
	}

	.chip {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 5px 11px;
		border-radius: 980px;
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
		background: var(--feed-chip-bg);
		box-shadow: inset 0 0 0 0.5px transparent;
		border: none;
		cursor: pointer;
		letter-spacing: 0;
		transition:
			background 140ms,
			color 140ms,
			box-shadow 140ms;
	}

	.chip:hover {
		color: var(--text-primary);
		background: var(--fill-hover);
	}

	.chip.active {
		color: var(--feed-chip-active-text);
		background: var(--feed-chip-active-bg);
		box-shadow: inset 0 0 0 0.5px var(--feed-chip-active-border);
		font-weight: 600;
	}

	.count {
		font-variant-numeric: tabular-nums;
		opacity: 0.65;
		font-weight: 500;
	}

	.chip.active .count {
		opacity: 0.85;
	}

	.loading-state {
		display: flex;
		justify-content: center;
		padding: 48px 0;
	}

	.loading-text {
		font-size: 14px;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
	}

	.empty-state,
	.search-empty {
		display: flex;
		flex-direction: column;
		align-items: center;
	}

	.empty-state {
		justify-content: center;
		padding: 60px 0;
		gap: 8px;
	}

	.empty-state svg {
		width: 32px;
		height: 32px;
		stroke: var(--text-tertiary);
		stroke-width: 1.5;
		fill: none;
		stroke-linecap: round;
		stroke-linejoin: round;
		margin-bottom: 4px;
	}

	.es-title {
		font-size: 17px;
		font-weight: 600;
		letter-spacing: 0;
		color: var(--text-primary);
		font-family: var(--font-sans);
	}

	.es-desc {
		font-size: 14px;
		color: var(--text-secondary);
		font-family: var(--font-sans);
		text-align: center;
		max-width: 320px;
		line-height: 1.5;
		margin-bottom: 8px;
	}

	.hero-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 9px 16px;
		border-radius: 980px;
		font-family: var(--font-sans);
		font-size: 13.5px;
		font-weight: 600;
		letter-spacing: 0;
		border: none;
		cursor: pointer;
		white-space: nowrap;
		transition:
			transform 140ms,
			box-shadow 140ms;
	}

	.hero-btn.primary {
		background: var(--feed-amber);
		color: var(--text-on-color);
		box-shadow: var(--feed-amber-shadow);
	}

	.hero-btn.primary:hover {
		transform: translateY(-1px);
		box-shadow: var(--feed-amber-shadow-hover);
	}

	.search-empty {
		gap: 4px;
		padding: 40px 0;
	}

	.search-empty svg {
		width: 24px;
		height: 24px;
		stroke: var(--text-quaternary);
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
		margin-bottom: 4px;
	}

	.search-empty-title {
		font-size: 14px;
		font-weight: 500;
		color: var(--text-secondary);
		font-family: var(--font-sans);
	}

	.search-empty-sub {
		font-size: 12px;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
	}

	@media (max-width: 760px) {
		.filter-bar {
			flex-direction: column;
			align-items: stretch;
		}

		.search-input-wrap {
			flex: 1;
		}
	}
</style>
