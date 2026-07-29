<script lang="ts">
	import DOMPurify from 'dompurify';

	import type { SearchResultResponse } from '$lib/api/generated/types.gen';

	interface Props {
		result: SearchResultResponse;
		selected: boolean;
		onSelect: () => void;
		onOpen: () => void;
		onSenderClick?: (canonicalAddr: string) => void;
		onDetail?: () => void;
	}

	let { result, selected, onSelect, onOpen, onSenderClick, onDetail }: Props = $props();

	let hovered = $state(false);

	// The snippet is Postgres `ts_headline` output: it wraps matches in <mark>
	// but does NOT HTML-escape the surrounding document text (titles/body are
	// user-controlled), so it is untrusted. Allow only the <mark> highlight.
	const safeSnippet = $derived(
		result.snippet
			? DOMPurify.sanitize(result.snippet, { ALLOWED_TAGS: ['mark'], ALLOWED_ATTR: [] })
			: ''
	);

	function thumbGradient(type: string, url?: string | null): string {
		const domain = url ? getDomain(url) : '';
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

	function thumbEmoji(type: string): string {
		if (type === 'video') return '\u{1F3AC}';
		if (type === 'podcast') return '\u{1F3A7}';
		if (type === 'email') return '\u{2709}\u{FE0F}';
		if (type === 'pdf') return '\u{1F4C4}';
		if (type === 'tweet') return '\u{1F426}';
		if (type === 'book') return '\u{1F4D6}';
		return '\u{1F4F0}';
	}

	function getDomain(url: string): string {
		try {
			return new URL(url).hostname.replace('www.', '');
		} catch {
			return '';
		}
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

	function faviconUrl(domain: string): string {
		return `https://www.google.com/s2/favicons?domain=${domain}&sz=32`;
	}

	function typeBadgeClass(type: string): string {
		return `type-badge ${type}`;
	}

	function typeLabel(type: string): string {
		if (type === 'pdf') return 'PDF';
		return type.charAt(0).toUpperCase() + type.slice(1);
	}

	const gradient = $derived(thumbGradient(result.content_type, result.url));
	const emoji = $derived(thumbEmoji(result.content_type));
	const domain = $derived(result.url ? getDomain(result.url) : null);
	const timestamp = $derived(formatTimestamp(result.saved_at));
	const sectionLabel = $derived(result.section?.title ? `Ch: ${result.section.title}` : null);
	const senderLabel = $derived(
		result.sender ? (result.sender.display_name ?? result.sender.canonical_addr) : null
	);
</script>

<div
	class="result-row"
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
>
	<div class="result-thumb {gradient}" aria-hidden="true">
		<span class="thumb-emoji">{emoji}</span>
	</div>

	<div class="result-content">
		<div class="result-title-line">
			<p class="result-title">{result.title}</p>
			<span class={typeBadgeClass(result.content_type)}>{typeLabel(result.content_type)}</span>
		</div>
		{#if result.snippet}
			<!-- eslint-disable-next-line svelte/no-at-html-tags -- snippet is DOMPurify-sanitized to <mark> only -->
			<p class="result-excerpt">{@html safeSnippet}</p>
		{/if}
		<div class="result-source-row">
			{#if domain}
				<img
					src={faviconUrl(domain)}
					alt=""
					class="favicon"
					width="14"
					height="14"
					onerror={(e) => {
						(e.currentTarget as HTMLImageElement).style.display = 'none';
					}}
				/>
				<span class="result-source">{domain}</span>
			{/if}
			{#if sectionLabel}
				<span class="result-meta">{sectionLabel}</span>
			{/if}
			{#if result.sender && senderLabel}
				<button
					type="button"
					class="sender-chip"
					data-testid="search-sender-chip"
					title={`Filter by ${result.sender.canonical_addr}`}
					onclick={(e) => {
						e.stopPropagation();
						onSenderClick?.(result.sender!.canonical_addr);
					}}
					onkeydown={(e) => {
						if (e.key === 'Enter' || e.key === ' ') {
							e.stopPropagation();
						}
					}}
				>
					<svg
						class="sender-chip-icon"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="1.8"
						stroke-linecap="round"
						stroke-linejoin="round"
						aria-hidden="true"
					>
						<rect x="3" y="5" width="18" height="14" rx="2" />
						<path d="m3 7 9 6 9-6" />
					</svg>
					<span class="sender-chip-label">{senderLabel}</span>
				</button>
			{/if}
			<span class="result-meta result-timestamp-inline">{timestamp}</span>
		</div>
		{#if result.entity_chips && result.entity_chips.length > 0}
			<div class="entity-chips-row">
				<span class="entity-cooccur-label">with</span>
				{#each result.entity_chips as chip (chip.entity_id)}
					<span class="entity-chip {chip.entity_type.toLowerCase()}">{chip.name}</span>
				{/each}
			</div>
		{/if}
	</div>

	<span class="result-timestamp">{timestamp}</span>

	{#if onDetail}
		<button
			type="button"
			class="detail-chevron"
			aria-label="Show details"
			onclick={(e) => {
				e.stopPropagation();
				onDetail?.();
			}}
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
</div>

<style>
	.result-row {
		display: flex;
		align-items: flex-start;
		gap: 16px;
		padding: 14px 20px;
		border-bottom: 0.5px solid var(--border-primary);
		cursor: pointer;
		position: relative;
		transition: background 120ms ease;
	}

	.result-row:hover {
		background: var(--fill-hover);
	}

	.result-row.selected {
		background: var(--fill-selected);
	}

	.result-row.selected::before {
		content: '';
		position: absolute;
		left: 0;
		top: 8px;
		bottom: 8px;
		width: 3px;
		background: var(--accent);
		border-radius: 0 2px 2px 0;
	}

	.result-thumb {
		width: 56px;
		height: 56px;
		border-radius: 12px;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 26px;
		flex-shrink: 0;
	}

	.result-thumb.blue-gradient {
		background: linear-gradient(135deg, rgba(0, 113, 227, 0.12), rgba(0, 113, 227, 0.26));
		border: 0.5px solid rgba(0, 113, 227, 0.18);
	}
	:global([data-theme='dark']) .result-thumb.blue-gradient {
		background: linear-gradient(135deg, rgba(10, 132, 255, 0.2), rgba(10, 132, 255, 0.38));
		border: 0.5px solid rgba(10, 132, 255, 0.3);
	}
	.result-thumb.green-gradient {
		background: linear-gradient(135deg, rgba(52, 199, 89, 0.12), rgba(52, 199, 89, 0.26));
		border: 0.5px solid rgba(52, 199, 89, 0.18);
	}
	:global([data-theme='dark']) .result-thumb.green-gradient {
		background: linear-gradient(135deg, rgba(52, 199, 89, 0.2), rgba(52, 199, 89, 0.38));
		border: 0.5px solid rgba(52, 199, 89, 0.3);
	}
	.result-thumb.purple-gradient {
		background: linear-gradient(135deg, rgba(175, 82, 222, 0.12), rgba(175, 82, 222, 0.26));
		border: 0.5px solid rgba(175, 82, 222, 0.18);
	}
	:global([data-theme='dark']) .result-thumb.purple-gradient {
		background: linear-gradient(135deg, rgba(175, 82, 222, 0.2), rgba(175, 82, 222, 0.38));
		border: 0.5px solid rgba(175, 82, 222, 0.3);
	}
	.result-thumb.orange-gradient {
		background: linear-gradient(135deg, rgba(255, 149, 0, 0.12), rgba(255, 149, 0, 0.26));
		border: 0.5px solid rgba(255, 149, 0, 0.18);
	}
	:global([data-theme='dark']) .result-thumb.orange-gradient {
		background: linear-gradient(135deg, rgba(255, 149, 0, 0.2), rgba(255, 149, 0, 0.38));
		border: 0.5px solid rgba(255, 149, 0, 0.3);
	}
	.result-thumb.red-gradient {
		background: linear-gradient(135deg, rgba(255, 59, 48, 0.12), rgba(255, 59, 48, 0.26));
		border: 0.5px solid rgba(255, 59, 48, 0.18);
	}
	:global([data-theme='dark']) .result-thumb.red-gradient {
		background: linear-gradient(135deg, rgba(255, 69, 58, 0.2), rgba(255, 69, 58, 0.38));
		border: 0.5px solid rgba(255, 69, 58, 0.3);
	}
	.result-thumb.teal-gradient {
		background: linear-gradient(135deg, rgba(90, 200, 250, 0.12), rgba(90, 200, 250, 0.26));
		border: 0.5px solid rgba(90, 200, 250, 0.18);
	}
	:global([data-theme='dark']) .result-thumb.teal-gradient {
		background: linear-gradient(135deg, rgba(90, 200, 250, 0.2), rgba(90, 200, 250, 0.38));
		border: 0.5px solid rgba(90, 200, 250, 0.3);
	}

	.thumb-emoji {
		user-select: none;
	}

	.result-content {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.result-title-line {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
	}

	.result-title {
		font-family: var(--font-sans);
		font-size: 14px;
		font-weight: 600;
		letter-spacing: -0.01em;
		line-height: 1.4;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		flex: 1;
		min-width: 0;
	}

	.result-excerpt {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.45;
		color: var(--text-secondary);
		display: -webkit-box;
		-webkit-line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}

	.result-excerpt :global(mark) {
		background: var(--highlight-yellow-bg, rgba(255, 214, 0, 0.22));
		color: inherit;
		padding: 1px 2px;
		border-radius: 2px;
	}

	.result-source-row {
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

	.result-source {
		font-family: var(--font-sans);
		font-size: 11.5px;
		font-weight: 500;
		color: var(--text-secondary);
	}

	.result-meta {
		font-family: var(--font-sans);
		font-size: 11.5px;
		font-weight: 400;
		color: var(--text-tertiary);
	}

	.result-meta::before {
		content: '\00b7';
		margin: 0 4px;
	}

	.sender-chip {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 1px 8px 1px 6px;
		margin-left: 6px;
		border: 0.5px solid var(--border-primary);
		border-radius: 980px;
		background: var(--bg-elevated);
		color: var(--text-secondary);
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 500;
		letter-spacing: -0.005em;
		cursor: pointer;
		transition:
			background 120ms ease,
			color 120ms ease,
			border-color 120ms ease;
		max-width: 200px;
	}

	.sender-chip:hover,
	.sender-chip:focus-visible {
		background: var(--fill-selected);
		color: var(--text-primary);
		border-color: var(--border-secondary);
		outline: none;
	}

	.sender-chip-icon {
		width: 11px;
		height: 11px;
		flex-shrink: 0;
		color: var(--text-tertiary);
	}

	.sender-chip-label {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.result-timestamp {
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 400;
		color: var(--text-tertiary);
		white-space: nowrap;
		flex-shrink: 0;
		align-self: flex-start;
		margin-top: 2px;
	}

	.type-badge {
		display: inline-flex;
		align-items: center;
		padding: 2px 6px;
		border-radius: 4px;
		font-family: var(--font-sans);
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		flex-shrink: 0;
	}

	.type-badge.article {
		background: rgba(0, 113, 227, 0.08);
		color: var(--accent);
	}
	.type-badge.book {
		background: rgba(175, 82, 222, 0.08);
		color: #af52de;
	}
	:global([data-theme='dark']) .type-badge.book {
		background: rgba(175, 82, 222, 0.15);
		color: #bf5af2;
	}
	.type-badge.pdf {
		background: rgba(255, 59, 48, 0.08);
		color: #ff3b30;
	}
	:global([data-theme='dark']) .type-badge.pdf {
		background: rgba(255, 69, 58, 0.15);
		color: #ff453a;
	}
	.type-badge.video {
		background: rgba(255, 149, 0, 0.08);
		color: #ff9500;
	}
	:global([data-theme='dark']) .type-badge.video {
		background: rgba(255, 159, 10, 0.15);
		color: #ff9f0a;
	}
	.type-badge.tweet {
		background: rgba(90, 200, 250, 0.08);
		color: #5ac8fa;
	}
	:global([data-theme='dark']) .type-badge.tweet {
		background: rgba(90, 200, 250, 0.15);
		color: #64d2ff;
	}
	.type-badge.email {
		background: rgba(255, 149, 0, 0.08);
		color: #ff9500;
	}
	:global([data-theme='dark']) .type-badge.email {
		background: rgba(255, 159, 10, 0.15);
		color: #ff9f0a;
	}
	.type-badge.podcast {
		background: rgba(175, 82, 222, 0.08);
		color: #af52de;
	}
	:global([data-theme='dark']) .type-badge.podcast {
		background: rgba(175, 82, 222, 0.15);
		color: #bf5af2;
	}

	.entity-chips-row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 4px;
		margin-top: 4px;
	}

	.entity-cooccur-label {
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 500;
		color: var(--text-quaternary);
		margin-right: 2px;
	}

	.entity-chip {
		display: inline-flex;
		align-items: center;
		padding: 2px 8px;
		border-radius: 980px;
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 500;
		letter-spacing: -0.005em;
	}

	.entity-chip.person {
		background: var(--entity-person-bg);
		color: var(--entity-person-text);
	}

	.entity-chip.organization {
		background: var(--entity-org-bg);
		color: var(--entity-org-text);
	}

	.entity-chip.location {
		background: var(--entity-location-bg);
		color: var(--entity-location-text);
	}

	.entity-chip.event {
		background: var(--entity-event-bg);
		color: var(--entity-event-text);
	}

	.entity-chip.work {
		background: var(--entity-work-bg);
		color: var(--entity-work-text);
	}

	/* ---- Responsive: mobile reflow ---- */

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

	/* Mobile shows the timestamp inline in the source row instead of the right
	   column, so the chevron can hug the edge without a dead right gutter. */
	.result-timestamp-inline {
		display: none;
	}

	.result-timestamp-inline:first-child::before {
		content: none;
	}

	@media (max-width: 599px) {
		.result-row {
			padding: 14px 16px;
			gap: 12px;
		}

		.detail-chevron {
			display: flex;
			margin-left: -10px;
			margin-right: -12px;
		}

		.result-timestamp {
			display: none;
		}

		.result-timestamp-inline {
			display: inline;
		}
	}
</style>
