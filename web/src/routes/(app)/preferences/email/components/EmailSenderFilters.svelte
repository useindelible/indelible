<script lang="ts">
	import type { SenderCounts, SenderFilter } from '../email-model';
	import { t, type MessageKey } from '$lib/i18n';

	interface Props {
		activeFilter: SenderFilter;
		search: string;
		counts: SenderCounts;
		onFilter: (filter: SenderFilter) => void;
		onSearch: (value: string) => void;
	}

	let { activeFilter, search, counts, onFilter, onSearch }: Props = $props();

	const filters: Array<{ key: SenderFilter; labelKey: MessageKey }> = [
		{ key: 'all', labelKey: 'common_all' },
		{ key: 'feed', labelKey: 'common_feed' },
		{ key: 'library', labelKey: 'common_library' },
		{ key: 'blocked', labelKey: 'email_blocked' },
		{ key: 'quiet', labelKey: 'email_quiet_30d' }
	];
</script>

<div class="filter-row">
	<label class="search-wrap" aria-label={$t('email_search_senders')}>
		<svg viewBox="0 0 24 24" class="search-icon" aria-hidden="true">
			<circle cx="11" cy="11" r="7" />
			<path d="M21 21l-4.35-4.35" />
		</svg>
		<input
			class="search-input"
			type="search"
			placeholder={$t('email_search_placeholder')}
			value={search}
			oninput={(event) => onSearch(event.currentTarget.value)}
		/>
	</label>
	<div class="chip-row" role="tablist" aria-label={$t('email_filter_senders')}>
		{#each filters as filter (filter.key)}
			<button
				type="button"
				class="chip"
				class:active={activeFilter === filter.key}
				role="tab"
				aria-selected={activeFilter === filter.key}
				onclick={() => onFilter(filter.key)}
			>
				{$t(filter.labelKey)} <span class="count">{counts[filter.key]}</span>
			</button>
		{/each}
	</div>
</div>

<style>
	.filter-row {
		display: flex;
		align-items: center;
		gap: 12px;
		flex-wrap: wrap;
	}

	.search-wrap {
		position: relative;
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 0 12px 0 36px;
		border-radius: 8px;
		background: var(--bg-secondary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		flex: 1;
		min-width: 220px;
		max-width: 360px;
		height: 34px;
		transition:
			box-shadow 150ms ease,
			background 150ms ease;
	}

	.search-wrap:focus-within {
		background: var(--bg-elevated);
		box-shadow:
			inset 0 0 0 1.5px var(--accent),
			0 0 0 4px var(--accent-soft);
	}

	.search-icon {
		position: absolute;
		left: 12px;
		top: 50%;
		transform: translateY(-50%);
		width: 14px;
		height: 14px;
		stroke: var(--text-tertiary);
		fill: none;
		stroke-width: 1.7;
		stroke-linecap: round;
		stroke-linejoin: round;
		flex-shrink: 0;
		pointer-events: none;
	}

	.search-input {
		flex: 1;
		background: transparent;
		border: none;
		outline: none;
		font: inherit;
		font-size: 13px;
		color: var(--text-primary);
		letter-spacing: -0.005em;
	}

	.search-input::placeholder {
		color: var(--text-tertiary);
	}

	.chip-row {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		flex-wrap: wrap;
	}

	.chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 5px 11px;
		border-radius: var(--radius-full);
		background: var(--chip-bg);
		color: var(--text-secondary);
		font-size: 12px;
		font-weight: 500;
		letter-spacing: -0.005em;
		border: none;
		box-shadow: inset 0 0 0 0.5px transparent;
		cursor: pointer;
		transition:
			background 140ms ease,
			color 140ms ease,
			box-shadow 140ms ease;
	}

	.chip:hover {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.chip.active {
		background: var(--chip-active-bg);
		color: var(--chip-active-text);
		box-shadow: inset 0 0 0 0.5px var(--chip-active-border);
		font-weight: 600;
	}

	.count {
		font-family: var(--font-mono);
		font-size: 10.5px;
		font-variant-numeric: tabular-nums;
		color: inherit;
		opacity: 0.65;
	}

	.chip.active .count {
		opacity: 0.85;
	}
</style>
