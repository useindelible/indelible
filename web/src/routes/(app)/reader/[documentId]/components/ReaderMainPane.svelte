<script lang="ts">
	import type { DocumentListEntry, HighlightWithNoteResponse } from '$lib/api';
	import type { BookSource } from '$lib/components/reader/book/book-source';
	import PdfScrollView from '$lib/components/reader/book/PdfScrollView.svelte';
	import type { ViewTab } from '$lib/components/reader/ViewTabs.svelte';
	import ReaderToolbar from '$lib/components/reader/ReaderToolbar.svelte';
	import ReaderContent from '$lib/components/reader/ReaderContent.svelte';
	import HighlightToolbar from '$lib/components/reader/HighlightToolbar.svelte';
	import OriginalContent from '$lib/components/reader/OriginalContent.svelte';
	import ScreenshotContent from '$lib/components/reader/ScreenshotContent.svelte';
	import ReaderFloatingControls from './ReaderFloatingControls.svelte';
	import TocRail from '$lib/components/reader/toc/TocRail.svelte';
	import {
		currentEntryTops,
		resolveActiveIndex,
		resolveEntryTargets
	} from '$lib/components/reader/toc/active-section';
	import type { ArticleTocEntry } from '$lib/api';
	import type { ReaderFailurePresentation } from '../reader-page-model';

	type FocusState = 'idle' | 'selecting' | 'active' | 'paused' | 'completed';
	type HighlightCreateInput = {
		text_content: string;
		color: string;
		start_offset: number;
		end_offset: number;
	};

	interface Props {
		documentId: string;
		item: DocumentListEntry;
		progress: number;
		sepiaActive: boolean;
		hasPrev: boolean;
		hasNext: boolean;
		aaButtonEl: HTMLButtonElement | undefined;
		readerArticleBodyEl: HTMLDivElement | undefined;
		tocEntries?: ArticleTocEntry[];
		activeTab: ViewTab;
		availableTabs: ViewTab[];
		ttsOpen: boolean;
		showTypography: boolean;
		showDetailPanel: boolean;
		savedToLibrary: boolean;
		savingToLibrary: boolean;
		readableReady: boolean;
		readerFailure: ReaderFailurePresentation | null;
		showReaderRetry: boolean;
		readerRetryError: string | null;
		readerRetryStatus: string | null;
		readerRetryOutcome: string | null;
		readerRetryLabel: string;
		readerRetryDisabled: boolean;
		transcriptUnavailable?: boolean;
		assetUrls: Partial<Record<ViewTab, string>>;
		readerHtmlContent: string;
		highlights: HighlightWithNoteResponse[];
		targetHighlightId: string | null;
		articlePdfSource: BookSource | null;
		articlePdfInitialPage: number;
		focusModeState: FocusState;
		focusStartProgress: number;
		focusHighlightsCreated: number;
		onBack: () => void;
		onPrev: () => void;
		onNext: () => void;
		onAaClick: () => void;
		onBookmark?: () => void;
		onSaveToLibrary?: () => void;
		onDetailPanelToggle: () => void;
		onMenuClick?: () => void;
		onTabChange: (tab: ViewTab) => void;
		onTtsToggle?: () => void;
		onRetryReader: () => void;
		onProgressScroll: (percent: number) => void;
		onArticlePdfProgress: (percent: number, pageIndex: number) => void;
		onHighlightCreate: (data: HighlightCreateInput) => void;
		onHighlightDelete: (highlightId: string) => void;
		onHighlightColorChange: (highlightId: string, color: string) => void;
		onHighlightTagsChange: (highlightId: string, tags: string[]) => void;
		onHighlightCreateForTag: (
			data: HighlightCreateInput & { chapter_id?: string }
		) => Promise<string | null>;
		onTypographyClose: () => void;
		onFocusStart: () => void;
		onFocusPause: () => void;
		onFocusResume: () => void;
		onFocusComplete: () => void;
		onFocusExit: () => void;
	}

	let {
		documentId,
		item,
		progress,
		sepiaActive,
		hasPrev,
		hasNext,
		aaButtonEl = $bindable(),
		readerArticleBodyEl = $bindable(),
		tocEntries = [],
		activeTab,
		availableTabs,
		ttsOpen,
		showTypography,
		showDetailPanel,
		savedToLibrary,
		savingToLibrary,
		readableReady,
		readerFailure,
		showReaderRetry,
		readerRetryError,
		readerRetryStatus,
		readerRetryOutcome,
		readerRetryLabel,
		readerRetryDisabled,
		transcriptUnavailable = false,
		assetUrls,
		readerHtmlContent,
		highlights,
		targetHighlightId,
		articlePdfSource,
		articlePdfInitialPage,
		focusModeState,
		focusStartProgress,
		focusHighlightsCreated,
		onBack,
		onPrev,
		onNext,
		onAaClick,
		onBookmark,
		onSaveToLibrary,
		onDetailPanelToggle,
		onMenuClick,
		onTabChange,
		onTtsToggle,
		onRetryReader,
		onProgressScroll,
		onArticlePdfProgress,
		onHighlightCreate,
		onHighlightDelete,
		onHighlightColorChange,
		onHighlightTagsChange,
		onHighlightCreateForTag,
		onTypographyClose,
		onFocusStart,
		onFocusPause,
		onFocusResume,
		onFocusComplete,
		onFocusExit
	}: Props = $props();

	let readerScrollEl = $state<HTMLDivElement | undefined>(undefined);
	let tocActiveIndex = $state(-1);
	let diagnosticCopyStatus = $state<string | null>(null);
	const showToc = $derived(tocEntries.length > 0 && activeTab === 'reader');

	$effect(() => {
		void documentId;
		diagnosticCopyStatus = null;
	});

	// Active-section tracking: scroll-geometry against the reader's own scroll
	// container, recomputed per scroll frame. Runs whenever the rail is visible
	// so the current tick stays lit while reading. Targets are snapshotted per
	// run, and the article body element outlives its async content — so the
	// effect must also key on the HTML itself or it caches null targets from
	// before the article rendered.
	$effect(() => {
		const scrollEl = readerScrollEl;
		const articleBody = readerArticleBodyEl;
		const html = readerHtmlContent;
		if (!showToc || !html || !scrollEl || !articleBody || tocEntries.length === 0) return;
		const targets = resolveEntryTargets(articleBody, tocEntries);
		let frame = 0;
		const update = () => {
			tocActiveIndex = resolveActiveIndex(currentEntryTops(scrollEl, targets), scrollEl.scrollTop);
		};
		const onScroll = () => {
			cancelAnimationFrame(frame);
			frame = requestAnimationFrame(update);
		};
		update();
		scrollEl.addEventListener('scroll', onScroll, { passive: true });
		return () => {
			cancelAnimationFrame(frame);
			scrollEl.removeEventListener('scroll', onScroll);
		};
	});

	function handleTocNavigate(entry: ArticleTocEntry) {
		const scrollEl = readerScrollEl;
		const articleBody = readerArticleBodyEl;
		if (!scrollEl || !articleBody) return;
		const target = resolveEntryTargets(articleBody, [entry])[0];
		if (!target) return;
		const top = currentEntryTops(scrollEl, [target])[0];
		if (top == null) return;
		scrollEl.scrollTo({ top: Math.max(0, top - 16), behavior: 'smooth' });
	}

	async function copyDiagnosticId() {
		if (!readerFailure) return;
		try {
			if (navigator.clipboard?.writeText) {
				await navigator.clipboard.writeText(readerFailure.diagnosticId);
			} else {
				copyWithSelection(readerFailure.diagnosticId);
			}
			diagnosticCopyStatus = 'Diagnostic ID copied.';
		} catch {
			try {
				copyWithSelection(readerFailure.diagnosticId);
				diagnosticCopyStatus = 'Diagnostic ID copied.';
			} catch {
				diagnosticCopyStatus = 'Could not copy diagnostic ID.';
			}
		}
	}

	function copyWithSelection(text: string) {
		const textarea = document.createElement('textarea');
		textarea.value = text;
		textarea.setAttribute('readonly', '');
		textarea.style.position = 'fixed';
		textarea.style.opacity = '0';
		document.body.appendChild(textarea);
		textarea.select();
		let copied = false;
		try {
			copied = document.execCommand('copy');
		} finally {
			textarea.remove();
		}
		if (!copied) throw new Error('Copy command failed');
	}

	function formatAttemptTime(value: string): string {
		const date = new Date(value);
		return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
	}
</script>

<div class="reader-main" class:sepia-bg={sepiaActive}>
	<span class="sr-only" role="status" aria-live="polite" data-testid="reader-retry-outcome"
		>{readerRetryOutcome ?? ''}</span
	>
	<ReaderToolbar
		{item}
		{progress}
		{hasPrev}
		{hasNext}
		bind:aaButtonEl
		isFavorite={item.is_favorite}
		{savedToLibrary}
		{savingToLibrary}
		detailPanelOpen={showDetailPanel}
		{availableTabs}
		{activeTab}
		ttsActive={ttsOpen}
		{onBack}
		{onPrev}
		{onNext}
		{onAaClick}
		{onBookmark}
		{onSaveToLibrary}
		{onDetailPanelToggle}
		{onMenuClick}
		{onTabChange}
		{onTtsToggle}
	/>

	<div class="reading-progress-bar">
		<div class="reading-progress-fill" style:width="{progress}%"></div>
	</div>

	<ReaderFloatingControls
		{documentId}
		{activeTab}
		{readableReady}
		{ttsOpen}
		{readerArticleBodyEl}
		{showTypography}
		{aaButtonEl}
		{focusModeState}
		{focusStartProgress}
		{progress}
		{focusHighlightsCreated}
		{onTypographyClose}
		{onFocusStart}
		{onFocusPause}
		{onFocusResume}
		{onFocusComplete}
		{onFocusExit}
	/>

	<div class="content-area" class:with-toc={showToc}>
		{#if activeTab === 'reader' && !readableReady}
			<div class="content-loading" data-testid="preparing-reader">
				<span class="loading-text">
					{readerFailure ? readerFailure.title : 'Preparing readable content...'}
				</span>
				{#if showReaderRetry || readerRetryStatus}
					{#if showReaderRetry}
						<p class="loading-hint">
							{readerFailure ? readerFailure.guidance : 'This is taking longer than usual.'}
						</p>
					{/if}
					{#if readerFailure}
						<div class="failure-diagnostics">
							<div class="failure-meta">
								<span>
									Attempted
									<time datetime={readerFailure.attemptedAt} data-testid="reader-failure-attempt"
										>{formatAttemptTime(readerFailure.attemptedAt)}</time
									>
								</span>
								<span class="diagnostic-id">
									Diagnostic ID <code>{readerFailure.diagnosticId}</code>
									<button type="button" onclick={copyDiagnosticId} aria-label="Copy diagnostic ID"
										>Copy</button
									>
									{#if diagnosticCopyStatus}
										<span
											class="diagnostic-copy-status"
											role="status"
											aria-live="polite"
											data-testid="reader-diagnostic-copy-status">{diagnosticCopyStatus}</span
										>
									{/if}
								</span>
							</div>
							{#if readerFailure.technicalReason}
								<details>
									<summary>Technical details</summary>
									<code>{readerFailure.technicalReason}</code>
								</details>
							{/if}
						</div>
					{/if}
					{#if readerRetryError}
						<p class="loading-hint retry-error">{readerRetryError}</p>
					{/if}
					{#if readerRetryStatus}
						<p class="loading-hint" aria-live="polite">{readerRetryStatus}</p>
					{/if}
					<button
						type="button"
						class="retry-button"
						data-testid="reader-retry"
						disabled={readerRetryDisabled}
						onclick={onRetryReader}
					>
						{readerRetryLabel}
					</button>
				{/if}
			</div>
		{:else if activeTab === 'reader'}
			{#if transcriptUnavailable}
				<div class="transcript-notice" role="status">
					<strong>No transcript available</strong>
					<span>
						The video embed, metadata, and description remain available. Chat and Listen need a
						transcript, so they are unavailable for this video.
					</span>
				</div>
			{/if}
			<ReaderContent
				htmlContent={readerHtmlContent}
				title={item.title}
				author={item.author}
				domain={item.domain}
				publishedAt={item.published_at}
				readingTimeMinutes={item.reading_time_minutes}
				onScroll={onProgressScroll}
				initialProgress={progress}
				bind:articleBodyEl={readerArticleBodyEl}
				bind:scrollEl={readerScrollEl}
			/>
			<HighlightToolbar
				{highlights}
				htmlContent={readerHtmlContent}
				{targetHighlightId}
				articleBodyEl={readerArticleBodyEl}
				{onHighlightCreate}
				{onHighlightDelete}
				{onHighlightColorChange}
				{onHighlightTagsChange}
				{onHighlightCreateForTag}
			/>
			{#if showToc}
				<TocRail
					entries={tocEntries}
					activeIndex={tocActiveIndex}
					{progress}
					onNavigate={handleTocNavigate}
				/>
			{/if}
		{:else if activeTab === 'original' && assetUrls.original}
			<OriginalContent downloadUrl={assetUrls.original} />
		{:else if activeTab === 'pdf' && articlePdfSource}
			<PdfScrollView
				source={articlePdfSource}
				{highlights}
				initialPage={articlePdfInitialPage}
				onProgress={onArticlePdfProgress}
			/>
		{:else if activeTab === 'screenshot' && assetUrls.screenshot}
			<ScreenshotContent downloadUrl={assetUrls.screenshot} />
		{:else}
			<div class="content-loading">
				<span class="loading-text">Loading content...</span>
			</div>
		{/if}
	</div>
</div>

<style>
	.reader-main {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		background: var(--bg-content);
		overflow: hidden;
		position: relative;
		z-index: 1;
	}

	.reader-main.sepia-bg {
		background: var(--reader-sepia-bg);
	}

	.reading-progress-bar {
		height: 2px;
		background: var(--border-primary);
		position: relative;
		z-index: 4;
		flex-shrink: 0;
	}

	.reading-progress-fill {
		height: 100%;
		background: var(--accent);
		border-radius: 0 1px 1px 0;
		transition: width 300ms ease;
	}

	.content-area {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		position: relative;
	}

	.content-area.with-toc :global(.reader-scroll) {
		padding-inline: 56px;
	}

	.transcript-notice {
		display: flex;
		align-items: baseline;
		gap: 8px;
		padding: 9px 16px;
		border-bottom: 1px solid var(--border-primary);
		background: var(--fill-selected);
		font-family: var(--font-sans);
		font-size: 12px;
		line-height: 1.4;
		color: var(--text-secondary);
	}

	.transcript-notice strong {
		flex-shrink: 0;
		color: var(--text-primary);
	}

	@media (max-width: 620px) {
		.transcript-notice {
			align-items: flex-start;
			flex-direction: column;
			gap: 2px;
		}
	}

	.content-loading {
		flex: 1;
		min-height: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: safe center;
		gap: 16px;
		overflow-y: auto;
		padding-block: 24px;
		box-sizing: border-box;
	}

	.loading-text {
		font-size: 14px;
		color: var(--text-secondary);
		font-family: var(--font-sans);
	}

	.loading-hint {
		font-size: 13px;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
		margin: 0;
	}

	.retry-error {
		color: var(--destructive);
	}

	.failure-diagnostics {
		width: min(520px, calc(100% - 48px));
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--text-tertiary);
	}

	.failure-meta {
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		gap: 8px 16px;
	}

	.failure-meta time,
	.diagnostic-id code {
		margin-left: 4px;
		color: var(--text-secondary);
	}

	.diagnostic-id button {
		margin-left: 6px;
		padding: 0;
		border: 0;
		background: transparent;
		color: var(--accent);
		font: inherit;
		cursor: pointer;
	}

	.diagnostic-copy-status {
		margin-left: 6px;
		color: var(--text-secondary);
	}

	.failure-diagnostics details {
		margin-top: 10px;
		text-align: left;
	}

	.failure-diagnostics summary {
		cursor: pointer;
		text-align: center;
	}

	.failure-diagnostics details code {
		display: block;
		margin-top: 8px;
		overflow-wrap: anywhere;
		color: var(--text-secondary);
	}

	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border: 0;
	}

	.retry-button {
		font-size: 13px;
		font-weight: 500;
		color: var(--accent);
		background: none;
		border: 1px solid var(--border-primary);
		border-radius: 980px;
		padding: 8px 20px;
		cursor: pointer;
		font-family: var(--font-sans);
		transition: background 120ms ease;
	}

	.retry-button:hover {
		background: var(--fill-hover);
	}

	.retry-button:disabled {
		cursor: default;
		opacity: 0.6;
	}
</style>
