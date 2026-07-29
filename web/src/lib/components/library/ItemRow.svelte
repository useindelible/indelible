<script lang="ts">
	import type { DocumentListEntry, TriageModeDto } from '$lib/api';
	import type { TriageTab } from '$lib/stores/library.svelte';
	import { formatReadingTime } from '$lib/utils/format';
	import * as api from '$lib/api';
	import { mount, unmount } from 'svelte';
	import HoverActions from './HoverActions.svelte';
	import ContextMenu from './ContextMenu.svelte';
	import TagInput from './TagInput.svelte';

	interface Props {
		item: DocumentListEntry;
		selected: boolean;
		onSelect: () => void;
		onOpen: () => void;
		onTriage: (state: TriageTab) => void;
		onDelete: () => void;
		onDetail?: () => void;
		animationDelay?: number;
		triageMode?: TriageModeDto;
		showFeedBadge?: boolean;
	}

	let {
		item,
		selected,
		onSelect,
		onOpen,
		onTriage,
		onDelete,
		onDetail,
		triageMode = 'focus',
		showFeedBadge = false
	}: Props = $props();

	let hovered = $state(false);
	let contextMenu = $state<{ x: number; y: number } | null>(null);
	let contextMenuInstance: ReturnType<typeof mount> | undefined;

	$effect(() => {
		if (contextMenu) {
			contextMenuInstance = mount(ContextMenu, {
				target: document.body,
				props: {
					item,
					x: contextMenu.x,
					y: contextMenu.y,
					onClose: () => (contextMenu = null),
					triageMode,
					onTriage,
					onDelete,
					onAddTags: isFeedRow ? undefined : openTagPicker
				}
			});
		} else {
			if (contextMenuInstance) {
				unmount(contextMenuInstance);
				contextMenuInstance = undefined;
			}
		}
		return () => {
			if (contextMenuInstance) {
				unmount(contextMenuInstance);
				contextMenuInstance = undefined;
			}
		};
	});

	// Feed deliveries carry a `dlv_` id, not a library-backed document; tag mutations would hit a
	// dead path, so the tag picker is only offered for saved library rows.
	const isFeedRow = $derived(item.object === 'feed_delivery');

	// Tag picker
	let tagPickerOpen = $state(false);
	let tagPickerTags = $state<string[]>([]);
	let tagPickerOriginal = $state<string[]>([]);
	let tagPickerLoading = $state(false);
	let tagPickerSaving = $state(false);
	let tagPickerError = $state<string | null>(null);

	const tagPickerChanged = $derived(
		tagPickerTags.length !== tagPickerOriginal.length ||
			tagPickerTags.some((t) => !tagPickerOriginal.includes(t))
	);

	async function openTagPicker() {
		tagPickerLoading = true;
		tagPickerError = null;
		tagPickerOpen = true;
		try {
			const resp = await api.getDocumentEntryTags({ path: { document_id: item.id } });
			const loaded = resp.data?.tags ?? [];
			tagPickerTags = [...loaded];
			tagPickerOriginal = [...loaded];
		} catch {
			tagPickerError = 'Failed to load tags';
		} finally {
			tagPickerLoading = false;
		}
	}

	async function saveTagPicker() {
		tagPickerSaving = true;
		tagPickerError = null;
		try {
			await api.replaceDocumentEntryTags({
				path: { document_id: item.id },
				body: { tags: tagPickerTags }
			});
			tagPickerOpen = false;
		} catch {
			tagPickerError = 'Failed to save tags';
		} finally {
			tagPickerSaving = false;
		}
	}

	function faviconUrl(domain: string | null | undefined): string {
		if (!domain) return '';
		return `https://www.google.com/s2/favicons?domain=${domain}&sz=32`;
	}

	function thumbGradient(item: DocumentListEntry): string {
		const type = item.item_type;
		const domain = item.domain ?? '';
		if (type === 'video') return 'red-gradient';
		if (type === 'podcast') return 'purple-gradient';
		if (type === 'email') return 'orange-gradient';
		if (type === 'pdf') return 'teal-gradient';
		if (type === 'tweet') return 'teal-gradient';
		if (type === 'book') return 'green-gradient';
		const hash = domain.split('').reduce((acc, c) => acc + c.charCodeAt(0), 0);
		const gradients = ['blue-gradient', 'green-gradient', 'purple-gradient', 'orange-gradient'];
		return gradients[hash % gradients.length] ?? 'blue-gradient';
	}

	function thumbEmoji(item: DocumentListEntry): string {
		const type = item.item_type;
		if (type === 'video') return '\u{1F3AC}';
		if (type === 'podcast') return '\u{1F3A7}';
		if (type === 'email') return '\u{2709}\u{FE0F}';
		if (type === 'pdf') return '\u{1F4C4}';
		if (type === 'tweet') return '\u{1F426}';
		if (type === 'book') return '\u{1F4D6}';
		return '\u{1F4F0}';
	}

	function formatTimestamp(saved: string): string {
		const d = new Date(saved);
		const now = new Date();
		const diffMs = now.getTime() - d.getTime();
		const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
		if (diffDays === 0) {
			const diffHrs = Math.floor(diffMs / (1000 * 60 * 60));
			if (diffHrs === 0) {
				const diffMin = Math.floor(diffMs / (1000 * 60));
				return `${diffMin}m`;
			}
			return `${diffHrs}h`;
		}
		if (diffDays < 7) return `${diffDays}d`;
		if (diffDays < 30) return `${Math.floor(diffDays / 7)}w`;
		return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
	}

	function openContextMenu(e: MouseEvent) {
		e.preventDefault();
		contextMenu = { x: e.clientX, y: e.clientY };
	}

	const isUnread = $derived(item.triage_state === 'inbox');
	const gradient = $derived(thumbGradient(item));
	const emoji = $derived(thumbEmoji(item));
	const timestamp = $derived(formatTimestamp(item.saved_at));
</script>

<div
	class="item-row"
	class:selected
	class:hovered
	role="option"
	aria-selected={selected}
	tabindex="0"
	onclick={onOpen}
	onkeydown={(e) => {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			onOpen();
		}
	}}
	onmouseenter={() => {
		hovered = true;
		onSelect();
	}}
	onmouseleave={() => (hovered = false)}
	oncontextmenu={openContextMenu}
>
	<div class="item-thumb {gradient}" aria-hidden="true">
		{#if item.thumbnail_url ?? item.lead_image_url}
			<img
				src={item.thumbnail_url ?? item.lead_image_url}
				alt=""
				class="thumb-img"
				onerror={(e) => {
					const img = e.currentTarget as HTMLImageElement;
					if (item.thumbnail_url && item.lead_image_url && img.src !== item.lead_image_url) {
						img.src = item.lead_image_url;
					} else {
						img.style.display = 'none';
					}
				}}
			/>
		{:else}
			<span class="thumb-emoji">{emoji}</span>
		{/if}
		{#if isUnread}
			<span class="unread-dot" aria-label="Unread"></span>
		{/if}
	</div>

	<div class="item-content">
		<div class="item-title-line">
			<p class="item-title">{item.title}</p>
			{#if showFeedBadge}
				<span class="feed-badge">Feed</span>
			{/if}
		</div>
		{#if item.summary ?? item.excerpt}
			<p class="item-excerpt">{item.summary ?? item.excerpt}</p>
		{/if}
		<div class="item-source-row">
			{#if item.domain}
				<img
					src={faviconUrl(item.domain)}
					alt=""
					class="favicon"
					width="14"
					height="14"
					onerror={(e) => {
						const el = e.currentTarget as HTMLImageElement;
						el.style.display = 'none';
						el.nextElementSibling?.removeAttribute('hidden');
					}}
				/>
				<span class="favicon-fallback" hidden aria-hidden="true">
					<span class="favicon-dot" style="background: var(--accent)"></span>
				</span>
				<span class="source-domain">{item.domain}</span>
			{/if}
			{#if item.author}
				<span class="source-meta">{item.author}</span>
			{/if}
			{#if item.reading_time_minutes}
				<span class="source-meta">{formatReadingTime(item.reading_time_minutes)}</span>
			{/if}
			<span class="source-meta source-timestamp">{timestamp}</span>
		</div>
		<div class="progress-bar" aria-hidden="true">
			<div class="progress-bar-fill" style="width: {item.progress_percent ?? 0}%"></div>
		</div>
	</div>

	<div class="item-meta-col" class:hidden={hovered}>
		<span class="item-timestamp">{timestamp}</span>
	</div>

	{#if onDetail}
		<button
			type="button"
			class="detail-chevron"
			onclick={(e) => {
				e.stopPropagation();
				onDetail();
			}}
			aria-label="Show details"
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
				<polyline points="9 18 15 12 9 6" />
			</svg>
		</button>
	{/if}

	{#if hovered}
		<div class="hover-actions-wrapper">
			<HoverActions
				currentTriage={item.triage_state}
				{triageMode}
				{onTriage}
				onMore={openContextMenu}
			/>
		</div>
	{/if}
</div>

{#if tagPickerOpen}
	<div
		class="cmd-backdrop"
		role="dialog"
		aria-modal="true"
		aria-label="Edit tags"
		tabindex="-1"
		onclick={() => (tagPickerOpen = false)}
		onkeydown={(e) => {
			if (e.key === 'Escape') tagPickerOpen = false;
		}}
	>
		<div class="cmd-card" role="none" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
			<div class="cmd-body">
				{#if tagPickerLoading}
					<p class="cmd-loading">Loading tags…</p>
				{:else}
					<div class="cmd-tag-field">
						<TagInput bind:tags={tagPickerTags} autofocus />
					</div>
				{/if}
				{#if tagPickerError}
					<p class="cmd-error">{tagPickerError}</p>
				{/if}
			</div>
			<div class="cmd-controls">
				<button type="button" class="cmd-secondary" onclick={() => (tagPickerOpen = false)}>
					Cancel
				</button>
				<button
					type="button"
					class="cmd-action"
					disabled={tagPickerSaving || tagPickerLoading || !tagPickerChanged}
					onclick={saveTagPicker}
				>
					{tagPickerSaving ? 'Saving…' : 'Save tags'}
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.item-row {
		display: flex;
		align-items: flex-start;
		gap: 16px;
		padding: 14px 20px;
		border-bottom: 0.5px solid var(--border-primary);
		cursor: pointer;
		position: relative;
		transition: background 0.12s ease;
		outline: none;
	}

	.item-row:hover,
	.item-row.hovered {
		background: var(--fill-hover);
	}

	.item-row.selected {
		background: var(--fill-selected);
	}

	.item-row.selected::before {
		content: '';
		position: absolute;
		left: 0;
		top: 8px;
		bottom: 8px;
		width: 3px;
		background: var(--accent);
		border-radius: 0 2px 2px 0;
	}

	.item-row:focus-visible {
		box-shadow: inset 0 0 0 2px var(--accent);
	}

	.item-thumb {
		width: 56px;
		height: 56px;
		border-radius: var(--radius-lg);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		position: relative;
		overflow: hidden;
	}

	.item-thumb.blue-gradient {
		background: linear-gradient(135deg, rgba(0, 113, 227, 0.12), rgba(0, 113, 227, 0.26));
		border: 0.5px solid rgba(0, 113, 227, 0.18);
	}

	.item-thumb.green-gradient {
		background: linear-gradient(135deg, rgba(52, 199, 89, 0.12), rgba(52, 199, 89, 0.26));
		border: 0.5px solid rgba(52, 199, 89, 0.18);
	}

	.item-thumb.purple-gradient {
		background: linear-gradient(135deg, rgba(175, 82, 222, 0.12), rgba(175, 82, 222, 0.26));
		border: 0.5px solid rgba(175, 82, 222, 0.18);
	}

	.item-thumb.orange-gradient {
		background: linear-gradient(135deg, rgba(255, 149, 0, 0.12), rgba(255, 149, 0, 0.26));
		border: 0.5px solid rgba(255, 149, 0, 0.18);
	}

	.item-thumb.red-gradient {
		background: linear-gradient(135deg, rgba(255, 59, 48, 0.12), rgba(255, 59, 48, 0.26));
		border: 0.5px solid rgba(255, 59, 48, 0.18);
	}

	.item-thumb.teal-gradient {
		background: linear-gradient(135deg, rgba(90, 200, 250, 0.12), rgba(90, 200, 250, 0.26));
		border: 0.5px solid rgba(90, 200, 250, 0.18);
	}

	:global([data-theme='dark']) .item-thumb.blue-gradient {
		background: linear-gradient(135deg, rgba(10, 132, 255, 0.2), rgba(10, 132, 255, 0.38));
		border-color: rgba(10, 132, 255, 0.3);
	}

	:global([data-theme='dark']) .item-thumb.green-gradient {
		background: linear-gradient(135deg, rgba(52, 199, 89, 0.2), rgba(52, 199, 89, 0.38));
		border-color: rgba(52, 199, 89, 0.3);
	}

	:global([data-theme='dark']) .item-thumb.purple-gradient {
		background: linear-gradient(135deg, rgba(175, 82, 222, 0.2), rgba(175, 82, 222, 0.38));
		border-color: rgba(175, 82, 222, 0.3);
	}

	:global([data-theme='dark']) .item-thumb.orange-gradient {
		background: linear-gradient(135deg, rgba(255, 149, 0, 0.2), rgba(255, 149, 0, 0.38));
		border-color: rgba(255, 149, 0, 0.3);
	}

	:global([data-theme='dark']) .item-thumb.red-gradient {
		background: linear-gradient(135deg, rgba(255, 69, 58, 0.2), rgba(255, 69, 58, 0.38));
		border-color: rgba(255, 69, 58, 0.3);
	}

	:global([data-theme='dark']) .item-thumb.teal-gradient {
		background: linear-gradient(135deg, rgba(90, 200, 250, 0.2), rgba(90, 200, 250, 0.38));
		border-color: rgba(90, 200, 250, 0.3);
	}

	.thumb-img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		border-radius: inherit;
	}

	.thumb-emoji {
		font-size: 26px;
		line-height: 1;
	}

	.unread-dot {
		position: absolute;
		top: -2px;
		right: -2px;
		width: 10px;
		height: 10px;
		border-radius: 50%;
		background: var(--accent);
		border: 2px solid var(--bg-content);
	}

	.item-content {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.item-title-line {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
	}

	.item-title {
		font-family: var(--font-sans);
		font-size: 14px;
		font-weight: 600;
		letter-spacing: -0.01em;
		line-height: 1.4;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		margin: 0;
		flex: 1;
		min-width: 0;
	}

	.feed-badge {
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		color: var(--warning);
		background: var(--fill-warning);
		padding: 2px 6px;
		border-radius: 4px;
		flex-shrink: 0;
		line-height: 1.2;
	}

	.item-excerpt {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.45;
		color: var(--text-secondary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		margin: 0;
	}

	.item-source-row {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-top: 2px;
	}

	.favicon {
		width: 14px;
		height: 14px;
		border-radius: 3px;
		flex-shrink: 0;
	}

	.favicon-fallback {
		flex-shrink: 0;
	}

	.favicon-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		display: block;
	}

	.source-domain {
		font-family: var(--font-sans);
		font-size: 11.5px;
		font-weight: 500;
		color: var(--text-secondary);
	}

	.source-meta {
		font-family: var(--font-sans);
		font-size: 11.5px;
		font-weight: 400;
		color: var(--text-tertiary);
	}

	.source-meta::before {
		content: '\00B7';
		margin-right: 6px;
	}

	.progress-bar {
		width: 120px;
		height: 3px;
		border-radius: 2px;
		background: var(--fill-hover);
		margin-top: 4px;
		overflow: hidden;
	}

	.progress-bar-fill {
		height: 100%;
		border-radius: 2px;
		background: var(--accent);
	}

	.item-meta-col {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		flex-shrink: 0;
		gap: 4px;
	}

	.item-meta-col.hidden {
		opacity: 0;
		pointer-events: none;
	}

	/* Disclosure affordance into the full-screen detail view. Rendered only when the
	   page provides onDetail (mobile widths), but hidden defensively above them. */
	.detail-chevron {
		display: none;
		width: 30px;
		height: 30px;
		align-items: center;
		justify-content: center;
		align-self: center;
		border: none;
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--text-tertiary);
		cursor: pointer;
		flex-shrink: 0;
	}

	.detail-chevron svg {
		width: 16px;
		height: 16px;
	}

	/* Mobile shows the timestamp inline in the source row instead of the meta
	   column, so the chevron can hug the edge without a dead right gutter. */
	.source-timestamp {
		display: none;
	}

	.source-timestamp:first-child::before {
		content: none;
	}

	@media (max-width: 599px) {
		.detail-chevron {
			display: flex;
			margin-left: -10px;
			margin-right: -12px;
		}

		.item-meta-col {
			display: none;
		}

		.source-timestamp {
			display: inline;
		}
	}

	.item-timestamp {
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 400;
		color: var(--text-tertiary);
		white-space: nowrap;
		margin-top: 2px;
	}

	.hover-actions-wrapper {
		position: absolute;
		right: 20px;
		top: 50%;
		transform: translateY(-50%);
		display: flex;
		align-items: center;
	}

	/* ---- Tag picker modal (cmd-palette style) ---- */
	.cmd-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.4);
		backdrop-filter: blur(4px);
		-webkit-backdrop-filter: blur(4px);
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 80px;
		z-index: 500;
		box-sizing: border-box;
	}

	:global([data-theme='dark']) .cmd-backdrop {
		background: rgba(0, 0, 0, 0.6);
	}

	.cmd-card {
		width: 460px;
		max-width: calc(100vw - 32px);
		background: var(--bg-elevated);
		border-radius: 14px;
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.22),
			0 0 0 0.5px rgba(0, 0, 0, 0.06);
	}

	:global([data-theme='dark']) .cmd-card {
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.55),
			0 0 0 0.5px rgba(255, 255, 255, 0.08);
	}

	.cmd-context-strip {
		display: flex;
		align-items: center;
		gap: 7px;
		padding: 10px 16px 8px;
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
		border-bottom: 0.5px solid var(--border-primary);
	}

	.cmd-context-strip svg {
		width: 13px;
		height: 13px;
		stroke: var(--text-tertiary);
		fill: none;
		stroke-width: 1.75;
		stroke-linecap: round;
		stroke-linejoin: round;
		flex-shrink: 0;
	}

	.cmd-item-title {
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.cmd-body {
		padding: 8px 8px 4px;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.cmd-tag-field {
		padding: 0 0 4px;
	}

	.cmd-loading {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-tertiary);
		margin: 8px 8px 4px;
	}

	.cmd-error {
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--destructive);
		margin: 0 8px;
	}

	.cmd-controls {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 16px 14px;
	}

	.cmd-secondary {
		padding: 6px 14px;
		border-radius: 980px;
		border: 1px solid var(--border-primary);
		background: transparent;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-secondary);
		cursor: pointer;
		transition: background 120ms ease;
		letter-spacing: -0.01em;
	}

	.cmd-secondary:hover {
		background: var(--fill-hover);
	}

	.cmd-action {
		margin-left: auto;
		padding: 6px 16px;
		border-radius: 980px;
		border: none;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		letter-spacing: -0.01em;
		color: var(--text-on-color);
		background: var(--accent);
		flex-shrink: 0;
		transition: opacity 120ms ease;
	}

	.cmd-action:hover:not(:disabled) {
		opacity: 0.88;
	}

	.cmd-action:disabled {
		opacity: 0.32;
		cursor: not-allowed;
	}
</style>
