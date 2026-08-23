<script lang="ts">
	import type { SearchEntityCardResponse } from '$lib/api/generated/types.gen';
	import { date, t, type MessageKey } from '$lib/i18n';

	interface Props {
		entityCard: SearchEntityCardResponse;
		onFilter: (name: string) => void;
		filterActive?: boolean;
	}

	let { entityCard, onFilter, filterActive = false }: Props = $props();

	function formatDate(iso: string): string {
		return $date(new Date(iso), { month: 'short', year: 'numeric' });
	}

	function typeLabel(type: string): string {
		const labels: Record<string, MessageKey> = {
			person: 'search_entity_type_person',
			organization: 'search_entity_type_organization',
			location: 'search_entity_type_location',
			event: 'search_entity_type_event',
			work: 'search_entity_type_work'
		};
		const key = labels[type.toLowerCase()];
		return key ? $t(key) : type;
	}

	const normalizedType = $derived(entityCard.entity_type.toLowerCase());
</script>

<div class="entity-card">
	<div class="entity-type-icon {normalizedType}" aria-hidden="true">
		{#if normalizedType === 'person'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
				<circle cx="12" cy="7" r="4" />
			</svg>
		{:else if normalizedType === 'organization'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<rect x="2" y="7" width="20" height="14" rx="2" />
				<path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16" />
			</svg>
		{:else if normalizedType === 'location'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z" />
				<circle cx="12" cy="10" r="3" />
			</svg>
		{:else if normalizedType === 'event'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<rect x="3" y="4" width="18" height="18" rx="2" />
				<line x1="16" y1="2" x2="16" y2="6" />
				<line x1="8" y1="2" x2="8" y2="6" />
				<line x1="3" y1="10" x2="21" y2="10" />
			</svg>
		{:else}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" />
				<path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" />
			</svg>
		{/if}
	</div>

	<div class="entity-card-info">
		<div class="entity-card-name">
			{entityCard.name}
			<span class="entity-type-pill {normalizedType}">{typeLabel(entityCard.entity_type)}</span>
		</div>
		<div class="entity-card-stats">
			{$t('search_entity_mention_count', { values: { count: entityCard.mention_count } })}
		</div>
		<div class="entity-card-dates">
			{$t('search_entity_dates', {
				values: {
					first: formatDate(entityCard.first_seen_at),
					recent: formatDate(entityCard.last_seen_at)
				}
			})}
		</div>
	</div>

	{#if !filterActive}
		<button class="entity-filter-btn" type="button" onclick={() => onFilter(entityCard.name)}>
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
				aria-hidden="true"
			>
				<polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" />
			</svg>
			{$t('search_filter_by_entity')}
		</button>
	{/if}
</div>

<style>
	.entity-card {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 14px 20px;
		background: var(--bg-secondary);
		border-bottom: 0.5px solid var(--border-primary);
	}

	.entity-type-icon {
		width: 44px;
		height: 44px;
		border-radius: 12px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.entity-type-icon svg {
		width: 20px;
		height: 20px;
	}

	.entity-type-icon.person {
		background: var(--entity-person-bg);
		color: var(--entity-person-text);
	}

	.entity-type-icon.organization {
		background: var(--entity-org-bg);
		color: var(--entity-org-text);
	}

	.entity-type-icon.location {
		background: var(--entity-location-bg);
		color: var(--entity-location-text);
	}

	.entity-type-icon.event {
		background: var(--entity-event-bg);
		color: var(--entity-event-text);
	}

	.entity-type-icon.work {
		background: var(--entity-work-bg);
		color: var(--entity-work-text);
	}

	.entity-card-info {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 3px;
		min-width: 0;
	}

	.entity-card-name {
		display: flex;
		align-items: center;
		gap: 8px;
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
	}

	.entity-type-pill {
		display: inline-flex;
		align-items: center;
		padding: 2px 8px;
		border-radius: 980px;
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.02em;
		font-family: var(--font-sans);
		flex-shrink: 0;
	}

	.entity-type-pill.person {
		background: var(--entity-person-bg);
		color: var(--entity-person-text);
	}

	.entity-type-pill.organization {
		background: var(--entity-org-bg);
		color: var(--entity-org-text);
	}

	.entity-type-pill.location {
		background: var(--entity-location-bg);
		color: var(--entity-location-text);
	}

	.entity-type-pill.event {
		background: var(--entity-event-bg);
		color: var(--entity-event-text);
	}

	.entity-type-pill.work {
		background: var(--entity-work-bg);
		color: var(--entity-work-text);
	}

	.entity-card-stats {
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 400;
		color: var(--text-secondary);
	}

	.entity-card-dates {
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 400;
		color: var(--text-tertiary);
	}

	.entity-filter-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		border-radius: 980px;
		border: 1px solid var(--border-primary);
		background: transparent;
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
		cursor: pointer;
		flex-shrink: 0;
		transition: background 120ms ease;
	}

	.entity-filter-btn:hover {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.entity-filter-btn svg {
		width: 12px;
		height: 12px;
	}
</style>
