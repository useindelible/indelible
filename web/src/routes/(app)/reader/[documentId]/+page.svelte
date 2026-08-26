<script lang="ts">
	import { browser } from '$app/environment';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onDestroy, untrack } from 'svelte';
	import * as apiSdk from '$lib/api';
	import type {
		DocumentListEntry,
		DocumentReaderAssetResponse,
		HighlightWithNoteResponse
	} from '$lib/api';
	import BookReader from '$lib/components/reader/book/BookReader.svelte';
	import { createPdfSource, type BookSource } from '$lib/components/reader/book/book-source';
	import { createProgressSaver } from '$lib/components/reader/progress-saver';
	import type { ViewTab } from '$lib/components/reader/ViewTabs.svelte';
	import LibraryShell from '$lib/components/library/LibraryShell.svelte';
	import LibrarySidebar from '$lib/components/library/LibrarySidebar.svelte';
	import { getLibrary } from '$lib/stores/library.svelte';
	import { getReaderPreferences } from '$lib/stores/reader-preferences.svelte';
	import { getViewport } from '$lib/stores/viewport.svelte';
	import { applyTheme, getSavedTheme } from '$lib/styles/theme';
	import { t } from '$lib/i18n';
	import AiFailureNotice from './components/AiFailureNotice.svelte';
	import ReaderErrorState from './components/ReaderErrorState.svelte';
	import ReaderCompactDetail from './components/ReaderCompactDetail.svelte';
	import ReaderLoadingState from './components/ReaderLoadingState.svelte';
	import ReaderMainPane from './components/ReaderMainPane.svelte';
	import {
		READER_ASSET_KIND_BY_TAB,
		revokeReaderAssetUrls,
		shouldLoadReaderAsset
	} from './reader-assets';
	import { subscribeReaderRealtime, type ReaderAiFailure } from './reader-realtime';
	import { ReaderChromeController } from './reader-chrome.svelte';
	import {
		READER_VIEW_TABS,
		computeArticlePdfInitialPage,
		computeAvailableReaderTabs,
		isBookReaderItem,
		isReaderContentReady,
		isTranscriptUnavailableVideo,
		isSavedToLibrary,
		readerFailurePresentation,
		shouldReprocessReaderPreparation
	} from './reader-page-model';
	import { createReaderPollController } from './reader-poll';
	import { ReaderRetryController } from './reader-retry.svelte';
	import { createTocStore } from './toc-store.svelte';

	const prefs = getReaderPreferences();
	const lib = getLibrary();
	const vp = getViewport();
	const documentId = $derived(page.params.documentId ?? '');
	const targetHighlightId = $derived(page.url.searchParams.get('highlight'));

	let item = $state<DocumentListEntry | null>(null);
	let assets = $state<DocumentReaderAssetResponse[]>([]);
	let highlights = $state<HighlightWithNoteResponse[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let aiFailure = $state<ReaderAiFailure | null>(null);
	let aiRetryStatus = $state<'idle' | 'pending' | 'queued' | 'error'>('idle');
	const readerRetry = new ReaderRetryController((key, options) => $t(key, options));
	const showReaderRetry = $derived(
		readerRetry.pollVisible || shouldReprocessReaderPreparation(item, assets)
	);

	let activeTab = $state<ViewTab>('reader');
	let progress = $state(0);
	let progressSaver = $state<ReturnType<typeof createProgressSaver> | null>(null);
	let readerHtmlContent = $state<string>('');
	let assetUrls = $state<Partial<Record<ViewTab, string>>>({});
	const readerChrome = new ReaderChromeController();

	let tocStore = $state<ReturnType<typeof createTocStore> | null>(null);
	$effect(() => {
		if (!browser || !documentId) return;
		const store = createTocStore(documentId);
		tocStore = store;
		store.start();
		return () => store.stop();
	});

	async function retryMilaAction() {
		if (!aiFailure || aiRetryStatus === 'pending') return;
		const retriedFailure = aiFailure;
		const isCurrentFailure = () =>
			aiFailure?.documentId === retriedFailure.documentId &&
			aiFailure.action === retriedFailure.action &&
			aiFailure.aiRunId === retriedFailure.aiRunId;
		aiRetryStatus = 'pending';
		try {
			const { data, error } = await apiSdk.retryMilaDocumentAction({
				path: { document_id: retriedFailure.documentId, action: retriedFailure.action }
			});
			if (!isCurrentFailure()) return;
			if (error || !data?.queued) throw new Error('Retry was not accepted');
			aiRetryStatus = 'queued';
		} catch {
			if (isCurrentFailure()) aiRetryStatus = 'error';
		}
	}
	const tocEntries = $derived(
		tocStore && tocStore.state.kind === 'ready' ? tocStore.state.entries : []
	);

	let aaButtonEl = $state<HTMLButtonElement | undefined>(undefined);
	let readerArticleBodyEl = $state<HTMLDivElement | undefined>(undefined);
	let showTypography = $state(false);

	const detailOpen = $derived(readerChrome.detailOpen(vp.isCompact));

	let articlePdfSource = $state<BookSource | null>(null);
	let articlePdfLoading = $state(false);

	let focusModeState = $state<'idle' | 'selecting' | 'active' | 'paused' | 'completed'>('idle');
	let focusStartProgress = $state(0);
	let focusHighlightsCreated = $state(0);
	let ttsOpen = $state(false);
	let savingToLibrary = $state(false);

	let highlightReloadTimer: ReturnType<typeof setTimeout> | undefined;
	let loadEpoch = 0;
	const readerPoll = createReaderPollController({
		canPoll: () => browser,
		onPoll: () => void loadData({ silent: true }),
		onRetryVisibleChange: (visible) => {
			readerRetry.pollVisible = visible;
		}
	});
	const libItems = $derived(lib.items);
	const currentIndex = $derived(libItems.findIndex((candidate) => candidate.id === documentId));
	const hasPrev = $derived(currentIndex > 0);
	const hasNext = $derived(currentIndex >= 0 && currentIndex < libItems.length - 1);
	const isBookItem = $derived(isBookReaderItem(item));
	const readableReady = $derived(isReaderContentReady(item, assets));
	const readerFailure = $derived(readerFailurePresentation($t, assets));
	const transcriptUnavailable = $derived(isTranscriptUnavailableVideo(item, assets));
	$effect(() => {
		if (transcriptUnavailable) ttsOpen = false;
	});
	const savedToLibrary = $derived(isSavedToLibrary(item));
	const availableTabs = $derived(computeAvailableReaderTabs(assets));
	const resolvedActiveTab = $derived(
		availableTabs.includes(activeTab) ? activeTab : (availableTabs[0] ?? activeTab)
	);
	const articlePdfInitialPage = $derived(
		computeArticlePdfInitialPage(
			articlePdfSource?.metadata.totalChapters,
			item?.chapter_locator,
			progress
		)
	);
	const sepiaActive = $derived(prefs.theme === 'sepia');
	function handlePrev() {
		if (!hasPrev) return;
		void progressSaver?.flush();
		const prev = libItems[currentIndex - 1];
		if (prev) goto(resolve('/(app)/reader/[documentId]', { documentId: prev.id }));
	}

	function handleNext() {
		if (!hasNext) return;
		void progressSaver?.flush();
		const next = libItems[currentIndex + 1];
		if (next) goto(resolve('/(app)/reader/[documentId]', { documentId: next.id }));
	}
	async function handleBookmark() {
		if (!item) return;
		try {
			const { data } = await apiSdk.toggleFavorite({ path: { document_id: documentId } });
			if (data) item = data;
		} catch {
			// Ignore.
		}
	}
	async function handleSaveToLibrary() {
		if (!item?.url || savingToLibrary) return;
		savingToLibrary = true;
		try {
			const { data } = await apiSdk.createDocumentEntry({
				body: {
					url: item.url,
					title: item.title,
					item_type: item.document_type
				}
			});
			if (data) {
				item = data;
				lib.updateItemInList(data);
			} else {
				await loadData({ silent: true });
			}
		} finally {
			savingToLibrary = false;
		}
	}

	async function loadData(options: { silent?: boolean } = {}) {
		const requestedDocumentId = documentId;
		const requestedLoadEpoch = ++loadEpoch;
		const isCurrentLoad = () =>
			requestedDocumentId === documentId && requestedLoadEpoch === loadEpoch;
		if (!options.silent) {
			loading = true;
			readerPoll.reset();
		}
		error = null;
		try {
			const [itemRes, assetsRes, highlightsRes] = await Promise.all([
				apiSdk.getDocumentEntry({ path: { document_id: requestedDocumentId } }),
				apiSdk.listAssets({ path: { document_id: requestedDocumentId } }),
				apiSdk.listHighlights({ path: { document_id: requestedDocumentId } })
			]);
			if (!isCurrentLoad()) return;
			if (itemRes.data) {
				item = itemRes.data;
				progress = item.progress_percent ?? 0;
			}
			if (assetsRes.data) {
				assets = assetsRes.data.data;
			}
			readerRetry.error = null;
			if (highlightsRes.data) {
				highlights = highlightsRes.data.highlights;
			}
			const ready = isReaderContentReady(itemRes.data ?? null, assetsRes.data?.data ?? []);
			readerRetry.onPreparationReady(ready);
			readerPoll.schedule(ready);
		} catch {
			if (!isCurrentLoad()) return;
			if (options.silent) {
				// A transient poll failure must not strand the reader on the spinner: keep polling
				// so it recovers once preparation finishes (or the network blip clears).
				readerPoll.schedule(false);
			} else {
				error = $t('reader_error_load_item');
			}
		} finally {
			if (isCurrentLoad()) loading = false;
		}
	}

	async function retryReader() {
		await readerRetry.submit({
			documentId,
			item,
			assets,
			onRetryPolling: () => readerPoll.retry()
		});
	}

	onDestroy(() => {
		readerRetry.destroy();
		if (highlightReloadTimer) clearTimeout(highlightReloadTimer);
		readerPoll.destroy();
		revokeReaderAssetUrls(assetUrls);
		articlePdfSource?.destroy();
		articlePdfSource = null;
	});

	async function reloadHighlights() {
		if (!documentId) return;
		try {
			const { data } = await apiSdk.listHighlights({ path: { document_id: documentId } });
			if (data) {
				highlights = data.highlights;
			}
		} catch {
			// Keep current reader state; the next realtime event or reload can recover.
		}
	}

	function scheduleHighlightReload() {
		if (highlightReloadTimer) clearTimeout(highlightReloadTimer);
		highlightReloadTimer = setTimeout(() => {
			highlightReloadTimer = undefined;
			void reloadHighlights();
		}, 250);
	}

	$effect(() => {
		if (browser && documentId) {
			const tabParam = untrack(() => page.url.searchParams.get('tab')) as ViewTab | null;
			if (tabParam && READER_VIEW_TABS.includes(tabParam)) {
				activeTab = tabParam;
			}
			loadData();
		}
	});

	$effect(() => {
		if (!browser || !documentId) return;
		const saver = createProgressSaver((body) =>
			apiSdk.updateProgress({
				path: { document_id: documentId },
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

	$effect(() => {
		if (!browser || !documentId) return;
		return subscribeReaderRealtime(documentId, {
			onHighlightsChanged: scheduleHighlightReload,
			onAiCompleted: (completion) => {
				if (aiFailure?.action === completion.action) {
					aiFailure = null;
					aiRetryStatus = 'idle';
				}
				void loadData({ silent: true });
			},
			onAiFailed: (failure) => {
				aiFailure = failure;
				aiRetryStatus = 'idle';
				void loadData({ silent: true });
			}
		});
	});

	$effect(() => {
		const pdfUrl = assetUrls.pdf;
		if (!pdfUrl || articlePdfSource || articlePdfLoading) return;
		articlePdfLoading = true;
		createPdfSource(pdfUrl, { title: item?.title, author: item?.author }).then(
			(source) => {
				articlePdfSource = source;
				articlePdfLoading = false;
			},
			() => {
				articlePdfLoading = false;
			}
		);
	});

	async function loadAssetContent(tab: ViewTab) {
		if (!shouldLoadReaderAsset(tab, readerHtmlContent, assetUrls)) return;
		try {
			if (tab === 'reader') {
				const { data } = await apiSdk.streamAsset({
					path: { document_id: documentId, asset_kind: READER_ASSET_KIND_BY_TAB[tab] },
					parseAs: 'text'
				});
				if (data) {
					readerHtmlContent = data;
				}
			} else {
				const { data } = await apiSdk.streamAsset({
					path: { document_id: documentId, asset_kind: READER_ASSET_KIND_BY_TAB[tab] },
					parseAs: 'blob'
				});
				if (data) {
					assetUrls = { ...assetUrls, [tab]: URL.createObjectURL(data) };
				}
			}
		} catch {
			// Asset loading failed gracefully.
		}
	}

	$effect(() => {
		if (browser && !isBookItem && availableTabs.includes(resolvedActiveTab)) {
			loadAssetContent(resolvedActiveTab);
		}
	});

	function handleProgressScroll(percent: number) {
		progress = percent;
		progressSaver?.update({ progress_percent: percent });
	}

	function handleArticlePdfProgress(percent: number, pageIndex: number) {
		progress = percent;
		progressSaver?.update({
			progress_percent: percent,
			chapter_locator: `page:${pageIndex + 1}`,
			chapter_offset: 0
		});
	}

	function handleBack() {
		void progressSaver?.flush();
		if (readerChrome.backHref) {
			// eslint-disable-next-line svelte/no-navigation-without-resolve -- backHref is captured from SvelteKit navigation state.
			goto(readerChrome.backHref);
		} else {
			goto(resolve('/library'));
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		const target = event.target as HTMLElement;
		const tag = target?.tagName;
		if (tag === 'INPUT' || tag === 'TEXTAREA' || target?.isContentEditable) return;

		if (event.key === 'Escape') {
			event.preventDefault();
			if (showTypography) {
				showTypography = false;
			} else {
				handleBack();
			}
			return;
		}

		if (event.key === 'f' || event.key === 'F') {
			if (!event.metaKey && !event.ctrlKey && !event.altKey) {
				event.preventDefault();
				handleFocusModeToggle();
			}
		}
	}

	async function handleHighlightCreate(data: {
		text_content: string;
		color: string;
		start_offset: number;
		end_offset: number;
		source_locator?: { type: 'text_quote'; prefix?: string; suffix?: string };
	}) {
		try {
			const { data: created } = await apiSdk.createHighlight({
				path: { document_id: documentId },
				body: {
					text_content: data.text_content,
					color: data.color,
					locator: {
						type: 'html' as const,
						start_offset: data.start_offset,
						end_offset: data.end_offset
					},
					source_locator: data.source_locator
				}
			});
			if (created) {
				highlights = [...highlights, { ...created, note: null, tags: [] }];
				if (focusModeState === 'active') {
					focusHighlightsCreated++;
				}
			}
		} catch {
			// Highlight creation failed.
		}
	}

	async function handleHighlightDelete(highlightId: string) {
		try {
			await apiSdk.deleteHighlight({
				path: { highlight_id: highlightId }
			});
			highlights = highlights.filter((highlight) => highlight.id !== highlightId);
		} catch {
			// Deletion failed.
		}
	}

	async function handleHighlightColorChange(highlightId: string, color: string) {
		try {
			const { data: updated } = await apiSdk.patchHighlight({
				path: { highlight_id: highlightId },
				body: { color }
			});
			if (updated) {
				highlights = highlights.map((highlight) =>
					highlight.id === highlightId ? { ...highlight, color: updated.color } : highlight
				);
			}
		} catch {
			// Color change failed.
		}
	}

	function handleHighlightTagsChange(highlightId: string, tags: string[]) {
		highlights = highlights.map((highlight) =>
			highlight.id === highlightId ? { ...highlight, tags } : highlight
		);
	}

	async function handleHighlightCreateForTag(data: {
		text_content: string;
		color: string;
		start_offset: number;
		end_offset: number;
		chapter_id?: string;
		source_locator?: { type: 'text_quote'; prefix?: string; suffix?: string };
	}): Promise<string | null> {
		try {
			const { data: created } = await apiSdk.createHighlight({
				path: { document_id: documentId },
				body: {
					text_content: data.text_content,
					color: data.color,
					locator: {
						type: 'html' as const,
						start_offset: data.start_offset,
						end_offset: data.end_offset
					},
					source_locator: data.source_locator
				}
			});
			if (created) {
				highlights = [...highlights, { ...created, note: null, tags: [] }];
				return created.id;
			}
		} catch {
			// Ignore.
		}
		return null;
	}

	function handleFocusModeToggle() {
		if (focusModeState === 'idle') {
			focusModeState = 'selecting';
			focusStartProgress = progress;
			focusHighlightsCreated = 0;
		} else if (focusModeState === 'active' || focusModeState === 'paused') {
			focusModeState = 'completed';
		} else if (focusModeState === 'selecting') {
			focusModeState = 'idle';
		} else if (focusModeState === 'completed') {
			focusModeState = 'idle';
		}
	}

	$effect(() => {
		void documentId;
		untrack(() => {
			ttsOpen = false;
			aiFailure = null;
			aiRetryStatus = 'idle';
			readerRetry.reset();
		});
	});

	$effect(() => {
		if (isBookItem) return;
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
</script>

<svelte:window onkeydown={isBookItem ? undefined : handleKeydown} />

{#if aiFailure}
	<AiFailureNotice
		failure={aiFailure}
		status={aiRetryStatus}
		onRetry={() => void retryMilaAction()}
		onDismiss={() => {
			aiFailure = null;
			aiRetryStatus = 'idle';
		}}
	/>
{/if}

{#if loading}
	<ReaderLoadingState />
{:else if error || !item}
	<ReaderErrorState message={error ?? $t('reader_item_not_found')} onBack={handleBack} />
{:else if isBookItem}
	<BookReader {item} {assets} {highlights} {targetHighlightId} />
{:else}
	{#snippet sidebar()}
		<LibrarySidebar />
	{/snippet}

	{#snippet content()}
		{#if item}
			<ReaderMainPane
				{documentId}
				{item}
				{progress}
				{sepiaActive}
				{hasPrev}
				{hasNext}
				bind:aaButtonEl
				bind:readerArticleBodyEl
				{tocEntries}
				activeTab={resolvedActiveTab}
				{availableTabs}
				{ttsOpen}
				{showTypography}
				showDetailPanel={detailOpen}
				{savedToLibrary}
				{savingToLibrary}
				{readableReady}
				{readerFailure}
				{showReaderRetry}
				readerRetryError={readerRetry.error}
				readerRetryStatus={readerRetry.status}
				readerRetryOutcome={readerRetry.outcome}
				readerRetryLabel={readerRetry.label}
				readerRetryDisabled={readerRetry.disabled}
				{transcriptUnavailable}
				{assetUrls}
				{readerHtmlContent}
				{highlights}
				{targetHighlightId}
				{articlePdfSource}
				{articlePdfInitialPage}
				{focusModeState}
				{focusStartProgress}
				{focusHighlightsCreated}
				onBack={handleBack}
				onPrev={handlePrev}
				onNext={handleNext}
				onAaClick={() => (showTypography = !showTypography)}
				onBookmark={savedToLibrary ? handleBookmark : undefined}
				onSaveToLibrary={item.url && !savedToLibrary ? handleSaveToLibrary : undefined}
				onDetailPanelToggle={() => readerChrome.toggleDetailPanel(vp.isCompact)}
				onMenuClick={vp.isMobile ? () => vp.openMobileNav() : undefined}
				onTabChange={(tab) => {
					void progressSaver?.flush();
					activeTab = tab;
				}}
				onTtsToggle={resolvedActiveTab === 'reader' && readableReady && !transcriptUnavailable
					? () => (ttsOpen = !ttsOpen)
					: undefined}
				onRetryReader={retryReader}
				onProgressScroll={handleProgressScroll}
				onArticlePdfProgress={handleArticlePdfProgress}
				onHighlightCreate={handleHighlightCreate}
				onHighlightDelete={handleHighlightDelete}
				onHighlightColorChange={handleHighlightColorChange}
				onHighlightTagsChange={handleHighlightTagsChange}
				onHighlightCreateForTag={handleHighlightCreateForTag}
				onTypographyClose={() => (showTypography = false)}
				onFocusStart={() => (focusModeState = 'active')}
				onFocusPause={() => (focusModeState = 'paused')}
				onFocusResume={() => (focusModeState = 'active')}
				onFocusComplete={() => (focusModeState = 'completed')}
				onFocusExit={() => (focusModeState = 'idle')}
			/>

			<ReaderCompactDetail
				{item}
				isCompact={vp.isCompact}
				isMobile={vp.isMobile}
				compactDetailOpen={readerChrome.compactDetailOpen}
				showDetailPanel={detailOpen}
				chatAvailable={!transcriptUnavailable}
				onClose={() => (readerChrome.compactDetailOpen = false)}
			/>
		{/if}
	{/snippet}

	<LibraryShell {sidebar} {content} />
{/if}
