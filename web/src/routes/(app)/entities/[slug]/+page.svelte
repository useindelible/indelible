<script lang="ts">
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import { browser } from '$app/environment';
	import * as apiSdk from '$lib/api';
	import type { EntityDetailResponse, EntityDocumentResponse } from '$lib/api';
	import LibraryShell from '$lib/components/library/LibraryShell.svelte';
	import LibrarySidebar from '$lib/components/library/LibrarySidebar.svelte';
	import { getViewport } from '$lib/stores/viewport.svelte';
	import { date, t, type MessageKey } from '$lib/i18n';
	import { setDocumentTitle } from '$lib/stores/page-title.svelte';

	const THUMB_COLORS = ['blue', 'green', 'purple', 'orange', 'red', 'teal'] as const;
	type ThumbColor = (typeof THUMB_COLORS)[number];

	const TYPE_LABEL_KEYS: Record<string, MessageKey> = {
		person: 'search_entity_type_person',
		organization: 'search_entity_type_organization',
		location: 'search_entity_type_location',
		event: 'search_entity_type_event',
		topic: 'entity_type_topic'
	};

	function entityTypeLabel(type: string): string {
		const key = TYPE_LABEL_KEYS[type];
		return key ? $t(key) : type;
	}

	const entityId = $derived(page.params.slug);

	const vp = getViewport();

	// Below the desktop breakpoint the docked info panel becomes a slide-over
	// (tablet) or full-screen view (mobile); session-only, matching the library list.
	let compactDetailOpen = $state(false);

	let entity = $state<EntityDetailResponse | null>(null);
	let items = $state<EntityDocumentResponse[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	setDocumentTitle(() => (entity && entity.id === entityId ? entity.name : null));

	let loadEpoch = 0;

	$effect(() => {
		if (!browser || !entityId) return;
		const requestedEntityId = entityId;
		const requestedEpoch = ++loadEpoch;
		// A superseded request must not overwrite the entity the URL now points at.
		const isCurrentLoad = () => requestedEntityId === entityId && requestedEpoch === loadEpoch;
		loading = true;
		error = null;
		Promise.all([
			apiSdk.getEntity({ path: { id: requestedEntityId } }),
			apiSdk.listEntityDocuments({ path: { id: requestedEntityId } })
		])
			.then(([entityRes, docsRes]) => {
				if (!isCurrentLoad()) return;
				if (entityRes.data) entity = entityRes.data;
				items = docsRes.data?.data ?? [];
			})
			.catch(() => {
				if (!isCurrentLoad()) return;
				error = $t('entity_error_load');
			})
			.finally(() => {
				if (!isCurrentLoad()) return;
				loading = false;
			});
	});

	function formatDate(iso: string): string {
		return $date(new Date(iso), {
			month: 'short',
			day: 'numeric',
			year: 'numeric'
		});
	}

	function thumbColor(index: number): ThumbColor {
		return THUMB_COLORS[index % THUMB_COLORS.length]!;
	}

	function thumbLabel(item: EntityDocumentResponse): string {
		return (item.domain ?? item.title ?? '?').charAt(0).toUpperCase();
	}
</script>

{#snippet sidebar()}
	<LibrarySidebar />
{/snippet}

{#snippet entityPanel(e: EntityDetailResponse)}
	<aside class="detail-panel">
		<div class="detail-scroll">
			<div class="detail-entity-header">
				<div class="detail-entity-name">{e.name}</div>
				<div class="detail-type-row">
					<span class="detail-type-pill">{entityTypeLabel(e.entity_type)}</span>
				</div>
			</div>

			{#if e.description}
				<div class="detail-about-section">
					<div class="section-title">{$t('entity_about')}</div>
					<div class="detail-about-text">{e.description}</div>
					<div class="detail-about-attribution">{$t('entity_generated_by_mila')}</div>
				</div>
			{/if}

			{#if items.length > 0}
				<div class="detail-timeline-section">
					<div class="section-title">{$t('entity_timeline')}</div>
					<div class="timeline">
						{#each items as item (item.id)}
							<a
								class="timeline-item"
								href={resolve('/(app)/reader/[documentId]', { documentId: item.id })}
							>
								<div class="timeline-dot"></div>
								<div class="timeline-content">
									<div class="timeline-date">
										{formatDate(item.saved_at)}
									</div>
									<div class="timeline-title">{item.title}</div>
								</div>
							</a>
						{/each}
					</div>
				</div>
			{/if}
		</div>
	</aside>
{/snippet}

{#snippet content()}
	{#if loading}
		<div class="entity-page-state">{$t('common_loading')}</div>
	{:else if error || !entity}
		<div class="entity-page-state">{error ?? $t('entity_not_found')}</div>
	{:else}
		<div class="entity-page">
			<!-- Main scroll area -->
			<div class="main-scroll">
				<!-- Entity header -->
				<div class="entity-header">
					<div class="header-nav-row">
						<button
							type="button"
							class="menu-btn"
							onclick={() => vp.openMobileNav()}
							aria-label={$t('common_open_navigation')}
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
						<a class="back-link" href={resolve('/library')}>
							<svg viewBox="0 0 24 24" aria-hidden="true">
								<polyline points="15 18 9 12 15 6" />
							</svg>
							{$t('reader_return_to_library')}
						</a>
					</div>
					<div class="entity-name-row">
						<h1 class="entity-title">{entity.name}</h1>
						<span class="entity-type-pill">{entityTypeLabel(entity.entity_type)}</span>
						{#if vp.isCompact}
							<button
								type="button"
								class="panel-toggle"
								class:active={compactDetailOpen}
								onclick={() => (compactDetailOpen = !compactDetailOpen)}
								aria-label={$t(
									compactDetailOpen ? 'reader_hide_info_panel' : 'reader_show_info_panel'
								)}
								title={$t(compactDetailOpen ? 'reader_hide_info_panel' : 'reader_show_info_panel')}
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
					<div class="entity-stats">
						<span>{$t('entity_reference_count', { values: { count: entity.total_mentions } })}</span
						>
						<span class="stat-dot"></span>
						<span>{$t('entity_document_count', { values: { count: entity.item_count } })}</span>
						<span class="stat-dot"></span>
						<span
							>{$t('entity_first_seen', {
								values: { date: formatDate(entity.first_seen_at) }
							})}</span
						>
						<span class="stat-dot"></span>
						<span
							>{$t('entity_last_seen', { values: { date: formatDate(entity.last_seen_at) } })}</span
						>
					</div>
				</div>

				<!-- Co-occurring entities -->
				{#if entity.co_occurring.length > 0}
					<div class="co-entities-section">
						<div class="section-title">{$t('entity_frequently_mentioned_with')}</div>
						<div class="co-entity-chips">
							{#each entity.co_occurring as co (co.id)}
								<a class="co-entity-chip" href={resolve('/(app)/entities/[slug]', { slug: co.id })}>
									{co.name}
									<span class="chip-count">({co.shared_item_count})</span>
								</a>
							{/each}
						</div>
					</div>
				{/if}

				<!-- Documents -->
				{#if items.length > 0}
					<div class="documents-section">
						<div class="section-header-row">
							<div class="section-title-with-count">
								<span class="section-title">{$t('entity_documents')}</span>
								<span class="section-count"
									>{$t('entity_document_count', { values: { count: entity.item_count } })}</span
								>
							</div>
						</div>
						<div class="doc-list">
							{#each items as item, i (item.id)}
								<a
									class="doc-row"
									href={resolve('/(app)/reader/[documentId]', { documentId: item.id })}
								>
									<div class="doc-thumb {thumbColor(i)}-gradient">{thumbLabel(item)}</div>
									<div class="doc-info">
										<div class="doc-title">{item.title}</div>
										<div class="doc-source-row">
											{#if item.domain}
												<span class="doc-source">{item.domain}</span>
											{/if}
											<span class="doc-date">{formatDate(item.saved_at)}</span>
										</div>
										{#if item.excerpt}
											<div class="doc-excerpt">{item.excerpt}</div>
										{/if}
									</div>
								</a>
							{/each}
						</div>
					</div>
				{/if}
			</div>

			<!-- Right detail panel -->
			{#if vp.isCompact}
				{#if compactDetailOpen}
					{#if vp.isMobile}
						<div class="m-detail">
							<div class="m-detailbar">
								<button
									type="button"
									class="m-back"
									onclick={() => (compactDetailOpen = false)}
									aria-label={$t('entity_back_to_entity')}
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
								<span class="m-dtitle">{entity.name}</span>
							</div>
							{@render entityPanel(entity)}
						</div>
					{:else}
						<!-- svelte-ignore a11y_click_events_have_key_events -->
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div class="detail-scrim" onclick={() => (compactDetailOpen = false)}></div>
						<div class="detail-overlay">
							{@render entityPanel(entity)}
						</div>
					{/if}
				{/if}
			{:else}
				{@render entityPanel(entity)}
			{/if}
		</div>
	{/if}
{/snippet}

<LibraryShell {sidebar} {content} />

<style>
	.entity-page-state {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 14px;
		color: var(--text-secondary);
		font-family: var(--font-sans);
	}

	.entity-page {
		display: flex;
		flex: 1;
		overflow: hidden;
		position: relative;
	}

	/* Main scroll */
	.main-scroll {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
		padding: 0 0 64px;
		display: flex;
		flex-direction: column;
		background: var(--bg-content);
	}

	/* Entity header */
	.entity-header {
		padding: 24px 32px 0;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.back-link {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--accent);
		cursor: pointer;
		transition: color 120ms ease;
		align-self: flex-start;
		text-decoration: none;
		font-family: var(--font-sans);
	}

	.back-link:hover {
		color: var(--accent-hover);
	}

	.back-link svg {
		width: 14px;
		height: 14px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.entity-name-row {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.entity-title {
		font-size: 28px;
		font-weight: 700;
		letter-spacing: -0.03em;
		line-height: 1.18;
		color: var(--text-primary);
		font-family: var(--font-sans);
		margin: 0;
	}

	.entity-type-pill {
		display: inline-flex;
		align-items: center;
		padding: 4px 12px;
		border-radius: 980px;
		font-size: 12px;
		font-weight: 500;
		letter-spacing: 0.02em;
		background: var(--fill-selected);
		color: var(--accent);
		flex-shrink: 0;
		font-family: var(--font-sans);
	}

	.entity-stats {
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.45;
		color: var(--text-secondary);
		display: flex;
		align-items: center;
		gap: 6px;
		flex-wrap: wrap;
		font-family: var(--font-sans);
	}

	.stat-dot {
		width: 3px;
		height: 3px;
		border-radius: 50%;
		background: var(--text-tertiary);
		flex-shrink: 0;
	}

	/* Co-occurring entities */
	.co-entities-section {
		padding: 24px 32px 0;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.section-title {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		line-height: 1.2;
		font-family: var(--font-sans);
	}

	.co-entity-chips {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}

	.co-entity-chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 5px 12px;
		border-radius: 980px;
		border: 1px solid var(--accent);
		font-size: 12px;
		font-weight: 500;
		letter-spacing: -0.005em;
		color: var(--accent);
		cursor: pointer;
		transition: background 150ms ease;
		background: transparent;
		text-decoration: none;
		font-family: var(--font-sans);
	}

	.co-entity-chip:hover {
		background: var(--fill-selected);
	}

	.chip-count {
		font-weight: 400;
		opacity: 0.7;
		font-size: 11px;
	}

	/* Documents */
	.documents-section {
		padding: 24px 32px 0;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.section-header-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.section-title-with-count {
		display: flex;
		align-items: baseline;
		gap: 8px;
	}

	.section-count {
		font-size: 12px;
		font-weight: 400;
		color: var(--text-tertiary);
		letter-spacing: -0.005em;
		font-family: var(--font-sans);
	}

	.doc-list {
		display: flex;
		flex-direction: column;
		gap: 0;
	}

	.doc-row {
		display: flex;
		align-items: flex-start;
		gap: 14px;
		padding: 14px 0;
		border-bottom: 0.5px solid var(--border-primary);
		cursor: pointer;
		transition: background 120ms ease;
		border-radius: 4px;
		text-decoration: none;
		color: var(--text-primary);
	}

	.doc-row:hover {
		background: var(--fill-hover);
		margin: 0 -8px;
		padding-left: 8px;
		padding-right: 8px;
	}

	.doc-row:last-child {
		border-bottom: none;
	}

	.doc-thumb {
		width: 48px;
		height: 48px;
		border-radius: 10px;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 22px;
		flex-shrink: 0;
	}

	.blue-gradient {
		background: linear-gradient(135deg, rgba(0, 113, 227, 0.12), rgba(0, 113, 227, 0.26));
		border: 0.5px solid rgba(0, 113, 227, 0.18);
	}

	.green-gradient {
		background: linear-gradient(135deg, rgba(52, 199, 89, 0.12), rgba(52, 199, 89, 0.26));
		border: 0.5px solid rgba(52, 199, 89, 0.18);
	}

	.purple-gradient {
		background: linear-gradient(135deg, rgba(175, 82, 222, 0.12), rgba(175, 82, 222, 0.26));
		border: 0.5px solid rgba(175, 82, 222, 0.18);
	}

	.orange-gradient {
		background: linear-gradient(135deg, rgba(255, 149, 0, 0.12), rgba(255, 149, 0, 0.26));
		border: 0.5px solid rgba(255, 149, 0, 0.18);
	}

	.red-gradient {
		background: linear-gradient(135deg, rgba(255, 59, 48, 0.12), rgba(255, 59, 48, 0.26));
		border: 0.5px solid rgba(255, 59, 48, 0.18);
	}

	.teal-gradient {
		background: linear-gradient(135deg, rgba(90, 200, 250, 0.12), rgba(90, 200, 250, 0.26));
		border: 0.5px solid rgba(90, 200, 250, 0.18);
	}

	.doc-info {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.doc-title {
		font-size: 14px;
		font-weight: 600;
		letter-spacing: -0.01em;
		line-height: 1.4;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		font-family: var(--font-sans);
	}

	.doc-source-row {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 11.5px;
	}

	.doc-source {
		font-weight: 500;
		color: var(--text-secondary);
		font-family: var(--font-sans);
	}

	.doc-date {
		font-weight: 400;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
	}

	.doc-date::before {
		content: '\00B7';
		margin: 0 3px;
	}

	.doc-excerpt {
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.45;
		color: var(--text-secondary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		font-family: var(--font-sans);
	}

	/* Right detail panel */
	.detail-panel {
		width: 300px;
		min-width: 300px;
		background: var(--vibrancy-sidebar);
		backdrop-filter: blur(60px) saturate(220%);
		-webkit-backdrop-filter: blur(60px) saturate(220%);
		border-left: 0.5px solid var(--border-primary);
		display: flex;
		flex-direction: column;
		position: relative;
		z-index: 2;
		overflow: hidden;
	}

	.detail-scroll {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
		padding: 20px;
		display: flex;
		flex-direction: column;
		gap: 0;
	}

	.detail-entity-header {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding-bottom: 16px;
	}

	.detail-entity-name {
		font-size: 20px;
		font-weight: 600;
		letter-spacing: -0.025em;
		line-height: 1.25;
		color: var(--text-primary);
		font-family: var(--font-sans);
	}

	.detail-type-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.detail-type-pill {
		display: inline-flex;
		align-items: center;
		padding: 3px 10px;
		border-radius: 980px;
		font-size: 11px;
		font-weight: 500;
		letter-spacing: 0.02em;
		background: var(--fill-selected);
		color: var(--accent);
		font-family: var(--font-sans);
	}

	.detail-about-section {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 16px 0;
		border-top: 0.5px solid var(--border-primary);
	}

	.detail-about-text {
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.65;
		color: var(--text-primary);
		font-family: var(--font-sans);
	}

	.detail-about-attribution {
		font-size: 11px;
		font-weight: 400;
		color: var(--text-tertiary);
		font-style: italic;
		font-family: var(--font-sans);
	}

	.detail-timeline-section {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 16px 0 0;
		border-top: 0.5px solid var(--border-primary);
	}

	.timeline {
		display: flex;
		flex-direction: column;
		gap: 0;
		position: relative;
		padding-left: 16px;
	}

	.timeline::before {
		content: '';
		position: absolute;
		left: 3px;
		top: 8px;
		bottom: 8px;
		width: 2px;
		background: var(--text-quaternary, var(--border-secondary));
		border-radius: 1px;
	}

	.timeline-item {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		padding: 8px 0;
		position: relative;
		cursor: pointer;
		text-decoration: none;
	}

	.timeline-item:first-child {
		padding-top: 0;
	}

	.timeline-item:last-child {
		padding-bottom: 0;
	}

	.timeline-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--accent);
		flex-shrink: 0;
		position: absolute;
		left: -16px;
		top: 10px;
		z-index: 1;
	}

	.timeline-item:first-child .timeline-dot {
		top: 4px;
	}

	.timeline-content {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.timeline-date {
		font-size: 11px;
		font-weight: 500;
		color: var(--text-secondary);
		letter-spacing: -0.005em;
		font-family: var(--font-sans);
	}

	.timeline-title {
		font-size: 12px;
		font-weight: 400;
		color: var(--text-primary);
		letter-spacing: -0.005em;
		line-height: 1.4;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
		transition: color 120ms ease;
		font-family: var(--font-sans);
	}

	.timeline-item:hover .timeline-title {
		color: var(--accent);
	}

	/* ---- Responsive: tablet slide-over + mobile reflow ---- */

	.header-nav-row {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
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
		padding: 0;
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
		margin-left: auto;
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

	/* Opaque surface: the docked panel's vibrancy blur would let the document
	   list bleed through when it floats above it. */
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

	.detail-overlay .detail-panel {
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

	.m-detail .detail-panel {
		width: 100%;
		min-width: 0;
		flex: 1;
		border-left: none;
		background: var(--bg-content);
		backdrop-filter: none;
		-webkit-backdrop-filter: none;
	}

	@media (max-width: 1099px) {
		.entity-header,
		.co-entities-section,
		.documents-section {
			padding-left: 24px;
			padding-right: 24px;
		}
	}

	@media (max-width: 599px) {
		.menu-btn {
			display: flex;
		}

		.entity-header {
			padding-top: 12px;
		}

		.entity-header,
		.co-entities-section,
		.documents-section {
			padding-left: 16px;
			padding-right: 16px;
		}

		.entity-title {
			font-size: 22px;
		}
	}
</style>
