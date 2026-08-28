<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { getLibrary } from '$lib/stores/library.svelte';
	import { getTags } from '$lib/stores/tags.svelte';
	import { getViewport } from '$lib/stores/viewport.svelte';
	import { sanitizeColor } from '$lib/utils/color';
	import TagColorPicker from '$lib/components/tags/TagColorPicker.svelte';
	import ItemList from '$lib/components/library/ItemList.svelte';
	import { date, t, type MessageKey } from '$lib/i18n';
	import { setDocumentTitle } from '$lib/stores/page-title.svelte';

	const store = getTags();
	const lib = getLibrary();
	const vp = getViewport();

	let showEditModal = $state(false);
	let showDeleteConfirm = $state(false);
	let activeTab = $state<'items' | 'highlights'>('items');

	let formName = $state('');
	let formColor = $state<string | null>(null);

	const tagId = $derived(page.params.id ?? '');

	setDocumentTitle(() => {
		const tag = store.currentTag;
		if (!tag || tag.id !== tagId) return null;
		return $t('tag_page_title_named', { values: { name: tag.name } });
	});

	$effect(() => {
		const id = tagId;
		if (id) {
			store.loadTag(id);
			store.loadTagItems(id, true);
			store.loadTagHighlights(id, true);
		}
	});

	function openEdit() {
		if (!store.currentTag) return;
		formName = store.currentTag.name;
		formColor = store.currentTag.color ?? null;
		showEditModal = true;
	}

	async function handleEditSubmit() {
		if (!store.currentTag || !formName.trim()) return;
		const updated = await store.updateTag(store.currentTag.id, {
			name: formName.trim(),
			color: formColor
		});
		if (updated) {
			showEditModal = false;
		}
	}

	async function confirmDelete() {
		if (!store.currentTag) return;
		const ok = await store.deleteTag(store.currentTag.id);
		if (ok) {
			goto(resolve('/(app)/tags'));
		}
		showDeleteConfirm = false;
	}

	function handleLoadMoreItems() {
		if (tagId) store.loadTagItems(tagId);
	}

	function handleLoadMoreHighlights() {
		if (tagId) store.loadTagHighlights(tagId);
	}

	function itemTypeLabel(value?: string | null): string {
		if (!value) return '';
		if (value === 'pdf' || value === 'epub') return value.toUpperCase();
		const keys: Record<string, MessageKey> = {
			article: 'library_filter_value_article',
			book: 'library_filter_value_book',
			email: 'library_filter_value_email',
			podcast: 'library_filter_value_podcast',
			tweet: 'library_filter_value_tweet',
			video: 'library_filter_value_video'
		};
		if (keys[value]) return $t(keys[value]);
		return value.replaceAll('_', ' ').replace(/^./, (letter) => letter.toUpperCase());
	}

	function highlightTitle(highlight: (typeof store.tagHighlights)[number]): string {
		return (
			highlight.item_title?.trim() ||
			highlight.item_domain?.trim() ||
			itemTypeLabel(highlight.item_type) ||
			$t('tag_saved_item')
		);
	}

	function highlightLocator(highlight: (typeof store.tagHighlights)[number]): string {
		const locator = highlight.locator;
		if (!locator) return $t('tag_highlight_text');
		if (locator.type === 'pdf' && locator.page) {
			return $t('tag_highlight_page', { values: { page: locator.page } });
		}
		if (locator.type === 'epub' && locator.chapter) {
			return $t('tag_highlight_chapter', { values: { chapter: locator.chapter } });
		}
		return $t('tag_highlight_text');
	}

	function highlightDate(value: string): string {
		return $date(new Date(value), {
			day: 'numeric',
			month: 'short',
			year: 'numeric'
		});
	}

	function highlightHref(documentId: string, highlightId: string): string {
		const readerHref = resolve('/(app)/reader/[documentId]', { documentId });
		return `${readerHref}?highlight=${encodeURIComponent(highlightId)}`;
	}
</script>

<div class="tag-detail">
	{#if store.loading && !store.currentTag}
		<div class="loading-state">
			<span class="loading-text">{$t('tag_loading')}</span>
		</div>
	{:else if store.currentTag}
		{@const tag = store.currentTag}

		<!-- Header -->
		<div class="detail-header">
			<div class="header-top">
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
				<div class="header-left">
					<span
						class="header-dot"
						style="background: {sanitizeColor(tag.color) ?? 'var(--text-tertiary)'}"
						aria-hidden="true"
					></span>
					<h1 class="header-title">{tag.name}</h1>
					{#if tag.aliases.length > 0}
						<span class="header-aka" title={tag.aliases.join(', ')}>{$t('tag_aka')}</span>
					{/if}
				</div>
				<div class="header-actions">
					<button type="button" class="action-btn" aria-label={$t('tag_edit')} onclick={openEdit}>
						<svg
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.6"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z" />
						</svg>
					</button>
					<button
						type="button"
						class="action-btn action-btn-danger"
						aria-label={$t('tag_delete')}
						onclick={() => {
							showDeleteConfirm = true;
						}}
					>
						<svg
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.6"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<polyline points="3 6 5 6 21 6" />
							<path
								d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
							/>
						</svg>
					</button>
				</div>
			</div>
			<div class="header-meta">
				<span class="meta-count"
					>{$t('tag_document_count', { values: { count: tag.item_count } })}</span
				>
				<span class="meta-sep">·</span>
				<span class="meta-count"
					>{$t('tag_highlight_count', { values: { count: tag.highlight_count } })}</span
				>
			</div>
		</div>

		<!-- Tab bar -->
		<div class="tab-bar">
			<button
				type="button"
				class="tab-btn"
				class:active={activeTab === 'items'}
				onclick={() => {
					activeTab = 'items';
				}}
			>
				{$t('tag_documents', { values: { count: tag.item_count } })}
			</button>
			<button
				type="button"
				class="tab-btn"
				class:active={activeTab === 'highlights'}
				onclick={() => {
					activeTab = 'highlights';
				}}
			>
				{$t('tag_highlights', { values: { count: tag.highlight_count } })}
			</button>
		</div>

		<!-- Content -->
		{#if activeTab === 'items'}
			<ItemList
				items={store.tagItems}
				loading={store.itemsLoading}
				loadingMore={store.itemsLoadingMore}
				hasMore={store.itemsHasMore}
				isEmpty={!store.itemsLoading && store.tagItems.length === 0}
				selectedId={null}
				triageTab="inbox"
				triageMode={lib.triageMode}
				onLoadMore={handleLoadMoreItems}
				onSelect={() => {}}
				onOpen={(id) => {
					goto(resolve('/(app)/reader/[documentId]', { documentId: id }));
				}}
				onTriage={() => {}}
				onDelete={(id) => store.deleteTagItem(id)}
			/>
		{:else}
			<div class="highlights-list">
				{#if store.highlightsLoading && store.tagHighlights.length === 0}
					<div class="loading-state">
						<span class="loading-text">{$t('tag_loading_highlights')}</span>
					</div>
				{:else if store.tagHighlights.length === 0}
					<div class="empty-state">
						<p class="empty-heading">{$t('tag_no_highlights')}</p>
					</div>
				{:else}
					{#each store.tagHighlights as hl (hl.id)}
						{#if hl.document_id}
							<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- highlightHref resolves the route before appending the query. -->
							<a class="highlight-card" href={highlightHref(hl.document_id, hl.id)}>
								<div
									class="hl-color-bar"
									style="background: {sanitizeColor(hl.color) ?? '#FFD600'}"
								></div>
								<div class="hl-body">
									<div class="hl-heading">
										<strong>{highlightTitle(hl)}</strong>
										<span>{highlightDate(hl.created_at)}</span>
									</div>
									<div class="hl-context">
										{#if hl.item_domain}
											<span>{hl.item_domain} · {itemTypeLabel(hl.item_type)}</span>
										{:else if hl.item_type}
											<span>{itemTypeLabel(hl.item_type)}</span>
										{/if}
										<span>{highlightLocator(hl)}</span>
									</div>
									{#if hl.text_content}
										<p class="hl-text">{hl.text_content}</p>
									{/if}
									{#if hl.note}
										<p class="hl-note">{hl.note}</p>
									{/if}
								</div>
							</a>
						{/if}
					{/each}

					{#if store.highlightsLoadingMore}
						<div class="loading-state">
							<span class="loading-text">{$t('tag_loading_more')}</span>
						</div>
					{:else if store.highlightsHasMore}
						<button
							type="button"
							class="btn btn-secondary load-more-btn"
							onclick={handleLoadMoreHighlights}
						>
							{$t('tag_load_more')}
						</button>
					{/if}
				{/if}
			</div>
		{/if}
	{:else if store.fetchError}
		<div class="error-state">
			<p class="error-text">{store.fetchError}</p>
			<a href={resolve('/(app)/tags')} class="btn btn-secondary">{$t('tag_back_to_all')}</a>
		</div>
	{/if}
</div>

<!-- Edit modal -->
{#if showEditModal}
	<div class="modal-overlay" role="dialog" aria-modal="true" aria-label={$t('tag_edit')}>
		<div
			class="modal-backdrop"
			onclick={() => {
				showEditModal = false;
			}}
			role="presentation"
		></div>
		<div class="modal-panel">
			<h2 class="modal-title">{$t('tag_edit')}</h2>
			<form
				class="modal-body"
				onsubmit={(e) => {
					e.preventDefault();
					handleEditSubmit();
				}}
			>
				<label class="field">
					<span class="field-label">{$t('tag_name')}</span>
					<input
						type="text"
						class="field-input"
						bind:value={formName}
						placeholder={$t('tag_name_placeholder')}
						required
						autofocus
					/>
				</label>
				<div class="field">
					<span class="field-label">{$t('tag_color')}</span>
					<TagColorPicker
						value={formColor}
						onChange={(c) => {
							formColor = c;
						}}
					/>
				</div>
				<div class="modal-actions">
					<button
						type="button"
						class="btn btn-secondary"
						onclick={() => {
							showEditModal = false;
						}}>{$t('common_cancel')}</button
					>
					<button type="submit" class="btn btn-primary" disabled={!formName.trim()}
						>{$t('common_save')}</button
					>
				</div>
			</form>
		</div>
	</div>
{/if}

<!-- Delete confirm -->
{#if showDeleteConfirm}
	<div class="modal-overlay" role="dialog" aria-modal="true" aria-label={$t('tag_delete')}>
		<div
			class="modal-backdrop"
			onclick={() => {
				showDeleteConfirm = false;
			}}
			role="presentation"
		></div>
		<div class="modal-panel">
			<h2 class="modal-title">
				{$t('tag_delete_question', { values: { name: store.currentTag?.name ?? '' } })}
			</h2>
			<p class="modal-desc">{$t('tag_delete_body')}</p>
			<div class="modal-actions">
				<button
					type="button"
					class="btn btn-secondary"
					onclick={() => {
						showDeleteConfirm = false;
					}}>{$t('common_cancel')}</button
				>
				<button type="button" class="btn btn-danger" onclick={confirmDelete}
					>{$t('common_delete')}</button
				>
			</div>
		</div>
	</div>
{/if}

<style>
	.tag-detail {
		flex: 1;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		background: var(--bg-content);
	}

	/* ---- Header ---- */
	.detail-header {
		padding: 16px 20px 12px;
		border-bottom: 0.5px solid var(--border-primary);
		flex-shrink: 0;
	}

	.header-top {
		display: flex;
		align-items: center;
		gap: 16px;
		margin-bottom: 6px;
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: 10px;
		min-width: 0;
		flex: 1;
	}

	.header-dot {
		width: 14px;
		height: 14px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.header-title {
		font-family: var(--font-sans);
		font-size: 22px;
		font-weight: 700;
		letter-spacing: -0.03em;
		color: var(--text-primary);
		margin: 0;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.header-aka {
		display: inline-flex;
		align-items: center;
		padding: 1px 6px;
		background: var(--fill-hover);
		border: 0.5px solid var(--border-primary);
		border-radius: 4px;
		font-family: var(--font-sans);
		font-size: 10px;
		font-weight: 500;
		letter-spacing: 0.02em;
		color: var(--text-tertiary);
		white-space: nowrap;
		flex-shrink: 0;
		cursor: default;
	}

	.header-actions {
		display: flex;
		gap: 4px;
		flex-shrink: 0;
	}

	.action-btn {
		width: 32px;
		height: 32px;
		border-radius: 8px;
		border: none;
		background: transparent;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		color: var(--text-tertiary);
		padding: 0;
		transition:
			background 0.12s ease,
			color 0.12s ease;
	}

	.action-btn:hover {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.action-btn-danger:hover {
		color: var(--destructive);
	}

	.action-btn svg {
		width: 16px;
		height: 16px;
	}

	.header-meta {
		display: flex;
		align-items: center;
		gap: 6px;
		padding-left: 24px;
	}

	.meta-count {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		color: var(--text-tertiary);
	}

	.meta-sep {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-quaternary);
	}

	/* ---- Tab bar ---- */
	.tab-bar {
		display: flex;
		padding: 0 20px;
		border-bottom: 0.5px solid var(--border-primary);
		flex-shrink: 0;
	}

	.tab-btn {
		padding: 10px 16px;
		border: none;
		background: transparent;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		letter-spacing: -0.01em;
		color: var(--text-secondary);
		cursor: pointer;
		border-bottom: 2px solid transparent;
		margin-bottom: -0.5px;
		transition:
			color 0.12s ease,
			border-color 0.12s ease;
	}

	.tab-btn:hover {
		color: var(--text-primary);
	}

	.tab-btn.active {
		color: var(--accent);
		border-bottom-color: var(--accent);
	}

	/* ---- Highlights list ---- */
	.highlights-list {
		padding: 16px 20px;
		display: flex;
		flex-direction: column;
		gap: 10px;
		flex: 1;
	}

	.highlight-card {
		display: flex;
		gap: 12px;
		padding: 14px 16px;
		border-radius: 10px;
		border: 1px solid var(--border-primary);
		background: var(--bg-secondary);
		color: inherit;
		text-decoration: none;
		transition: border-color 0.12s ease;
	}

	.highlight-card:hover,
	.highlight-card:focus-visible {
		border-color: var(--accent);
	}

	.highlight-card:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 2px;
	}

	.hl-color-bar {
		width: 3px;
		border-radius: 2px;
		flex-shrink: 0;
		align-self: stretch;
	}

	.hl-body {
		display: flex;
		flex-direction: column;
		gap: 6px;
		min-width: 0;
	}

	.hl-heading,
	.hl-context {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
	}

	.hl-heading strong {
		font-size: 13px;
		color: var(--text-primary);
	}

	.hl-heading span,
	.hl-context {
		font-size: 12px;
		color: var(--text-tertiary);
	}

	.hl-text {
		font-family: var(--font-sans);
		font-size: 14px;
		font-weight: 400;
		color: var(--text-primary);
		line-height: 1.5;
		margin: 0;
	}

	.hl-note {
		margin: 0;
		padding-left: 10px;
		border-left: 2px solid var(--border-primary);
		font-size: 13px;
		line-height: 1.45;
		color: var(--text-secondary);
	}

	/* ---- States ---- */
	.loading-state,
	.empty-state,
	.error-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
		padding: 60px 20px;
		flex: 1;
	}

	.loading-text,
	.error-text {
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--text-secondary);
		margin: 0;
	}

	.empty-heading {
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0;
	}

	/* ---- Shared buttons + modals ---- */
	.btn {
		height: 36px;
		padding: 0 16px;
		border-radius: 8px;
		font-family: var(--font-sans);
		font-size: 14px;
		font-weight: 500;
		letter-spacing: -0.01em;
		cursor: pointer;
		border: none;
		text-decoration: none;
		display: inline-flex;
		align-items: center;
		transition:
			background 0.12s ease,
			opacity 0.12s ease;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-primary {
		background: var(--accent);
		color: var(--text-on-color);
	}

	.btn-primary:hover:not(:disabled) {
		opacity: 0.9;
	}

	.btn-secondary {
		background: var(--fill-secondary);
		color: var(--text-primary);
	}

	.btn-secondary:hover {
		background: var(--fill-hover);
	}

	.btn-danger {
		background: var(--destructive);
		color: var(--text-on-color);
	}

	.btn-danger:hover:not(:disabled) {
		opacity: 0.9;
	}

	.load-more-btn {
		align-self: center;
		margin-top: 8px;
	}

	.modal-overlay {
		position: fixed;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 100;
	}

	.modal-backdrop {
		position: absolute;
		inset: 0;
		background: rgba(0, 0, 0, 0.4);
		backdrop-filter: blur(4px);
	}

	.modal-panel {
		position: relative;
		background: var(--bg-primary);
		border: 1px solid var(--border-primary);
		border-radius: 16px;
		padding: 24px;
		width: 380px;
		max-width: 90vw;
		box-shadow: 0 20px 60px rgba(0, 0, 0, 0.15);
	}

	.modal-title {
		font-family: var(--font-sans);
		font-size: 17px;
		font-weight: 700;
		letter-spacing: -0.02em;
		color: var(--text-primary);
		margin: 0 0 16px;
	}

	.modal-body {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.modal-desc {
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--text-secondary);
		margin: 0 0 20px;
		line-height: 1.5;
	}

	.modal-actions {
		display: flex;
		justify-content: flex-end;
		gap: 10px;
		padding-top: 8px;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.field-label {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-secondary);
	}

	.field-input {
		height: 40px;
		border-radius: 10px;
		border: 1px solid var(--border-primary);
		background: transparent;
		padding: 0 12px;
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--text-primary);
		outline: none;
		transition:
			border-color 0.15s ease,
			box-shadow 0.15s ease;
	}

	.field-input:focus {
		border-color: var(--accent);
		box-shadow: 0 0 0 3px var(--fill-selected);
	}

	/* ---- Responsive: mobile reflow ---- */

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

	@media (max-width: 599px) {
		.menu-btn {
			display: flex;
		}

		.detail-header {
			padding: 12px 16px 10px;
		}

		/* The dot indents 24px on desktop to align meta under the title; with the
		   hamburger present the meta hugs the left gutter instead. */
		.header-meta {
			padding-left: 0;
		}

		.tab-bar {
			padding: 0 16px;
		}

		.highlights-list {
			padding: 16px;
		}
	}
</style>
