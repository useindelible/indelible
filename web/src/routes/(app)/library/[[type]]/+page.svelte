<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { untrack } from 'svelte';
	import {
		getLibrary,
		triageOptionsForMode,
		type TriageTab,
		type ReadStatusTab
	} from '$lib/stores/library.svelte';
	import { getSmartLists } from '$lib/stores/smart-lists.svelte';
	import { getViewport } from '$lib/stores/viewport.svelte';
	import ItemList from '$lib/components/library/ItemList.svelte';
	import MorphSwitcher from '$lib/components/ui/MorphSwitcher.svelte';
	import DetailPanel from '$lib/components/library/DetailPanel.svelte';
	import SortDropdown from '$lib/components/library/SortDropdown.svelte';
	import FilterBar from '$lib/components/library/FilterBar.svelte';
	import SaveViewModal from '$lib/components/library/SaveViewModal.svelte';
	import ViewConfigDropdown from '$lib/components/library/ViewConfigDropdown.svelte';

	const lib = getLibrary();
	const smartListsStore = getSmartLists();
	const vp = getViewport();

	let showSaveModal = $state(false);
	let showRenameModal = $state(false);

	// Below the desktop breakpoint the detail panel can't dock beside the list, so it
	// becomes a slide-over (tablet) or full-screen view (mobile). That visibility is
	// session-only state — deliberately separate from the persisted side_panel pref,
	// which keeps governing the docked desktop panel.
	let compactDetailOpen = $state(false);

	const detailOpen = $derived(vp.isCompact ? compactDetailOpen : lib.sidePanelOpen);

	function toggleDetail() {
		if (vp.isCompact) {
			compactDetailOpen = !compactDetailOpen;
		} else {
			lib.toggleSidePanel();
		}
	}

	function openItemDetail(id: string) {
		lib.setSelectedId(id);
		compactDetailOpen = true;
	}

	$effect(() => {
		const urlType = page.params.type as string | undefined;
		untrack(() => lib.setActiveType(urlType));
	});

	$effect(() => {
		const slId = page.url.searchParams.get('smart_list');
		untrack(() => lib.setSmartList(slId));
	});

	$effect(() => {
		untrack(() => smartListsStore.loadAllSmartLists());
	});

	$effect(() => {
		const mode = lib.triageMode;
		untrack(() => lib.ensureTriageTabForMode(mode));
	});

	const triageOptions = $derived(triageOptionsForMode(lib.triageMode));

	const hasActiveConditions = $derived(lib.draftConditions.length > 0);

	const displayItems = $derived(
		lib.groupBy === 'read_status'
			? lib.readStatusTab === 'unseen'
				? lib.items.filter((i) => !i.last_read_at)
				: lib.items.filter((i) => !!i.last_read_at)
			: lib.items
	);

	function currentLibraryHref(): string {
		const base = resolve('/(app)/library');
		const type = page.params.type as string | undefined;
		return type ? `${base}/${type}` : base;
	}

	// A smart list defines its own scope; entering one always lands on the untyped
	// library route so a lingering type page can't linger in the URL.
	function smartListHref(id: string): string {
		return `${resolve('/(app)/library')}?smart_list=${id}`;
	}

	$effect(() => {
		if (!lib.filterBarOpen || lib.draftConditions.length === 0) return;
		const conds = lib.draftConditions;
		const allFilled = conds.every((c) => {
			if (Array.isArray(c.value)) return c.value.length > 0;
			if (typeof c.value === 'boolean') return true;
			if (typeof c.value === 'number') return c.value !== 0;
			return c.value !== '';
		});
		if (!allFilled) return;
		const t = setTimeout(() => lib.resetAndFetch(), 600);
		return () => clearTimeout(t);
	});

	async function handleSaveView(name: string) {
		const expr = lib.getDraftFilterExpression();
		const result = await smartListsStore.createSmartList({ name, filter_expression: expr });
		if (result) {
			showSaveModal = false;
			lib.toggleFilterBar();
			// eslint-disable-next-line svelte/no-navigation-without-resolve -- URL is built from resolve()
			goto(smartListHref(result.id));
		}
	}

	async function handleRenameView(name: string) {
		if (!lib.activeSmartList) return;
		await smartListsStore.updateSmartList(lib.activeSmartList.id, { name });
		showRenameModal = false;
	}

	async function handleDeleteView() {
		if (!lib.activeSmartList) return;
		const success = await smartListsStore.deleteSmartList(lib.activeSmartList.id);
		if (success) {
			lib.toggleViewPanel();
			// eslint-disable-next-line svelte/no-navigation-without-resolve -- URL is built from resolve()
			goto(currentLibraryHref());
		}
	}
</script>

<div class="list-panel" class:compact={lib.listDensity === 'compact'}>
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
		<button type="button" class="view-dropdown-btn" onclick={() => lib.toggleViewPanel()}>
			{lib.activeSmartList?.name ??
				(lib.activeType
					? lib.activeType.charAt(0).toUpperCase() + lib.activeType.slice(1)
					: 'Library')}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2.5"
				aria-hidden="true"
			>
				<polyline points="6 9 12 15 18 9" />
			</svg>
		</button>

		<div class="header-right">
			{#if lib.groupBy === 'triage'}
				<MorphSwitcher
					options={triageOptions}
					value={lib.triageTab}
					onchange={(v) => lib.setTriageTab(v as TriageTab)}
				/>
				<div class="tool-divider"></div>
			{:else if lib.groupBy === 'read_status'}
				<MorphSwitcher
					options={[
						{ value: 'unseen', label: 'Unseen' },
						{ value: 'seen', label: 'Seen' }
					]}
					value={lib.readStatusTab}
					onchange={(v) => lib.setReadStatusTab(v as ReadStatusTab)}
				/>
				<div class="tool-divider"></div>
			{/if}
			<button
				type="button"
				class="filter-btn"
				class:active={lib.filterBarOpen || hasActiveConditions}
				onclick={() => lib.toggleFilterBar()}
			>
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" />
				</svg>
				{#if hasActiveConditions}
					<span class="filter-count">{lib.draftConditions.length}</span>
				{/if}
				<span class="filter-label">Filter</span>
			</button>
			<SortDropdown />
			<button
				type="button"
				class="panel-toggle"
				class:active={detailOpen}
				onclick={toggleDetail}
				aria-label={detailOpen ? 'Hide detail panel' : 'Show detail panel'}
				title={detailOpen ? 'Hide detail panel' : 'Show detail panel'}
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
		</div>
	</div>

	{#if lib.filterBarOpen}
		<FilterBar
			conditions={lib.draftConditions}
			conjunction={lib.draftConjunction}
			activeType={lib.activeType}
			onConditionsChange={(c) => lib.setDraftConditions(c)}
			onConjunctionChange={(c) => lib.setDraftConjunction(c)}
			onSaveClick={() => (showSaveModal = true)}
		/>
	{/if}

	<div class="list-body">
		{#if lib.fetchError}
			<div class="fetch-error" role="alert">
				{lib.fetchError}
			</div>
		{/if}
		<ItemList
			items={displayItems}
			loading={lib.loading}
			loadingMore={lib.loadingMore}
			hasMore={lib.groupBy === 'triage' ? lib.hasMore : false}
			isEmpty={!lib.loading && displayItems.length === 0}
			selectedId={lib.selectedId}
			triageTab={lib.triageTab}
			triageMode={lib.triageMode}
			onLoadMore={() => lib.loadMore()}
			onSelect={(id) => lib.setSelectedId(id)}
			onOpen={(id) => goto(resolve('/(app)/reader/[documentId]', { documentId: id }))}
			onTriage={(id, state) => lib.triageAction(id, state)}
			onDetail={vp.isMobile ? openItemDetail : undefined}
		/>
	</div>

	{#if lib.viewPanelOpen}
		<ViewConfigDropdown
			smartList={lib.activeSmartList ?? undefined}
			allSmartLists={smartListsStore.allSmartLists}
			groupBy={lib.groupBy}
			triageMode={lib.triageMode}
			onGroupByChange={(gb) => lib.setGroupBy(gb)}
			onSwitchView={(id) => {
				// eslint-disable-next-line svelte/no-navigation-without-resolve -- URL is built from resolve()
				goto(smartListHref(id));
			}}
			onClearView={() => {
				// eslint-disable-next-line svelte/no-navigation-without-resolve -- URL is built from resolve()
				goto(currentLibraryHref());
			}}
			onNewView={() => {
				lib.setSmartList(null);
				// eslint-disable-next-line svelte/no-navigation-without-resolve -- URL is built from resolve()
				goto(currentLibraryHref());
				lib.toggleFilterBar();
			}}
			onRenameView={() => (showRenameModal = true)}
			onDeleteView={handleDeleteView}
			onEditFilter={() => {
				lib.toggleViewPanel();
				if (!lib.filterBarOpen) lib.toggleFilterBar();
			}}
			showCountBadge={lib.showCountBadge}
			onToggleCountBadge={() => lib.toggleCountBadge()}
			onMarkAllSeen={() => lib.markAllSeen()}
			onArchiveAll={() => lib.archiveAll()}
			onClose={() => lib.toggleViewPanel()}
		/>
	{/if}

	{#if showSaveModal}
		<SaveViewModal onClose={() => (showSaveModal = false)} onSaved={handleSaveView} />
	{/if}

	{#if showRenameModal && lib.activeSmartList}
		<SaveViewModal
			initialName={lib.activeSmartList.name}
			onClose={() => (showRenameModal = false)}
			onSaved={handleRenameView}
		/>
	{/if}
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
					<span class="m-dtitle">{lib.selectedItem?.title ?? 'Details'}</span>
				</div>
				<DetailPanel item={lib.selectedItem} />
			</div>
		{:else}
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="detail-scrim" onclick={() => (compactDetailOpen = false)}></div>
			<div class="detail-overlay">
				<DetailPanel item={lib.selectedItem} />
			</div>
		{/if}
	{/if}
{:else if lib.sidePanelOpen}
	<DetailPanel item={lib.selectedItem} />
{/if}

<style>
	.fetch-error {
		margin: 12px 16px 0;
		padding: 10px 14px;
		border: 1px solid var(--destructive);
		border-radius: 8px;
		background: var(--fill-danger);
		color: var(--text-primary);
		font-size: 13px;
	}

	.list-panel {
		flex: 1;
		display: flex;
		flex-direction: column;
		background: var(--bg-content);
		min-width: 0;
		overflow: hidden;
		position: relative;
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

	.header-right {
		margin-left: auto;
		display: flex;
		align-items: center;
		gap: 6px;
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

	.list-body {
		flex: 1;
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}

	.tool-divider {
		width: 0.5px;
		height: 20px;
		background: var(--border-secondary);
		margin: 0 6px;
		flex-shrink: 0;
	}

	/* Compact density overrides */
	.list-panel.compact :global(.item-row) {
		padding: 8px 20px;
		gap: 12px;
	}

	.list-panel.compact :global(.item-thumb) {
		width: 40px;
		height: 40px;
		border-radius: var(--radius-md);
	}

	.list-panel.compact :global(.item-excerpt) {
		display: none;
	}

	.list-panel.compact :global(.progress-bar) {
		display: none;
	}

	.filter-btn {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 5px 10px;
		border-radius: var(--radius-sm);
		font-size: 13px;
		font-weight: 500;
		letter-spacing: -0.01em;
		color: var(--text-secondary);
		cursor: pointer;
		background: none;
		border: none;
		font-family: var(--font-sans);
	}

	.filter-btn:hover {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.filter-btn.active {
		color: var(--accent);
		background: var(--fill-selected);
	}

	.filter-btn svg {
		width: 14px;
		height: 14px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.filter-count {
		width: 16px;
		height: 16px;
		border-radius: var(--radius-circle);
		background: var(--accent);
		color: var(--text-on-color);
		font-size: 10px;
		font-weight: 600;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.view-dropdown-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 5px 10px;
		border-radius: var(--radius-sm);
		font-size: 14px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--accent);
		cursor: pointer;
		background: none;
		border: none;
		font-family: var(--font-sans);
	}

	.view-dropdown-btn:hover {
		background: var(--fill-hover);
	}

	.view-dropdown-btn svg {
		width: 12px;
		height: 12px;
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

		/* Push the action cluster to the right edge; the triage morph stays left,
		   mirroring the prototype's controls row. */
		.header-right > .filter-btn {
			margin-left: auto;
		}

		.filter-label {
			display: none;
		}
	}
</style>
