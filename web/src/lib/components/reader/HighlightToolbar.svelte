<script lang="ts">
	import { browser } from '$app/environment';
	import type { HighlightWithNoteResponse } from '$lib/api/generated/types.gen';
	import * as apiSdk from '$lib/api';
	import { fetchAllPages } from '$lib/api/pagination';
	import { getSelectionOffsets, applyHighlights, resolveHighlightRanges } from './highlight-utils';
	import { t } from '$lib/i18n';
	import HighlightFloatingToolbar from './HighlightFloatingToolbar.svelte';
	import HighlightContextMenu from './HighlightContextMenu.svelte';
	import HighlightTagPicker from './HighlightTagPicker.svelte';
	import {
		filterTagSuggestions,
		getPdfHighlightData,
		getTagPickerPlacement,
		getVisibleHighlightAnchors,
		HIGHLIGHT_COLORS,
		normalizeTagName
	} from './highlight-toolbar-model';
	import {
		applyHighlightTagIndicators,
		findEpubChapterFromSelection,
		findPageContextFromSelection,
		getHighlightLayerEl,
		getTextLayerEl
	} from './highlight-dom';
	import { captureSelectionRects, renderHighlightOverlays } from './book/pdf/pdf-highlight-overlay';
	import './highlight-toolbar.css';

	type PdfHighlightCreateData = {
		text_content: string;
		color: string;
		page?: number;
		pdf_rect: {
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
	};

	interface HighlightCreateData {
		text_content: string;
		color: string;
		start_offset: number;
		end_offset: number;
		chapter_id?: string;
		source_locator?: { type: 'text_quote'; prefix?: string; suffix?: string };
	}

	interface Props {
		highlights: HighlightWithNoteResponse[];
		htmlContent?: string;
		targetHighlightId?: string | null;
		articleBodyEl?: HTMLElement | null;
		onHighlightCreate?: (data: HighlightCreateData) => void;
		onPdfHighlightCreate?: (data: PdfHighlightCreateData) => void;
		onHighlightDelete: (highlightId: string) => void;
		onHighlightColorChange: (highlightId: string, color: string) => void;
		onHighlightTagsChange?: (highlightId: string, tags: string[]) => void;
		onHighlightCreateForTag?: (data: HighlightCreateData) => Promise<string | null>;
		epubChapterId?: string;
		pdfPage?: number;
		pdfScrollMode?: boolean;
		epubScrollMode?: boolean;
	}

	let {
		highlights,
		htmlContent = '',
		targetHighlightId,
		articleBodyEl = null,
		onHighlightCreate,
		onPdfHighlightCreate,
		onHighlightDelete,
		onHighlightColorChange,
		onHighlightTagsChange,
		onHighlightCreateForTag,
		epubChapterId,
		pdfPage,
		pdfScrollMode = false,
		epubScrollMode = false
	}: Props = $props();

	let showToolbar = $state(false);
	let unplacedCount = $state(0);
	let toolbarX = $state(0);
	let toolbarY = $state(0);
	let toolbarEl = $state<HTMLDivElement | undefined>(undefined);

	let showContextMenu = $state(false);
	let contextMenuX = $state(0);
	let contextMenuY = $state(0);
	let contextHighlightId = $state<string | null>(null);
	let contextHighlightColor = $state<string | null>(null);

	let showTagPicker = $state(false);
	let tagPickerX = $state(0);
	let tagPickerY = $state(0);
	let tagPickerHighlightId = $state<string | null>(null);
	let tagPickerTags = $state<string[]>([]);
	let tagInput = $state('');
	let allUserTags = $state<import('$lib/api/generated/types.gen').TagResponse[]>([]);
	let tagSuggestionIndex = $state(0);
	let tagPickerAbove = $state(false);

	const filteredSuggestions = $derived.by(() => {
		return filterTagSuggestions(allUserTags, tagPickerTags, tagInput);
	});

	let hoverHideTimer = $state<ReturnType<typeof setTimeout> | undefined>(undefined);
	let toolbarClientX = $state(0);
	let toolbarClientY = $state(0);

	function textQuote(offsets: {
		prefix?: string;
		suffix?: string;
	}): HighlightCreateData['source_locator'] {
		if (!offsets.prefix && !offsets.suffix) return undefined;
		return { type: 'text_quote', prefix: offsets.prefix, suffix: offsets.suffix };
	}

	function getArticleBody(): HTMLElement | null {
		return articleBodyEl ?? null;
	}

	let hasScrolledToTarget = $state(false);

	$effect(() => {
		void targetHighlightId;
		hasScrolledToTarget = false;
	});

	const isPdfMode = $derived(pdfPage != null || pdfScrollMode);

	function applyVisibleHighlights(container: HTMLElement) {
		if (pdfScrollMode || epubScrollMode) return;

		const locatorType = epubChapterId ? 'epub' : pdfPage != null ? 'pdf' : 'html';
		if (locatorType === 'pdf') {
			const highlightLayer = getHighlightLayerEl(container);
			if (!highlightLayer) return;

			renderHighlightOverlays(highlightLayer, getPdfHighlightData(highlights), pdfPage!);
			return;
		}

		if (!htmlContent) return;

		const resolved = resolveHighlightRanges(
			container,
			getVisibleHighlightAnchors(highlights, locatorType, epubChapterId)
		);
		unplacedCount = resolved.unplaced;
		applyHighlights(container, resolved.ranges);
		applyHighlightTagIndicators(container, highlights, HIGHLIGHT_COLORS);
	}

	$effect(() => {
		const container = getArticleBody();
		if (!container) return;
		applyVisibleHighlights(container);

		if (targetHighlightId && !hasScrolledToTarget) {
			requestAnimationFrame(() => {
				const searchTarget = pdfPage != null ? getHighlightLayerEl(container) : container;
				if (!searchTarget) return;
				const highlightEl = searchTarget.querySelector(
					`[data-highlight-id="${CSS.escape(targetHighlightId)}"]`
				);
				if (highlightEl) {
					highlightEl.scrollIntoView({ behavior: 'smooth', block: 'center' });
					(highlightEl as HTMLElement).style.transition = 'opacity 300ms ease';
					(highlightEl as HTMLElement).style.opacity = '0.5';
					setTimeout(() => {
						(highlightEl as HTMLElement).style.opacity = '1';
					}, 300);
					hasScrolledToTarget = true;
				}
			});
		}
	});

	$effect(() => {
		if (pdfScrollMode) return;
		const container = getArticleBody();
		if (!container || pdfPage == null) return;

		const handleRendered = () => {
			applyVisibleHighlights(container);
		};

		container.addEventListener('pdf-text-layer-rendered', handleRendered);
		return () => {
			container.removeEventListener('pdf-text-layer-rendered', handleRendered);
		};
	});

	// Show context menu on hover over existing highlights
	$effect(() => {
		if (!browser) return;
		const container = articleBodyEl;
		if (!container) return;

		const handleMouseOver = (e: MouseEvent) => {
			const target = e.target as HTMLElement;
			if (target.closest('.highlight-context-menu') || target.closest('.highlight-tag-picker'))
				return;
			const highlightEl = target.closest<HTMLElement>('[data-highlight-id]');
			if (!highlightEl) return;
			const highlightId = highlightEl.dataset.highlightId;
			if (!highlightId) return;
			clearTimeout(hoverHideTimer);
			const hl = highlights.find((h) => h.id === highlightId);
			contextHighlightId = highlightId;
			contextHighlightColor = hl?.color ?? null;
			contextMenuX = e.clientX;
			contextMenuY = e.clientY;
			showContextMenu = true;
		};

		const handleMouseOut = (e: MouseEvent) => {
			const target = e.target as HTMLElement;
			const relatedTarget = e.relatedTarget as HTMLElement | null;
			if (
				target.closest('[data-highlight-id]') &&
				!relatedTarget?.closest('[data-highlight-id]') &&
				!relatedTarget?.closest('.highlight-context-menu') &&
				!relatedTarget?.closest('.highlight-tag-picker')
			) {
				hoverHideTimer = setTimeout(() => {
					showContextMenu = false;
				}, 200);
			}
		};

		container.addEventListener('mouseover', handleMouseOver);
		container.addEventListener('mouseout', handleMouseOut);
		return () => {
			container.removeEventListener('mouseover', handleMouseOver);
			container.removeEventListener('mouseout', handleMouseOut);
		};
	});

	async function handleTagOnSelection() {
		const container = getArticleBody();
		if (!container || !onHighlightCreateForTag) return;

		let data: HighlightCreateData | null = null;

		if (epubScrollMode) {
			const chapterCtx = findEpubChapterFromSelection();
			if (!chapterCtx) return;
			const offsets = getSelectionOffsets(chapterCtx.bodyEl);
			if (!offsets) return;
			data = {
				text_content: offsets.text,
				color: HIGHLIGHT_COLORS[0]!.name,
				start_offset: offsets.startOffset,
				end_offset: offsets.endOffset,
				chapter_id: chapterCtx.chapterId
			};
		} else if (!isPdfMode) {
			const offsets = getSelectionOffsets(container);
			if (!offsets) return;
			data = {
				text_content: offsets.text,
				color: HIGHLIGHT_COLORS[0]!.name,
				start_offset: offsets.startOffset,
				end_offset: offsets.endOffset,
				source_locator: textQuote(offsets)
			};
		}

		if (!data) return;

		const anchorX = toolbarClientX;
		const anchorY = toolbarClientY;
		window.getSelection()?.removeAllRanges();
		showToolbar = false;

		const highlightId = await onHighlightCreateForTag(data);
		if (highlightId) {
			await openTagPicker(highlightId, anchorX, anchorY, []);
		}
	}

	function handlePointerUp(e: PointerEvent) {
		if (!browser) return;

		const container = getArticleBody();
		if (!container) return;

		// Clicks on existing highlights are handled by the hover-based context menu
		const target = e.target as HTMLElement;
		if (target.closest('[data-highlight-id]')) return;

		setTimeout(() => {
			const selection = window.getSelection();
			if (!selection || selection.rangeCount === 0) {
				showToolbar = false;
				return;
			}

			const range = selection.getRangeAt(0);
			const rect = range.getBoundingClientRect();

			let hasSelection: unknown = null;
			if (epubScrollMode) {
				const chapterCtx = findEpubChapterFromSelection();
				if (chapterCtx) {
					hasSelection = getSelectionOffsets(chapterCtx.bodyEl);
				}
			} else if (pdfScrollMode) {
				const pageCtx = findPageContextFromSelection();
				if (pageCtx) {
					hasSelection = captureSelectionRects(pageCtx.textLayer, pageCtx.wrapper);
				}
			} else if (pdfPage != null) {
				const textLayer = getTextLayerEl(container);
				if (textLayer) {
					hasSelection = captureSelectionRects(textLayer, container);
				}
			} else {
				hasSelection = getSelectionOffsets(container);
			}
			if (!hasSelection) {
				showToolbar = false;
				return;
			}

			const containerRect = container.closest('.content-area')?.getBoundingClientRect();
			if (!containerRect) return;

			toolbarX = rect.left + rect.width / 2 - containerRect.left;
			toolbarY = rect.top - containerRect.top - 10;
			toolbarClientX = rect.left + rect.width / 2;
			toolbarClientY = rect.top - 10;

			showToolbar = true;
			showContextMenu = false;

			// The toolbar centers on the selection; near the viewport edges that
			// pushes half of it off-screen on narrow widths. Clamp once rendered.
			requestAnimationFrame(() => {
				if (!toolbarEl) return;
				const half = toolbarEl.offsetWidth / 2;
				const min = half + 8;
				const max = containerRect.width - half - 8;
				toolbarX = Math.min(Math.max(toolbarX, min), Math.max(min, max));
			});
		}, 10);
	}

	function handlePointerDown(e: PointerEvent) {
		if (!browser) return;
		const target = e.target as HTMLElement;
		if (toolbarEl?.contains(target)) return;
		if (target.closest('.highlight-context-menu')) return;
		if (target.closest('.highlight-tag-picker')) return;

		if (showContextMenu) showContextMenu = false;
		if (showTagPicker) showTagPicker = false;
	}

	function handleColorClick(color: string) {
		const container = getArticleBody();
		if (!container) return;

		if (epubScrollMode) {
			const chapterCtx = findEpubChapterFromSelection();
			if (!chapterCtx) return;
			const offsets = getSelectionOffsets(chapterCtx.bodyEl);
			if (!offsets || !onHighlightCreate) return;

			onHighlightCreate({
				text_content: offsets.text,
				color,
				start_offset: offsets.startOffset,
				end_offset: offsets.endOffset,
				chapter_id: chapterCtx.chapterId
			});

			window.getSelection()?.removeAllRanges();
			showToolbar = false;
			return;
		}

		if (pdfScrollMode) {
			const pageCtx = findPageContextFromSelection();
			if (!pageCtx) return;
			const selectionResult = captureSelectionRects(pageCtx.textLayer, pageCtx.wrapper);
			if (!selectionResult) return;

			onPdfHighlightCreate?.({
				text_content: selectionResult.text,
				color,
				page: pageCtx.page,
				pdf_rect: {
					x: selectionResult.x,
					y: selectionResult.y,
					width: selectionResult.width,
					height: selectionResult.height
				},
				pdf_rects: selectionResult.rects
			});
		} else if (pdfPage != null) {
			const textLayer = getTextLayerEl(container);
			if (!textLayer) return;
			const selectionResult = captureSelectionRects(textLayer, container);
			if (!selectionResult) return;

			onPdfHighlightCreate?.({
				text_content: selectionResult.text,
				color,
				pdf_rect: {
					x: selectionResult.x,
					y: selectionResult.y,
					width: selectionResult.width,
					height: selectionResult.height
				},
				pdf_rects: selectionResult.rects
			});
		} else {
			const offsets = getSelectionOffsets(container);
			if (!offsets || !onHighlightCreate) return;

			onHighlightCreate({
				text_content: offsets.text,
				color,
				start_offset: offsets.startOffset,
				end_offset: offsets.endOffset,
				source_locator: textQuote(offsets)
			});
		}

		window.getSelection()?.removeAllRanges();
		showToolbar = false;
	}

	function handleCopy() {
		const selection = window.getSelection();
		if (selection) {
			navigator.clipboard.writeText(selection.toString()).catch(() => {});
		}
		showToolbar = false;
	}

	function handleContextColorChange(color: string) {
		if (contextHighlightId) {
			onHighlightColorChange(contextHighlightId, color);
		}
		showContextMenu = false;
	}

	function handleContextDelete() {
		if (contextHighlightId) {
			onHighlightDelete(contextHighlightId);
		}
		showContextMenu = false;
	}

	function handleContextCopy() {
		if (contextHighlightId) {
			const hl = highlights.find((h) => h.id === contextHighlightId);
			if (hl) {
				navigator.clipboard.writeText(hl.text_content).catch(() => {});
			}
		}
		showContextMenu = false;
	}

	async function openTagPicker(highlightId: string, x: number, y: number, existingTags: string[]) {
		tagPickerHighlightId = highlightId;
		tagPickerTags = [...existingTags];
		tagInput = '';
		tagSuggestionIndex = 0;
		tagPickerX = x;
		tagPickerY = y;
		tagPickerAbove = getTagPickerPlacement(y, window.innerHeight);
		showTagPicker = true;
		try {
			allUserTags = await fetchAllPages(async (cursor) => {
				const { data } = await apiSdk.listTags({
					query: { cursor, limit: 100 }
				});
				if (!data) return undefined;
				return {
					data: data.data,
					page: { next_cursor: data.page.next_cursor ?? null }
				};
			});
		} catch {
			// suggestions unavailable
		}
	}

	function handleContextTag() {
		if (!contextHighlightId) return;
		const hl = highlights.find((h) => h.id === contextHighlightId);
		showContextMenu = false;
		openTagPicker(contextHighlightId, contextMenuX, contextMenuY, hl?.tags ?? []);
	}

	async function saveTagPickerTags(tags: string[]) {
		if (!tagPickerHighlightId) return;
		try {
			const { data } = await apiSdk.setHighlightTags({
				path: { highlight_id: tagPickerHighlightId },
				body: { tags }
			});
			if (data) {
				tagPickerTags = data.tags;
				onHighlightTagsChange?.(tagPickerHighlightId, data.tags);
			}
		} catch {
			// Save failed silently
		}
	}

	async function addTag(name: string) {
		const normalized = name.trim();
		tagInput = '';
		if (
			!normalized ||
			tagPickerTags.some(
				(existingTag) => normalizeTagName(existingTag) === normalizeTagName(normalized)
			)
		) {
			return;
		}
		const newTags = [...tagPickerTags, normalized];
		tagPickerTags = newTags;
		await saveTagPickerTags(newTags);
	}

	async function removeTag(name: string) {
		const newTags = tagPickerTags.filter((t) => t !== name);
		tagPickerTags = newTags;
		await saveTagPickerTags(newTags);
	}
</script>

<svelte:window onpointerup={handlePointerUp} onpointerdown={handlePointerDown} />

{#if unplacedCount > 0}
	<p class="highlight-unplaced-notice" role="status">
		{$t('reader_highlights_unplaced', { values: { count: unplacedCount } })}
	</p>
{/if}

{#if showToolbar}
	<HighlightFloatingToolbar
		x={toolbarX}
		y={toolbarY}
		colors={HIGHLIGHT_COLORS}
		showTagAction={!isPdfMode}
		onColorClick={handleColorClick}
		onCopy={handleCopy}
		onTag={handleTagOnSelection}
		onToolbarMount={(node) => {
			toolbarEl = node;
		}}
	/>
{/if}

{#if showContextMenu}
	<HighlightContextMenu
		x={contextMenuX}
		y={contextMenuY}
		colors={HIGHLIGHT_COLORS}
		activeColor={contextHighlightColor}
		hasTags={(highlights.find((h) => h.id === contextHighlightId)?.tags?.length ?? 0) > 0}
		onColorChange={handleContextColorChange}
		onCopy={handleContextCopy}
		onTag={handleContextTag}
		onDelete={handleContextDelete}
		onMouseEnter={() => clearTimeout(hoverHideTimer)}
		onMouseLeave={() => {
			hoverHideTimer = setTimeout(() => {
				showContextMenu = false;
			}, 150);
		}}
	/>
{/if}

{#if showTagPicker}
	<HighlightTagPicker
		x={tagPickerX}
		y={tagPickerY}
		above={tagPickerAbove}
		tags={tagPickerTags}
		{tagInput}
		suggestions={filteredSuggestions}
		suggestionIndex={tagSuggestionIndex}
		onTagInputChange={(value) => {
			tagInput = value;
		}}
		onSuggestionIndexChange={(index) => {
			tagSuggestionIndex = index;
		}}
		onAddTag={addTag}
		onRemoveTag={removeTag}
		onClose={() => {
			showTagPicker = false;
		}}
	/>
{/if}
