<script lang="ts">
	import { resolve } from '$app/paths';
	import * as apiSdk from '$lib/api';
	import type { EntitySummaryResponse } from '$lib/api/generated/types.gen';
	import { t, type MessageKey } from '$lib/i18n';
	import { SvelteMap } from 'svelte/reactivity';
	import { addDomainEventHandler } from '$lib/realtime/domain-events';
	import { READER_AI_DOMAIN_EVENT_TYPES } from '$lib/realtime/event-types';

	interface Props {
		itemId: string;
	}

	let { itemId }: Props = $props();

	type EntityKind = 'person' | 'organization' | 'location' | 'event' | 'topic';

	interface EntityGroup {
		labelKey: MessageKey;
		kind: EntityKind;
		items: EntitySummaryResponse[];
	}

	const KIND_ORDER: EntityKind[] = ['person', 'organization', 'location', 'event', 'topic'];
	const KIND_LABELS: Record<EntityKind, MessageKey> = {
		person: 'library_entities_people',
		organization: 'library_entities_organizations',
		location: 'library_entities_locations',
		event: 'library_entities_events',
		topic: 'library_entities_topics'
	};

	let entities = $state<EntitySummaryResponse[]>([]);
	let loading = $state(true);
	let failure = $state<string | null>(null);
	const aiEvents = new Set<string>(READER_AI_DOMAIN_EVENT_TYPES);

	async function loadEntities() {
		if (!itemId) return;
		loading = true;
		try {
			const { data } = await apiSdk.listDocumentEntities({ path: { document_id: itemId } });
			entities = data ?? [];
		} catch {
			entities = [];
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (!itemId) return;
		void loadEntities();
		const unsubscribe = addDomainEventHandler((event) => {
			if (!aiEvents.has(event.type)) return;
			const payload = event.payload as {
				document_id?: unknown;
				action?: unknown;
			};
			if (payload.document_id !== itemId) return;
			if (payload.action !== 'entities') return;
			if (event.type === 'ai.output.failed') {
				failure = $t('library_entities_error');
				return;
			}
			failure = null;
			void loadEntities();
		});
		return unsubscribe;
	});

	const groups = $derived.by((): EntityGroup[] => {
		const byKind = new SvelteMap<EntityKind, EntitySummaryResponse[]>();
		for (const entity of entities) {
			const kind = (
				KIND_ORDER.includes(entity.entity_type as EntityKind) ? entity.entity_type : 'topic'
			) as EntityKind;
			const existing = byKind.get(kind) ?? [];
			existing.push(entity);
			byKind.set(kind, existing);
		}
		return KIND_ORDER.filter((kind) => byKind.has(kind)).map((kind) => ({
			labelKey: KIND_LABELS[kind],
			kind,
			items: byKind.get(kind)!
		}));
	});
</script>

<div class="entities-section">
	<div class="entities-header">
		<div class="section-heading">{$t('library_entities_title')}</div>
		<p class="section-subtext">{$t('library_entities_extracted_by_mila')}</p>
	</div>

	{#if loading}
		<div class="entities-loading">{$t('common_loading')}</div>
	{:else if groups.length === 0}
		<p class="entities-empty">{failure ?? $t('library_entities_empty')}</p>
	{:else}
		{#each groups as group (group.kind)}
			<div class="entity-group">
				<div class="entity-group-header">
					{#if group.kind === 'person'}
						<svg
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.5"
							stroke-linecap="round"
							stroke-linejoin="round"
							aria-hidden="true"
						>
							<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
							<circle cx="12" cy="7" r="4" />
						</svg>
					{:else if group.kind === 'organization'}
						<svg
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.5"
							stroke-linecap="round"
							stroke-linejoin="round"
							aria-hidden="true"
						>
							<rect x="4" y="2" width="16" height="20" rx="2" ry="2" />
							<line x1="9" y1="6" x2="9" y2="6.01" />
							<line x1="15" y1="6" x2="15" y2="6.01" />
							<line x1="9" y1="10" x2="9" y2="10.01" />
							<line x1="15" y1="10" x2="15" y2="10.01" />
							<line x1="9" y1="14" x2="9" y2="14.01" />
							<line x1="15" y1="14" x2="15" y2="14.01" />
							<line x1="9" y1="18" x2="15" y2="18" />
						</svg>
					{:else if group.kind === 'location'}
						<svg
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.5"
							stroke-linecap="round"
							stroke-linejoin="round"
							aria-hidden="true"
						>
							<path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z" />
							<circle cx="12" cy="10" r="3" />
						</svg>
					{:else if group.kind === 'event'}
						<svg
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.5"
							stroke-linecap="round"
							stroke-linejoin="round"
							aria-hidden="true"
						>
							<rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
							<line x1="16" y1="2" x2="16" y2="6" />
							<line x1="8" y1="2" x2="8" y2="6" />
							<line x1="3" y1="10" x2="21" y2="10" />
						</svg>
					{:else}
						<svg
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.5"
							stroke-linecap="round"
							stroke-linejoin="round"
							aria-hidden="true"
						>
							<circle cx="12" cy="12" r="10" />
							<line x1="12" y1="8" x2="12" y2="12" />
							<line x1="12" y1="16" x2="12.01" y2="16" />
						</svg>
					{/if}
					{$t(group.labelKey)}
				</div>
				{#each group.items as entity (entity.id)}
					<div class="entity-row">
						<a class="entity-name" href={resolve('/(app)/entities/[slug]', { slug: entity.id })}
							>{entity.name}</a
						>
						{#if entity.item_count > 1}
							<span class="entity-badge"
								>{$t('library_entities_other_documents', {
									values: { count: entity.item_count - 1 }
								})}</span
							>
						{/if}
					</div>
				{/each}
			</div>
		{/each}
	{/if}
</div>

<style>
	.entities-section {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.entities-header {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.section-heading {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		line-height: 1.2;
	}

	.section-subtext {
		font-size: 11px;
		font-weight: 400;
		color: var(--text-tertiary);
		font-style: italic;
		margin: 0;
		letter-spacing: -0.005em;
		line-height: 1.2;
	}

	.entities-loading,
	.entities-empty {
		font-size: 12px;
		color: var(--text-tertiary);
		font-style: italic;
		margin: 4px 0 0;
		padding: 0;
	}

	.entity-group {
		display: flex;
		flex-direction: column;
		gap: 0;
	}

	.entity-group-header {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 0 4px;
		font-size: 11px;
		font-weight: 500;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-secondary);
		line-height: 1.2;
	}

	.entity-group-header svg {
		width: 14px;
		height: 14px;
		flex-shrink: 0;
		color: var(--text-tertiary);
	}

	.entity-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 7px 4px 7px 20px;
		border-bottom: 0.5px solid var(--border-primary);
		border-radius: 4px;
		margin: 0 -4px;
		transition: background 120ms ease;
	}

	.entity-row:last-child {
		border-bottom: none;
	}

	.entity-row:hover {
		background: var(--fill-hover);
	}

	.entity-name {
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--accent);
		cursor: pointer;
		text-decoration: none;
		transition: color 150ms ease;
		line-height: 1.45;
		flex: 1;
		min-width: 0;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.entity-name:hover {
		text-decoration: underline;
		color: var(--accent-hover);
	}

	.entity-badge {
		font-size: 11px;
		font-weight: 400;
		letter-spacing: -0.005em;
		color: var(--text-tertiary);
		background: var(--fill-hover);
		padding: 2px 8px;
		border-radius: 980px;
		white-space: nowrap;
		line-height: 1.4;
		flex-shrink: 0;
	}
</style>
