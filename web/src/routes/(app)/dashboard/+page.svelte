<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import * as apiSdk from '$lib/api';
	import type { HighlightWithNoteResponse, HomeItemResponse } from '$lib/api';
	import { getAuth } from '$lib/stores/auth.svelte';
	import { getViewport } from '$lib/stores/viewport.svelte';
	import DashboardConfigurePopover from './components/DashboardConfigurePopover.svelte';
	import DashboardEmptyState from './components/DashboardEmptyState.svelte';
	import HighlightsSection from './components/HighlightsSection.svelte';
	import HomeSection from './components/HomeSection.svelte';
	import {
		cloneConfig,
		DEFAULT_CONFIG_SECTIONS,
		DEFAULT_CONFIG_TYPES,
		greetingLine as buildGreetingLine,
		longReadItems,
		reorder,
		type DashboardConfigItem
	} from './dashboard-model';

	const auth = getAuth();
	const vp = getViewport();
	const user = $derived(auth.user);
	const libraryHref = resolve('/library');

	let continueReadingItems = $state<HomeItemResponse[]>([]);
	let quickReadItems = $state<HomeItemResponse[]>([]);
	let recentItems = $state<HomeItemResponse[]>([]);
	let recentHighlights = $state<HighlightWithNoteResponse[]>([]);
	let loading = $state(true);
	let configureOpen = $state(false);

	// Default config — swap these out when the API endpoint is ready:
	// $effect(() => { apiSdk.getHomeConfig().then(r => { if (r.data) { configSections = r.data.sections; configTypes = r.data.types; } }); });
	let configSections = $state<DashboardConfigItem[]>(cloneConfig(DEFAULT_CONFIG_SECTIONS));
	let configTypes = $state<DashboardConfigItem[]>(cloneConfig(DEFAULT_CONFIG_TYPES));

	let secOver = $state<number | null>(null);
	let typeOver = $state<number | null>(null);
	let draggedSectionId: string | null = null;
	let draggedTypeId: string | null = null;

	const greeting = $derived(buildGreetingLine(user?.display_name));
	const longReads = $derived(longReadItems(recentItems));
	const showEmptyState = $derived(
		!loading && continueReadingItems.length === 0 && recentItems.length === 0
	);

	$effect(() => {
		loading = true;
		Promise.all([apiSdk.getHome({}), apiSdk.listRecentHighlights({ query: { limit: 5 } })])
			.then(([home, highlights]) => {
				continueReadingItems = home.data?.continue_reading?.items ?? [];
				quickReadItems = home.data?.quick_reads?.items ?? [];
				recentItems = home.data?.recently_added?.items ?? [];
				recentHighlights = highlights.data?.highlights ?? [];
			})
			.catch(() => {})
			.finally(() => {
				loading = false;
			});
	});

	function toggleConfig(items: DashboardConfigItem[], id: string): DashboardConfigItem[] {
		return items.map((item) => (item.id === id ? { ...item, on: !item.on } : item));
	}

	function onSecDragStart(id: string) {
		draggedSectionId = id;
	}

	function onSecDragOver(event: DragEvent, index: number) {
		event.preventDefault();
		secOver = index;
	}

	function onSecDrop(event: DragEvent, index: number) {
		event.preventDefault();
		if (!draggedSectionId) return;
		const from = configSections.findIndex((section) => section.id === draggedSectionId);
		if (from !== -1 && from !== index) {
			configSections = reorder(configSections, from, index);
		}
		draggedSectionId = null;
		secOver = null;
	}

	function onSecDragEnd() {
		draggedSectionId = null;
		secOver = null;
	}

	function onTypeDragStart(id: string) {
		draggedTypeId = id;
	}

	function onTypeDragOver(event: DragEvent, index: number) {
		event.preventDefault();
		typeOver = index;
	}

	function onTypeDrop(event: DragEvent, index: number) {
		event.preventDefault();
		if (!draggedTypeId) return;
		const from = configTypes.findIndex((type) => type.id === draggedTypeId);
		if (from !== -1 && from !== index) {
			configTypes = reorder(configTypes, from, index);
		}
		draggedTypeId = null;
		typeOver = null;
	}

	function onTypeDragEnd() {
		draggedTypeId = null;
		typeOver = null;
	}

	function goToItem(documentId: string) {
		goto(resolve('/(app)/reader/[documentId]', { documentId }));
	}
</script>

{#if configureOpen}
	<DashboardConfigurePopover
		sections={configSections}
		types={configTypes}
		sectionOver={secOver}
		{typeOver}
		onClose={() => (configureOpen = false)}
		onToggleSection={(id) => (configSections = toggleConfig(configSections, id))}
		onToggleType={(id) => (configTypes = toggleConfig(configTypes, id))}
		onSectionDragStart={onSecDragStart}
		onSectionDragOver={onSecDragOver}
		onSectionDrop={onSecDrop}
		onSectionDragEnd={onSecDragEnd}
		{onTypeDragStart}
		{onTypeDragOver}
		{onTypeDrop}
		{onTypeDragEnd}
	/>
{/if}

<div class="home">
	<div class="home-header">
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
		<p class="home-greeting">{greeting}</p>
		<button type="button" class="configure-btn" onclick={() => (configureOpen = true)}>
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.5"
				stroke-linecap="round"
				stroke-linejoin="round"
				aria-hidden="true"
			>
				<path
					d="M12 3h7a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-7m0-18H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h7m0-18v18"
				/>
				<path d="M7 8h2" />
				<path d="M7 12h2" />
				<path d="M15 8h2" />
				<path d="M15 16h2" />
			</svg>
			<span class="configure-label">Configure</span>
		</button>
	</div>

	<div class="home-sections">
		{#each configSections as section (section.id)}
			{#if section.on}
				{#if section.id === 'continue' && (loading || continueReadingItems.length > 0)}
					<HomeSection
						title="Continue Reading"
						items={continueReadingItems}
						{loading}
						seeAllHref={libraryHref}
						onOpen={goToItem}
					/>
				{/if}

				{#if section.id === 'quick' && (loading || quickReadItems.length > 0)}
					<HomeSection
						title="Quick Reads"
						items={quickReadItems}
						{loading}
						seeAllHref={libraryHref}
						onOpen={goToItem}
					/>
				{/if}

				{#if section.id === 'long' && (loading || longReads.length > 0)}
					<HomeSection
						title="Long Reads"
						items={longReads}
						{loading}
						seeAllHref={libraryHref}
						onOpen={goToItem}
					/>
				{/if}

				{#if section.id === 'review'}
					<HomeSection title="Daily Review">
						<div class="review-widget">
							<div class="review-icon">
								<svg
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="1.8"
									stroke-linecap="round"
									stroke-linejoin="round"
									aria-hidden="true"
								>
									<path
										d="M12 2L15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2z"
									/>
								</svg>
							</div>
							<div class="review-body">
								<div class="review-title">Review your highlights</div>
								<div class="review-subtitle">
									Resurface what you've saved and build lasting knowledge.
								</div>
							</div>
							<button type="button" class="review-cta" disabled>Coming soon</button>
						</div>
					</HomeSection>
				{/if}

				{#if section.id === 'recent' && (loading || recentItems.length > 0)}
					<HomeSection
						title="Recently Added"
						items={recentItems}
						{loading}
						seeAllHref={libraryHref}
						onOpen={goToItem}
					/>
				{/if}

				{#if section.id === 'highlights' && (loading || recentHighlights.length > 0)}
					<HighlightsSection highlights={recentHighlights} {loading} />
				{/if}
			{/if}
		{/each}

		{#if showEmptyState}
			<DashboardEmptyState />
		{/if}
	</div>
</div>

<style>
	.home {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		overflow-y: auto;
		overflow-x: hidden;
		background: var(--bg-content, var(--bg-primary));
	}

	.home::-webkit-scrollbar {
		width: 6px;
	}

	.home::-webkit-scrollbar-track {
		background: transparent;
	}

	.home::-webkit-scrollbar-thumb {
		background: var(--text-quaternary);
		border-radius: 3px;
	}

	.home::-webkit-scrollbar-thumb:hover {
		background: var(--text-tertiary);
	}

	.home-header {
		padding: 0 28px;
		height: 60px;
		display: flex;
		align-items: center;
		gap: 10px;
		flex-shrink: 0;
		border-bottom: 0.5px solid var(--border-primary);
	}

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
		margin-left: -6px;
	}

	.menu-btn:hover {
		background: var(--fill-hover);
	}

	.menu-btn svg {
		width: 20px;
		height: 20px;
	}

	.configure-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 7px 14px;
		border-radius: 980px;
		border: 1px solid var(--border-primary);
		background: transparent;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-secondary);
		cursor: pointer;
		transition:
			background 150ms ease,
			border-color 150ms ease,
			color 150ms ease;
	}

	.configure-btn svg {
		width: 14px;
		height: 14px;
	}

	.configure-btn:hover {
		background: var(--fill-hover);
		border-color: var(--border-secondary);
		color: var(--text-primary);
	}

	.home-greeting {
		padding: 0;
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 400;
		line-height: 1.5;
		color: var(--text-secondary);
		margin: 0;
		flex: 1;
		min-width: 0;
	}

	.home-sections {
		padding: 24px 28px 40px;
		display: flex;
		flex-direction: column;
		gap: 32px;
		flex: 1;
	}

	.review-widget {
		display: flex;
		align-items: center;
		gap: 20px;
		padding: 20px 24px;
		border-radius: 14px;
		background: var(--fill-warning);
		border: 0.5px solid var(--highlight-yellow-border);
	}

	.review-icon {
		width: 52px;
		height: 52px;
		border-radius: 14px;
		background: var(--warning);
		color: var(--text-on-color);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		box-shadow: var(--shadow-1);
	}

	.review-icon svg {
		width: 26px;
		height: 26px;
	}

	.review-body {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.review-title {
		font-family: var(--font-sans);
		font-size: 16px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.review-subtitle {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		color: var(--text-secondary);
	}

	.review-cta {
		padding: 9px 20px;
		border-radius: 980px;
		background: var(--accent);
		color: var(--text-on-color);
		font-family: var(--font-sans);
		font-size: 14px;
		font-weight: 600;
		border: none;
		cursor: pointer;
		white-space: nowrap;
		flex-shrink: 0;
		transition:
			filter 150ms ease,
			transform 150ms ease;
	}

	.review-cta:hover:not(:disabled) {
		filter: brightness(1.1);
		transform: scale(1.02);
	}

	.review-cta:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}

	/* ---- Responsive ---- */

	@media (max-width: 599px) {
		.menu-btn {
			display: flex;
		}

		.home-header {
			padding: 0 16px;
			height: 54px;
		}

		.home-greeting {
			white-space: nowrap;
			overflow: hidden;
			text-overflow: ellipsis;
		}

		.configure-label {
			display: none;
		}

		.configure-btn {
			padding: 7px 9px;
		}

		.home-sections {
			padding: 18px 16px 32px;
			gap: 26px;
		}

		/* The CTA drops below the text instead of squeezing it. */
		.review-widget {
			flex-wrap: wrap;
			gap: 14px;
			padding: 16px 18px;
		}

		.review-body {
			flex: 1 1 160px;
		}
	}
</style>
