<script lang="ts">
	import { onDestroy } from 'svelte';
	import { goto, afterNavigate } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { browser } from '$app/environment';
	import * as apiSdk from '$lib/api';
	import type {
		DocumentListEntry,
		DocumentReaderAssetResponse,
		HighlightWithNoteResponse
	} from '$lib/api';
	import type { BookSource } from './book-source';
	import {
		createEpubSource,
		createPdfSource,
		epubChapterSequence,
		estimatePageNumber
	} from './book-source';
	import ReaderToolbar from '$lib/components/reader/ReaderToolbar.svelte';
	import TypographyPopover from '$lib/components/reader/TypographyPopover.svelte';
	import HighlightToolbar from '$lib/components/reader/HighlightToolbar.svelte';
	import BookSidebar, { type SidebarTab } from './BookSidebar.svelte';
	import PdfScrollView from './PdfScrollView.svelte';
	import EpubScrollView from './EpubScrollView.svelte';
	import BookNavBar from './BookNavBar.svelte';
	import BookDetailPanel from './BookDetailPanel.svelte';
	import { isImageOnlyPdf } from './book-reader-model';
	import { createProgressSaver } from '$lib/components/reader/progress-saver';
	import { getReaderPreferences } from '$lib/stores/reader-preferences.svelte';
	import { getViewport } from '$lib/stores/viewport.svelte';
	import { applyTheme, getSavedTheme } from '$lib/styles/theme';
	import { t } from '$lib/i18n';

	interface Props {
		item: DocumentListEntry;
		assets: DocumentReaderAssetResponse[];
		highlights: HighlightWithNoteResponse[];
		targetHighlightId?: string | null;
	}

	let { item, assets, highlights: initialHighlights, targetHighlightId = null }: Props = $props();

	const prefs = getReaderPreferences();
	const vp = getViewport();

	let source = $state<BookSource | null>(null);
	let currentIndex = $state(0);
	let pageLoading = $state(true);
	let initError = $state<string | null>(null);
	let annotationError = $state<string | null>(null);

	function annotationErrorMessage(error: unknown): string {
		if (!error || typeof error !== 'object') return $t('reader_error_save_annotation');
		const problem = error as {
			detail?: unknown;
			message?: unknown;
			errors?: Array<{ message?: unknown }>;
		};
		const fieldMessage = problem.errors?.find(
			(entry) => typeof entry.message === 'string' && entry.message.trim()
		)?.message;
		if (typeof fieldMessage === 'string') return fieldMessage;
		if (typeof problem.detail === 'string' && problem.detail.trim()) return problem.detail;
		if (
			!(error instanceof Error) &&
			typeof problem.message === 'string' &&
			problem.message.trim()
		) {
			return problem.message;
		}
		return $t('reader_error_save_annotation');
	}

	function showAnnotationError(error?: unknown) {
		annotationError = annotationErrorMessage(error);
	}

	let sidebarTab = $state<SidebarTab>('contents');
	let sidebarOpen = $state(true);
	let detailPanelOpen = $state(true);

	// Compact widths can't dock the info panel beside the book, so it becomes a
	// slide-over (tablet) or full-screen view (mobile); on mobile the TOC sidebar
	// becomes a left drawer. Both are session-only state.
	let compactDetailOpen = $state(false);
	let mobileTocOpen = $state(false);

	const detailOpen = $derived(vp.isCompact ? compactDetailOpen : detailPanelOpen);

	function toggleDetailPanel() {
		if (vp.isCompact) {
			compactDetailOpen = !compactDetailOpen;
		} else {
			detailPanelOpen = !detailPanelOpen;
		}
	}
	let activeEntryId = $state<string | null>(null);
	let highlights = $derived(initialHighlights);

	let aaButtonEl = $state<HTMLButtonElement | undefined>(undefined);
	let showTypography = $state(false);
	let chapterBodyEl = $state<HTMLDivElement | undefined>(undefined);
	let pdfScrollContainerEl = $state<HTMLDivElement | undefined>(undefined);
	let pdfScrollViewRef = $state<PdfScrollView | undefined>(undefined);
	let epubScrollContainerEl = $state<HTMLDivElement | undefined>(undefined);
	let epubScrollViewRef = $state<EpubScrollView | undefined>(undefined);

	let progressSaver = $state<ReturnType<typeof createProgressSaver> | null>(null);
	let currentCharOffset = $state(0);
	let progress = $state(item.progress_percent ?? 0);
	let backHref = $state<string | null>(null);

	afterNavigate((nav) => {
		if (nav.from?.url) {
			backHref = nav.from.url.pathname + nav.from.url.search;
		}
	});

	const isPdf = $derived(item.item_type === 'pdf');
	const textAvailable = $derived(!isImageOnlyPdf(item, assets));

	const toc = $derived(source?.toc ?? []);
	const metadata = $derived(source?.metadata ?? { totalChapters: 0 });
	const totalPages = $derived(metadata.estimatedPages ?? metadata.totalChapters);

	const currentTocEntry = $derived(toc.find((e) => e.index === currentIndex));
	const navigableEntries = $derived(epubChapterSequence(toc));
	const currentNavigableEntry = $derived(navigableEntries.find((e) => e.index === currentIndex));

	const currentPageNumber = $derived(
		(() => {
			if (isPdf) return currentIndex + 1;
			const entry = currentNavigableEntry ?? currentTocEntry;
			if (!entry) return 1;
			const totalChars = chapterBodyEl?.textContent?.length ?? 1;
			return estimatePageNumber(entry, currentCharOffset, totalChars);
		})()
	);

	const toolbarSubtitle = $derived(
		(() => {
			if (isPdf) return `Page ${currentIndex + 1}`;
			const entry = currentNavigableEntry ?? currentTocEntry;
			if (!entry) return '';
			const idx = navigableEntries.findIndex((e) => e.index === currentIndex);
			const chNum = idx >= 0 ? idx + 1 : currentIndex + 1;
			return `Ch. ${chNum}: ${entry.title}`;
		})()
	);

	const progressLabel = $derived(
		totalPages > 0 ? `Page ${currentPageNumber} of ${totalPages}` : ''
	);

	const hideDetailPanel = $derived(sidebarTab === 'search' && sidebarOpen);

	const sepiaActive = $derived(prefs.theme === 'sepia');

	$effect(() => {
		const readerTheme = prefs.theme;
		if (!browser) return;
		if (readerTheme === 'light' || readerTheme === 'sepia') {
			document.documentElement.dataset.theme = 'light';
		} else if (readerTheme === 'dark') {
			document.documentElement.dataset.theme = 'dark';
		} else if (readerTheme === 'auto') {
			applyTheme('system');
		}
		return () => {
			applyTheme(getSavedTheme());
		};
	});

	$effect(() => {
		if (browser) {
			initializeSource();
		}
	});

	$effect(() => {
		if (!browser) return;
		const saver = createProgressSaver((body) =>
			apiSdk.updateProgress({
				path: { document_id: item.id },
				body
			})
		);
		progressSaver = saver;
		const handlePageHide = () => {
			void saver.flush();
		};
		window.addEventListener('pagehide', handlePageHide);
		return () => {
			window.removeEventListener('pagehide', handlePageHide);
			saver.destroy();
			if (progressSaver === saver) progressSaver = null;
		};
	});

	onDestroy(() => {
		source?.destroy();
	});

	async function initializeSource() {
		pageLoading = true;
		initError = null;

		try {
			let nextSource: BookSource | null = null;
			let nextIndex = 0;
			const targetLocator = initialHighlights.find((h) => h.id === targetHighlightId)?.locator;

			if (item.item_type === 'book') {
				nextSource = await createEpubSource(item.id);
				const targetEntry =
					targetLocator?.type === 'epub' && targetLocator.chapter
						? nextSource.toc.find((entry) => entry.id === targetLocator.chapter)
						: undefined;

				if (targetEntry) {
					nextIndex = targetEntry.index;
					currentCharOffset = targetLocator?.start_offset ?? 0;
				} else if (item.chapter_locator) {
					const entry = nextSource.toc.find((e) => e.id === item.chapter_locator);
					if (entry) {
						nextIndex = entry.index;
						currentCharOffset = item.chapter_offset ?? 0;
					}
				}
			} else if (item.item_type === 'pdf') {
				const pdfAsset = assets.find(
					(a) =>
						a.asset_kind === 'pdf' ||
						a.asset_kind === 'original' ||
						a.asset_kind === 'original_upload'
				);
				if (!pdfAsset) throw new Error('No PDF asset found');

				const { data } = await apiSdk.streamAsset({
					path: { document_id: item.id, asset_kind: pdfAsset.asset_kind },
					parseAs: 'blob'
				});
				if (!data) throw new Error('Failed to load PDF');
				const url = URL.createObjectURL(data);
				nextSource = await createPdfSource(url, { title: item.title, author: item.author });
				const targetPage = targetLocator?.type === 'pdf' ? targetLocator.page : undefined;

				if (
					typeof targetPage === 'number' &&
					Number.isInteger(targetPage) &&
					targetPage > 0 &&
					targetPage <= nextSource.metadata.totalChapters
				) {
					nextIndex = targetPage - 1;
				} else if (item.chapter_locator?.startsWith('page:')) {
					const pageNum = parseInt(item.chapter_locator.slice(5), 10);
					if (!isNaN(pageNum) && pageNum > 0) {
						const maxIndex = Math.max(0, nextSource.metadata.totalChapters - 1);
						nextIndex = Math.min(pageNum - 1, maxIndex);
					}
				}
			}

			source = nextSource;
			currentIndex = nextIndex;
		} catch {
			initError = $t('reader_error_load_book');
		} finally {
			pageLoading = false;
		}
	}

	function handleNavigate(index: number, fragment?: string) {
		mobileTocOpen = false;
		if (isPdf) {
			currentIndex = index;
			pdfScrollViewRef?.scrollToPage(index);
		} else {
			// Set active entry from TOC click — find matching entry
			const clickedEntry =
				toc.find((e) => e.index === index && e.fragment === fragment) ??
				toc.find((e) => e.index === index);
			if (clickedEntry) activeEntryId = clickedEntry.id;

			currentIndex = index;
			currentCharOffset = 0;
			epubScrollViewRef?.scrollToChapter(index, 0, fragment);
		}
	}

	function handleBookmarkNavigate(chapterId: string, offset: number) {
		mobileTocOpen = false;
		if (chapterId.startsWith('page:')) {
			const pageNum = parseInt(chapterId.slice(5), 10);
			if (!isNaN(pageNum) && pageNum > 0) {
				handleNavigate(pageNum - 1);
			}
			return;
		}
		const entry = toc.find((e) => e.id === chapterId);
		if (entry) {
			currentIndex = entry.index;
			currentCharOffset = offset;
			epubScrollViewRef?.scrollToChapter(entry.index, offset);
		}
	}

	function handlePrevChapter() {
		if (isPdf) {
			if (currentIndex > 0) {
				currentIndex = currentIndex - 1;
				pdfScrollViewRef?.scrollToPage(currentIndex);
			}
		} else {
			const idx = navigableEntries.findIndex((e) => e.index === currentIndex);
			const prev = idx > 0 ? navigableEntries[idx - 1] : undefined;
			if (prev) {
				handleNavigate(prev.index, prev.fragment);
			}
		}
	}

	function handleNextChapter() {
		if (isPdf) {
			if (currentIndex < metadata.totalChapters - 1) {
				currentIndex = currentIndex + 1;
				pdfScrollViewRef?.scrollToPage(currentIndex);
			}
		} else {
			const idx = navigableEntries.findIndex((e) => e.index === currentIndex);
			const next =
				idx >= 0 && idx < navigableEntries.length - 1 ? navigableEntries[idx + 1] : undefined;
			if (next) {
				handleNavigate(next.index, next.fragment);
			}
		}
	}

	function handlePdfPageChange(pageIndex: number) {
		currentIndex = pageIndex;
	}

	function handlePdfProgress(pct: number, pageIndex: number) {
		currentIndex = pageIndex;
		progress = pct;
		progressSaver?.update({
			progress_percent: pct,
			chapter_locator: `page:${pageIndex + 1}`,
			chapter_offset: 0
		});
	}

	function handleEpubChapterChange(index: number) {
		currentIndex = index;
	}

	function handleActiveEntryChange(entryId: string) {
		activeEntryId = entryId;
	}

	function handleEpubProgress(pct: number, chapterIndex: number, charOffset: number) {
		currentIndex = chapterIndex;
		currentCharOffset = charOffset;
		progress = pct;
		const entry = toc.find((e) => e.index === chapterIndex);
		progressSaver?.update({
			progress_percent: pct,
			chapter_locator: entry?.id ?? null,
			chapter_offset: charOffset
		});
	}

	function handleBack() {
		void progressSaver?.flush();
		if (backHref) {
			// eslint-disable-next-line svelte/no-navigation-without-resolve -- backHref is captured from SvelteKit navigation state.
			goto(backHref);
		} else {
			goto(resolve('/library'));
		}
	}

	function handleAaClick() {
		showTypography = !showTypography;
	}

	async function handleBookmarkCreate() {
		if (!source) return;

		const visibleText =
			chapterBodyEl?.textContent?.slice(currentCharOffset, currentCharOffset + 150) ?? '';

		const locator = isPdf
			? {
					type: 'pdf' as const,
					page: currentIndex + 1,
					x: 0,
					y: 0,
					width: 0.001,
					height: 0.001,
					text_snapshot: visibleText.trim() || `Page ${currentIndex + 1}`
				}
			: {
					type: 'epub' as const,
					chapter: currentTocEntry?.id ?? '',
					start_offset: currentCharOffset,
					end_offset: currentCharOffset + 1
				};

		try {
			const { data: created, error } = await apiSdk.createHighlight({
				path: { document_id: item.id },
				body: {
					text_content: visibleText.trim() || 'Bookmark',
					color: 'bookmark',
					locator
				}
			});
			if (error) {
				showAnnotationError(error);
				return;
			}
			if (created) {
				annotationError = null;
				highlights = [...highlights, { ...created, note: null, tags: [] }];
			} else {
				showAnnotationError();
			}
		} catch (error) {
			showAnnotationError(error);
		}
	}

	async function handleHighlightCreate(data: {
		text_content: string;
		color: string;
		start_offset?: number;
		end_offset?: number;
		chapter_id?: string;
		page?: number;
		pdf_rect?: {
			x: number;
			y: number;
			width: number;
			height: number;
		};
		pdf_rects?: Array<{
			x: number;
			y: number;
			width: number;
			height: number;
		}>;
	}) {
		const locator = isPdf
			? {
					type: 'pdf' as const,
					page: data.page ?? currentIndex + 1,
					x: data.pdf_rect?.x ?? 0,
					y: data.pdf_rect?.y ?? 0,
					width: data.pdf_rect?.width ?? 0,
					height: data.pdf_rect?.height ?? 0,
					text_snapshot: data.text_content.slice(0, 100),
					rects: data.pdf_rects
				}
			: {
					type: 'epub' as const,
					chapter: data.chapter_id ?? currentTocEntry?.id ?? '',
					start_offset: data.start_offset ?? 0,
					end_offset: data.end_offset ?? 0
				};

		try {
			const { data: created, error } = await apiSdk.createHighlight({
				path: { document_id: item.id },
				body: {
					text_content: data.text_content,
					color: data.color,
					locator
				}
			});
			if (error) {
				showAnnotationError(error);
				return;
			}
			if (created) {
				annotationError = null;
				highlights = [...highlights, { ...created, note: null, tags: [] }];
			} else {
				showAnnotationError();
			}
		} catch (error) {
			showAnnotationError(error);
		}
	}

	async function handleHighlightDelete(highlightId: string) {
		try {
			const { error } = await apiSdk.deleteHighlight({ path: { highlight_id: highlightId } });
			if (error) {
				showAnnotationError(error);
				return;
			}
			annotationError = null;
			highlights = highlights.filter((h) => h.id !== highlightId);
		} catch (error) {
			showAnnotationError(error);
		}
	}

	async function handleHighlightColorChange(highlightId: string, color: string) {
		try {
			const { data: updated, error } = await apiSdk.patchHighlight({
				path: { highlight_id: highlightId },
				body: { color }
			});
			if (error) {
				showAnnotationError(error);
				return;
			}
			if (updated) {
				annotationError = null;
				highlights = highlights.map((h) =>
					h.id === highlightId ? { ...h, color: updated.color } : h
				);
			} else {
				showAnnotationError();
			}
		} catch (error) {
			showAnnotationError(error);
		}
	}

	function handleHighlightTagsChange(highlightId: string, tags: string[]) {
		highlights = highlights.map((h) => (h.id === highlightId ? { ...h, tags } : h));
	}

	async function handleHighlightCreateForTag(data: {
		text_content: string;
		color: string;
		start_offset: number;
		end_offset: number;
		chapter_id?: string;
	}): Promise<string | null> {
		const locator = {
			type: 'epub' as const,
			chapter: data.chapter_id ?? currentTocEntry?.id ?? '',
			start_offset: data.start_offset,
			end_offset: data.end_offset
		};
		try {
			const { data: created, error } = await apiSdk.createHighlight({
				path: { document_id: item.id },
				body: { text_content: data.text_content, color: data.color, locator }
			});
			if (error) {
				showAnnotationError(error);
				return null;
			}
			if (created) {
				annotationError = null;
				highlights = [...highlights, { ...created, note: null, tags: [] }];
				return created.id;
			}
			showAnnotationError();
		} catch (error) {
			showAnnotationError(error);
		}
		return null;
	}

	function handleKeydown(e: KeyboardEvent) {
		const tag = (e.target as HTMLElement)?.tagName;
		if (tag === 'INPUT' || tag === 'TEXTAREA' || (e.target as HTMLElement)?.isContentEditable)
			return;

		if (e.key === 'Escape') {
			e.preventDefault();
			if (showTypography) {
				showTypography = false;
			} else if (mobileTocOpen) {
				mobileTocOpen = false;
			} else if (compactDetailOpen) {
				compactDetailOpen = false;
			} else {
				handleBack();
			}
		}

		if (e.key === 'ArrowLeft' && !e.metaKey && !e.ctrlKey) {
			handlePrevChapter();
		}
		if (e.key === 'ArrowRight' && !e.metaKey && !e.ctrlKey) {
			handleNextChapter();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="book-reader" class:sepia-bg={sepiaActive}>
	{#if initError}
		<div class="book-error">
			<p>{initError}</p>
			<button type="button" class="error-back-btn" onclick={handleBack}
				>{$t('reader_return_to_library')}</button
			>
		</div>
	{:else}
		{#if vp.isMobile}
			{#if mobileTocOpen && source}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div class="toc-scrim" onclick={() => (mobileTocOpen = false)}></div>
				<div class="toc-drawer">
					<BookSidebar
						{source}
						{currentIndex}
						{activeEntryId}
						{progress}
						{highlights}
						activeTab={sidebarTab}
						onTabChange={(tab) => {
							sidebarTab = tab;
						}}
						onNavigate={handleNavigate}
						onBookmarkNavigate={handleBookmarkNavigate}
						thumbnailUrl={item.thumbnail_url}
						{textAvailable}
					/>
				</div>
			{/if}
		{:else if sidebarOpen && source}
			<BookSidebar
				{source}
				{currentIndex}
				{activeEntryId}
				{progress}
				{highlights}
				activeTab={sidebarTab}
				onTabChange={(tab) => {
					sidebarTab = tab;
				}}
				onNavigate={handleNavigate}
				onBookmarkNavigate={handleBookmarkNavigate}
				thumbnailUrl={item.thumbnail_url}
				{textAvailable}
			/>
		{/if}

		<div class="book-main">
			<ReaderToolbar
				{item}
				{progress}
				bookMode
				subtitle={toolbarSubtitle}
				{progressLabel}
				onBack={handleBack}
				bind:aaButtonEl
				onAaClick={handleAaClick}
				onBookmarkCreate={handleBookmarkCreate}
				detailPanelOpen={detailOpen && !hideDetailPanel}
				onDetailPanelToggle={toggleDetailPanel}
				onMenuClick={vp.isMobile && source ? () => (mobileTocOpen = true) : undefined}
				menuAriaLabel={$t('reader_open_contents')}
			/>
			{#if annotationError}
				<div class="annotation-error" role="alert">
					<span>{annotationError}</span>
					<button
						type="button"
						aria-label={$t('reader_dismiss_annotation_error')}
						onclick={() => (annotationError = null)}
					>
						&times;
					</button>
				</div>
			{/if}
			{#if !textAvailable}
				<div class="image-only-pdf-note" role="status">
					<strong>{$t('reader_pdf_no_searchable_text')}</strong>
					<span>{$t('reader_pdf_image_only_description')}</span>
				</div>
			{/if}

			<div class="reading-progress-bar">
				<div class="reading-progress-fill" style:width="{progress}%"></div>
			</div>

			{#if showTypography && aaButtonEl}
				<TypographyPopover
					anchorEl={aaButtonEl}
					onClose={() => {
						showTypography = false;
					}}
				/>
			{/if}

			<div class="content-area">
				{#if isPdf && source}
					<PdfScrollView
						bind:this={pdfScrollViewRef}
						{source}
						{highlights}
						{targetHighlightId}
						initialPage={currentIndex}
						onPageChange={handlePdfPageChange}
						onProgress={handlePdfProgress}
						bind:scrollContainerEl={pdfScrollContainerEl}
					/>
				{:else if !isPdf && source}
					<EpubScrollView
						bind:this={epubScrollViewRef}
						{source}
						{highlights}
						initialChapterIndex={currentIndex}
						initialCharOffset={currentCharOffset}
						onChapterChange={handleEpubChapterChange}
						onActiveEntryChange={handleActiveEntryChange}
						onProgress={handleEpubProgress}
						bind:scrollContainerEl={epubScrollContainerEl}
					/>
				{:else if pageLoading}
					<div class="book-page-loading">
						<span class="loading-text">{$t('common_loading')}</span>
					</div>
				{/if}

				{#if isPdf && pdfScrollContainerEl}
					<HighlightToolbar
						{highlights}
						articleBodyEl={pdfScrollContainerEl}
						pdfScrollMode
						onPdfHighlightCreate={handleHighlightCreate}
						onHighlightDelete={handleHighlightDelete}
						onHighlightColorChange={handleHighlightColorChange}
						onHighlightTagsChange={handleHighlightTagsChange}
						onHighlightCreateForTag={handleHighlightCreateForTag}
					/>
				{:else if !isPdf && epubScrollContainerEl}
					<HighlightToolbar
						{highlights}
						{targetHighlightId}
						articleBodyEl={epubScrollContainerEl}
						epubScrollMode
						onHighlightCreate={handleHighlightCreate}
						onHighlightDelete={handleHighlightDelete}
						onHighlightColorChange={handleHighlightColorChange}
						onHighlightTagsChange={handleHighlightTagsChange}
						onHighlightCreateForTag={handleHighlightCreateForTag}
					/>
				{/if}
			</div>

			<BookNavBar
				{toc}
				{currentIndex}
				totalChapters={metadata.totalChapters}
				onPrev={handlePrevChapter}
				onNext={handleNextChapter}
				{isPdf}
			/>
		</div>

		{#if vp.isCompact}
			{#if compactDetailOpen && source}
				{#if vp.isMobile}
					<div class="m-detail">
						<div class="m-detailbar">
							<button
								type="button"
								class="m-back"
								onclick={() => (compactDetailOpen = false)}
								aria-label={$t('reader_back_to_book')}
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
							<span class="m-dtitle">{item.title}</span>
						</div>
						<BookDetailPanel {item} bookMetadata={metadata} {progress} {textAvailable} />
					</div>
				{:else}
					<!-- svelte-ignore a11y_click_events_have_key_events -->
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div class="detail-scrim" onclick={() => (compactDetailOpen = false)}></div>
					<div class="detail-overlay">
						<BookDetailPanel {item} bookMetadata={metadata} {progress} {textAvailable} />
					</div>
				{/if}
			{/if}
		{:else if detailPanelOpen && !hideDetailPanel && source}
			<BookDetailPanel {item} bookMetadata={metadata} {progress} {textAvailable} />
		{/if}
	{/if}
</div>

<style>
	.book-reader {
		display: flex;
		width: 100%;
		height: 100vh;
		overflow: hidden;
		background: var(--bg-content);
		position: relative;
	}

	.book-reader.sepia-bg {
		background: #f5edda;
	}

	.book-main {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.annotation-error {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 8px 14px;
		border-bottom: 1px solid var(--border-primary);
		background: var(--bg-secondary);
		color: var(--text-primary);
		font-size: 13px;
		font-family: var(--font-sans);
		flex-shrink: 0;
	}

	.annotation-error button {
		border: 0;
		background: transparent;
		color: var(--text-secondary);
		font: inherit;
		font-size: 18px;
		line-height: 1;
		cursor: pointer;
	}

	.image-only-pdf-note {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 8px;
		padding: 8px 14px;
		border-bottom: 1px solid var(--border-primary);
		background: var(--bg-secondary);
		color: var(--text-secondary);
		font-size: 12px;
		font-family: var(--font-sans);
		line-height: 1.4;
		flex-shrink: 0;
	}

	.image-only-pdf-note strong {
		color: var(--text-primary);
		font-weight: 600;
		white-space: nowrap;
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

	.book-error {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 16px;
		width: 100%;
	}

	.book-error p {
		font-size: 14px;
		color: var(--text-secondary);
		font-family: var(--font-sans);
		margin: 0;
	}

	.error-back-btn {
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

	.error-back-btn:hover {
		background: var(--fill-hover);
	}

	/* ---- Responsive: TOC drawer + info slide-over / full-screen ---- */

	.toc-scrim {
		position: absolute;
		inset: 0;
		background: var(--overlay-backdrop, rgba(0, 0, 0, 0.34));
		z-index: 30;
	}

	.toc-drawer {
		position: absolute;
		top: 0;
		left: 0;
		bottom: 0;
		width: 300px;
		z-index: 31;
		display: flex;
		box-shadow: 18px 0 56px rgba(0, 0, 0, 0.28);
	}

	.toc-drawer > :global(.left-panel) {
		width: 100%;
		min-width: 0;
	}

	.detail-scrim {
		position: absolute;
		inset: 0;
		background: rgba(0, 0, 0, 0.1);
		z-index: 20;
	}

	/* Opaque surface: the docked panel's vibrancy blur would let the book bleed
	   through when it floats above it. */
	.detail-overlay {
		position: absolute;
		top: 0;
		right: 0;
		bottom: 0;
		width: 340px;
		z-index: 21;
		display: flex;
		background: var(--bg-elevated);
		box-shadow: -18px 0 56px rgba(0, 0, 0, 0.18);
	}

	.detail-overlay > :global(.right-panel) {
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

	.m-detail > :global(.right-panel) {
		width: 100%;
		min-width: 0;
		flex: 1;
		border-left: none;
		background: var(--bg-content);
		backdrop-filter: none;
		-webkit-backdrop-filter: none;
	}
</style>
