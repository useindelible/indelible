<script lang="ts">
	import { SvelteMap } from 'svelte/reactivity';
	import type { BookSource, TocEntry } from './book-source';
	import { estimatePageNumber } from './book-source';

	interface SearchResult {
		chapterIndex: number;
		chapterTitle: string;
		tocEntry: TocEntry | undefined;
		charOffset: number;
		context: string;
		matchStart: number;
		matchEnd: number;
	}

	interface Props {
		source: BookSource;
		onNavigate: (chapterIndex: number) => void;
	}

	let { source, onNavigate }: Props = $props();

	let query = $state('');
	let results = $state<SearchResult[]>([]);
	let searching = $state(false);
	let hasSearched = $state(false);
	let chapterTexts = new SvelteMap<number, string>();
	let searchTimeout: ReturnType<typeof setTimeout> | undefined;

	async function loadAllChapters() {
		if (chapterTexts.size >= source.metadata.totalChapters) return;

		const texts = new SvelteMap<number, string>();
		for (let i = 0; i < source.metadata.totalChapters; i++) {
			try {
				const page = await source.loadPage(i);
				if (page.type === 'html') {
					// Parse inertly for text extraction: DOMParser does not execute scripts or load
					// resources, so untrusted chapter HTML cannot fire onerror/onload here.
					const doc = new DOMParser().parseFromString(page.html, 'text/html');
					texts.set(i, doc.body.textContent ?? '');
				} else if (page.type === 'pdf') {
					const textContent = await page.page.getTextContent();
					const items = textContent.items.filter(
						(item): item is import('pdfjs-dist/types/src/display/api').TextItem => 'str' in item
					);
					texts.set(i, items.map((t) => t.str).join(' '));
				}
			} catch {
				// Skip chapters that fail to load
			}
		}
		chapterTexts = texts;
	}

	async function performSearch() {
		if (!query.trim()) {
			results = [];
			hasSearched = false;
			return;
		}

		searching = true;
		hasSearched = true;

		await loadAllChapters();

		const q = query.toLowerCase();
		const found: SearchResult[] = [];

		for (const [chapterIdx, text] of chapterTexts) {
			const textLower = text.toLowerCase();
			let searchFrom = 0;

			while (searchFrom < textLower.length) {
				const matchIdx = textLower.indexOf(q, searchFrom);
				if (matchIdx === -1) break;

				const contextStart = Math.max(0, matchIdx - 40);
				const contextEnd = Math.min(text.length, matchIdx + q.length + 40);
				const context =
					(contextStart > 0 ? '...' : '') +
					text.slice(contextStart, contextEnd) +
					(contextEnd < text.length ? '...' : '');

				const tocEntry = source.toc.find((e) => e.index === chapterIdx);

				found.push({
					chapterIndex: chapterIdx,
					chapterTitle: tocEntry?.title ?? `Chapter ${chapterIdx + 1}`,
					tocEntry,
					charOffset: matchIdx,
					context,
					matchStart: matchIdx - contextStart + (contextStart > 0 ? 3 : 0),
					matchEnd: matchIdx - contextStart + q.length + (contextStart > 0 ? 3 : 0)
				});

				searchFrom = matchIdx + q.length;
			}
		}

		results = found;
		searching = false;
	}

	function handleInput() {
		if (searchTimeout) clearTimeout(searchTimeout);
		searchTimeout = setTimeout(() => {
			performSearch();
		}, 300);
	}

	function getPageLabel(result: SearchResult): string {
		if (!result.tocEntry) return '';
		const deep = source.toc.filter((e) => e.depth >= 2);
		const navigable = deep.length > 0 ? deep : source.toc;
		const chapterNum = navigable.indexOf(result.tocEntry) + 1;
		const chapterTotalChars = chapterTexts.get(result.chapterIndex)?.length ?? 1;
		const page = estimatePageNumber(result.tocEntry, result.charOffset, chapterTotalChars);
		return `Ch. ${chapterNum} \u00B7 Page ${page}`;
	}

	function highlightMatch(context: string, start: number, end: number): string {
		const before = context.slice(0, start);
		const match = context.slice(start, end);
		const after = context.slice(end);
		return `${escapeHtml(before)}<mark>${escapeHtml(match)}</mark>${escapeHtml(after)}`;
	}

	function escapeHtml(text: string): string {
		return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
	}
</script>

<div class="search-panel">
	<div class="search-input-wrap">
		<svg
			class="search-input-icon"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
			><circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" /></svg
		>
		<input
			class="search-input"
			type="text"
			bind:value={query}
			oninput={handleInput}
			placeholder="Search in book..."
		/>
	</div>

	{#if searching}
		<div class="search-status">Searching...</div>
	{:else if hasSearched}
		<div class="search-results-count">
			{results.length} result{results.length !== 1 ? 's' : ''} for "{query}"
		</div>
	{/if}

	{#each results as result (result.chapterIndex + '-' + result.charOffset)}
		<button
			type="button"
			class="search-result-item"
			onclick={() => onNavigate(result.chapterIndex)}
		>
			<div class="search-result-chapter">{getPageLabel(result)}</div>
			<div class="search-result-text">
				<!-- eslint-disable-next-line svelte/no-at-html-tags -- search results are escaped via escapeHtml utility -->
				{@html highlightMatch(result.context, result.matchStart, result.matchEnd)}
			</div>
		</button>
	{/each}
</div>

<style>
	.search-panel {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.search-input-wrap {
		position: relative;
		padding: 0 12px;
		margin-bottom: 8px;
	}

	.search-input-icon {
		position: absolute;
		left: 20px;
		top: 50%;
		transform: translateY(-50%);
		width: 14px;
		height: 14px;
		color: var(--text-tertiary);
		pointer-events: none;
	}

	.search-input {
		width: 100%;
		height: 32px;
		padding: 0 10px 0 30px;
		border-radius: 8px;
		border: 0.5px solid var(--border-primary);
		background: var(--fill-hover);
		font-size: 13px;
		font-weight: 400;
		color: var(--text-primary);
		outline: none;
		font-family: var(--font-sans);
		transition: border-color 120ms ease;
	}

	.search-input:focus {
		border-color: var(--accent);
	}

	.search-input::placeholder {
		color: var(--text-tertiary);
	}

	.search-status {
		padding: 4px 16px;
		font-size: 12px;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
	}

	.search-results-count {
		padding: 4px 16px 8px;
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
		font-family: var(--font-sans);
	}

	.search-result-item {
		display: flex;
		flex-direction: column;
		gap: 3px;
		padding: 8px 16px;
		border: none;
		background: none;
		width: 100%;
		text-align: left;
		cursor: pointer;
		transition: background 120ms ease;
		font-family: var(--font-sans);
	}

	.search-result-item:hover {
		background: var(--fill-hover);
	}

	.search-result-chapter {
		font-size: 11px;
		font-weight: 600;
		color: var(--text-tertiary);
		letter-spacing: 0.02em;
	}

	.search-result-text {
		font-size: 12px;
		font-weight: 400;
		color: var(--text-secondary);
		line-height: 1.45;
	}

	.search-result-text :global(mark) {
		background: var(--highlight-yellow-bg, rgba(255, 214, 0, 0.3));
		color: var(--text-primary);
		border-radius: 2px;
		padding: 0 1px;
	}
</style>
