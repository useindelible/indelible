<script lang="ts">
	import { untrack } from 'svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import * as apiSdk from '$lib/api';
	import type { FeedDeliveryResponse, DocumentListEntry } from '$lib/api';
	import type { TriageTab } from '$lib/stores/library.svelte';
	import { getViewport } from '$lib/stores/viewport.svelte';
	import ItemList from '$lib/components/library/ItemList.svelte';
	import MorphSwitcher from '$lib/components/ui/MorphSwitcher.svelte';
	import DetailPanel from '$lib/components/library/DetailPanel.svelte';

	type StateFilter = 'unseen' | 'seen';

	const LIMIT = 40;

	const vp = getViewport();

	// Below the desktop breakpoint the docked detail panel becomes a slide-over
	// (tablet) or full-screen view (mobile); session-only, matching the library list.
	let compactDetailOpen = $state(false);

	let stateFilter = $state<StateFilter>('unseen');
	let items = $state<FeedDeliveryResponse[]>([]);
	let loading = $state(false);
	let loadingMore = $state(false);
	let hasMore = $state(false);
	let lastCursor = $state<string | null>(null);
	let selectedId = $state<string | null>(null);
	let markingAllSeen = $state(false);

	const stateOptions = [
		{ value: 'unseen' as StateFilter, label: 'Unseen' },
		{ value: 'seen' as StateFilter, label: 'Seen' }
	] as const;

	$effect(() => {
		const filter = stateFilter;
		untrack(() => load(true, filter));
	});

	async function load(reset: boolean, filter: StateFilter) {
		if (reset) {
			loading = true;
			items = [];
			lastCursor = null;
			selectedId = null;
		} else {
			loadingMore = true;
		}

		const { data } = await apiSdk.listFeedDeliveries({
			query: {
				state: filter,
				cursor: reset ? null : lastCursor,
				limit: LIMIT
			}
		});

		if (data) {
			const incoming: FeedDeliveryResponse[] = data.data ?? [];
			items = reset ? incoming : [...items, ...incoming];
			hasMore = data.page?.has_more ?? incoming.length >= LIMIT;
			lastCursor = data.page?.next_cursor ?? null;
		}

		loading = false;
		loadingMore = false;
	}

	function domainOf(url: string | null | undefined): string | null {
		if (!url) return null;
		try {
			return new URL(url).hostname.replace(/^www\./, '');
		} catch {
			return null;
		}
	}

	// Map feed deliveries to DocumentListEntry for the shared list/detail components. A delivery is
	// not a library item; `document_id` may be null (unprepared) and required fields use feed
	// defaults. The row stays usable for preview, mark-seen, save, and external-open (AC #6).
	function toDisplayItem(fd: FeedDeliveryResponse): DocumentListEntry {
		const subjectId = fd.document_id ?? fd.delivery_id;
		const documentType = fd.document_type ?? 'article';
		return {
			id: fd.delivery_id,
			document_id: subjectId,
			document_type: documentType,
			library_entry_id: null,
			object: 'feed_delivery',
			title: fd.title,
			url: fd.url ?? null,
			author: fd.author ?? null,
			excerpt: fd.excerpt ?? null,
			published_at: fd.published_at ?? null,
			saved_at: fd.delivered_at,
			created_at: fd.delivered_at,
			updated_at: fd.delivered_at,
			source: 'feed',
			item_type: documentType as DocumentListEntry['item_type'],
			triage_state: 'inbox',
			is_favorite: false,
			is_shortlisted: false,
			domain: domainOf(fd.url),
			lead_image_url: fd.lead_image_url ?? null,
			thumbnail_url: fd.thumbnail_url ?? null,
			canonical_url: fd.url ?? null,
			deleted_at: null,
			language: null,
			last_read_at: null,
			pipeline_error: null,
			pipeline_status: null,
			progress_percent: null,
			reading_time_minutes: null,
			readable_ready: fd.document_id != null,
			saved: fd.saved,
			word_count: null
		};
	}

	const displayItems = $derived(items.map(toDisplayItem));
	const selectedItem = $derived(displayItems.find((i) => i.id === selectedId) ?? null);
	const isEmpty = $derived(!loading && items.length === 0);

	function openItemDetail(id: string) {
		selectedId = id;
		compactDetailOpen = true;
	}

	async function handleMarkAllSeen() {
		markingAllSeen = true;
		await apiSdk.markAllDeliveriesSeen({ body: {} });
		untrack(() => load(true, stateFilter));
		markingAllSeen = false;
	}

	// Triage actions from ItemRow are repurposed for feed deliveries:
	//   later  → save to Library via the delivery (bookmark icon = save for later)
	//   archive → mark the delivery seen (archive = dismiss from Unseen)
	//   inbox  → no-op (already the default state for unseen deliveries)
	async function handleTriage(deliveryId: string, action: TriageTab) {
		if (action === 'later') {
			await apiSdk.saveFromDelivery({ body: { delivery_id: deliveryId } });
			// Saving hides the delivery from Feed.
			items = items.filter((i) => i.delivery_id !== deliveryId);
		} else if (action === 'archive') {
			await apiSdk.markDeliverySeen({ path: { delivery_id: deliveryId } });
			if (stateFilter === 'unseen') {
				items = items.filter((i) => i.delivery_id !== deliveryId);
			}
		}
	}

	// Phase 7: opening a delivery opens the canonical in-app reader. prepareFeedDelivery is
	// idempotent and always called (a delivery can carry a document_id while its readable content
	// is still missing or rendering), materializes/adopts the document, marks the delivery seen,
	// and enqueues the render. We then navigate to the reader, which polls until the
	// prepared content lands. (A delivery with no canonical URL returns 422 and stays in Feed.)
	async function handleOpen(deliveryId: string) {
		selectedId = deliveryId;
		try {
			const { data } = await apiSdk.prepareFeedDelivery({ path: { delivery_id: deliveryId } });
			if (stateFilter === 'unseen') {
				items = items.filter((item) => item.delivery_id !== deliveryId);
			}
			if (data?.document_id) {
				await goto(resolve('/(app)/reader/[documentId]', { documentId: data.document_id }));
			}
		} catch {
			// Preparation failed (e.g. a no-URL delivery); leave the delivery in place.
		}
	}
</script>

<div class="list-panel">
	<div class="list-header">
		<button
			type="button"
			class="menu-btn"
			onclick={() => vp.openMobileNav()}
			aria-label="Open navigation"
		>
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.7"
				stroke-linecap="round"
				stroke-linejoin="round"
				aria-hidden="true"
			>
				<line x1="3" y1="6" x2="21" y2="6" />
				<line x1="3" y1="12" x2="21" y2="12" />
				<line x1="3" y1="18" x2="21" y2="18" />
			</svg>
		</button>
		<div class="feed-title-row">
			<svg class="feed-icon" viewBox="0 0 24 24" aria-hidden="true">
				<path d="M4 11a9 9 0 0 1 9 9" />
				<path d="M4 4a16 16 0 0 1 16 16" />
				<circle cx="5" cy="19" r="1" fill="currentColor" stroke="none" />
			</svg>
			<span class="feed-title">Feed</span>
		</div>

		<div class="header-right">
			<MorphSwitcher
				options={stateOptions}
				value={stateFilter}
				onchange={(v) => (stateFilter = v as StateFilter)}
			/>
			{#if stateFilter === 'unseen'}
				<div class="tool-divider"></div>
				<button
					type="button"
					class="mark-all-btn"
					disabled={markingAllSeen || loading || items.length === 0}
					onclick={handleMarkAllSeen}
				>
					{markingAllSeen ? 'Marking…' : 'Mark all seen'}
				</button>
			{/if}
			{#if vp.isCompact && !vp.isMobile}
				<button
					type="button"
					class="panel-toggle"
					class:active={compactDetailOpen}
					onclick={() => (compactDetailOpen = !compactDetailOpen)}
					aria-label={compactDetailOpen ? 'Hide detail panel' : 'Show detail panel'}
					title={compactDetailOpen ? 'Hide detail panel' : 'Show detail panel'}
				>
					<svg
						viewBox="0 0 20 20"
						fill="none"
						stroke="currentColor"
						stroke-width="1.5"
						stroke-linecap="round"
						stroke-linejoin="round"
						aria-hidden="true"
					>
						<rect x="3" y="4" width="14" height="12" rx="1.5" />
						<line x1="13" y1="4" x2="13" y2="16" />
					</svg>
				</button>
			{/if}
		</div>
	</div>

	<div class="list-body">
		<ItemList
			items={displayItems}
			{loading}
			{loadingMore}
			{hasMore}
			{isEmpty}
			{selectedId}
			triageTab="inbox"
			showFeedBadge={true}
			onLoadMore={() => load(false, stateFilter)}
			onSelect={(id) => {
				selectedId = id;
			}}
			onOpen={handleOpen}
			onTriage={(id, state) => handleTriage(id, state)}
			onDetail={vp.isMobile ? openItemDetail : undefined}
		/>
	</div>
</div>

{#if vp.isCompact}
	{#if compactDetailOpen}
		{#if vp.isMobile}
			<div class="m-detail">
				<div class="m-detailbar">
					<button
						type="button"
						class="m-back"
						onclick={() => (compactDetailOpen = false)}
						aria-label="Back to list"
					>
						<svg
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
							aria-hidden="true"
						>
							<polyline points="15 18 9 12 15 6" />
						</svg>
					</button>
					<span class="m-dtitle">{selectedItem?.title ?? 'Details'}</span>
				</div>
				<DetailPanel item={selectedItem} />
			</div>
		{:else}
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="detail-scrim" onclick={() => (compactDetailOpen = false)}></div>
			<div class="detail-overlay">
				<DetailPanel item={selectedItem} />
			</div>
		{/if}
	{/if}
{:else}
	<DetailPanel item={selectedItem} />
{/if}

<style>
	.list-panel {
		flex: 1;
		display: flex;
		flex-direction: column;
		background: var(--bg-content);
		min-width: 0;
		overflow: hidden;
	}

	.list-header {
		padding: 0 20px;
		height: 60px;
		border-bottom: 0.5px solid var(--border-primary);
		display: flex;
		flex-direction: row;
		align-items: center;
		gap: 12px;
		flex-shrink: 0;
	}

	.feed-title-row {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-shrink: 0;
	}

	.feed-icon {
		width: 20px;
		height: 20px;
		stroke: var(--orange);
		fill: none;
		stroke-width: 1.75;
		stroke-linecap: round;
		stroke-linejoin: round;
		flex-shrink: 0;
	}

	.feed-title {
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
	}

	.header-right {
		margin-left: auto;
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.tool-divider {
		width: 0.5px;
		height: 20px;
		background: var(--border-secondary);
		margin: 0 6px;
		flex-shrink: 0;
	}

	.mark-all-btn {
		padding: 5px 12px;
		border-radius: 7px;
		border: 1px solid var(--border-secondary);
		background: none;
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
		cursor: pointer;
		white-space: nowrap;
		transition:
			background 120ms ease,
			color 120ms ease;
	}

	.mark-all-btn:hover:not(:disabled) {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.mark-all-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.list-body {
		flex: 1;
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}

	/* ---- Responsive: tablet slide-over + mobile reflow ---- */

	.menu-btn {
		display: none;
		width: 34px;
		height: 34px;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		flex-shrink: 0;
	}

	.menu-btn:hover {
		background: var(--fill-hover);
	}

	.menu-btn svg {
		width: 20px;
		height: 20px;
	}

	.panel-toggle {
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-tertiary);
		cursor: pointer;
		transition:
			background 0.12s ease,
			color 0.12s ease;
		flex-shrink: 0;
	}

	.panel-toggle svg {
		width: 18px;
		height: 18px;
	}

	.panel-toggle:hover {
		background: var(--fill-hover);
		color: var(--text-secondary);
	}

	.panel-toggle.active {
		color: var(--accent);
		background: var(--fill-selected);
	}

	.detail-scrim {
		position: absolute;
		inset: 0;
		background: rgba(0, 0, 0, 0.1);
		z-index: 20;
	}

	/* Opaque surface: the docked panel's vibrancy blur would let the list rows
	   bleed through when it floats above them. */
	.detail-overlay {
		position: absolute;
		top: 0;
		right: 0;
		bottom: 0;
		width: 330px;
		z-index: 21;
		display: flex;
		background: var(--bg-elevated);
		box-shadow: -18px 0 56px rgba(0, 0, 0, 0.18);
	}

	.detail-overlay :global(.detail-panel) {
		width: 100%;
		min-width: 0;
		background: var(--bg-elevated);
		backdrop-filter: none;
		-webkit-backdrop-filter: none;
	}

	.m-detail {
		position: absolute;
		inset: 0;
		z-index: 21;
		display: flex;
		flex-direction: column;
		background: var(--bg-content);
	}

	.m-detailbar {
		height: 52px;
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 0 8px;
		border-bottom: 0.5px solid var(--border-primary);
		flex-shrink: 0;
		background: var(--bg-content);
	}

	.m-back {
		width: 34px;
		height: 34px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		flex-shrink: 0;
	}

	.m-back:hover {
		background: var(--fill-hover);
	}

	.m-back svg {
		width: 20px;
		height: 20px;
	}

	.m-dtitle {
		font-size: 15px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		min-width: 0;
	}

	.m-detail :global(.detail-panel) {
		width: 100%;
		min-width: 0;
		flex: 1;
		border-left: none;
		background: var(--bg-content);
		backdrop-filter: none;
		-webkit-backdrop-filter: none;
	}

	@media (max-width: 599px) {
		.menu-btn {
			display: flex;
		}

		.list-header {
			flex-wrap: wrap;
			height: auto;
			padding: 8px 16px;
			row-gap: 4px;
		}

		.header-right {
			width: 100%;
			margin-left: 0;
		}

		/* Switcher stays left, "Mark all seen" hugs the right edge; the divider
		   has no room in the wrapped row. */
		.tool-divider {
			display: none;
		}

		.header-right > .mark-all-btn {
			margin-left: auto;
		}
	}
</style>
