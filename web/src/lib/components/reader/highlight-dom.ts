import type { HighlightWithNoteResponse } from '$lib/api/generated/types.gen';
import type { HighlightColorOption } from './highlight-toolbar-model';

export interface PdfPageSelectionContext {
	wrapper: HTMLElement;
	textLayer: HTMLElement;
	highlightLayer: HTMLElement;
	page: number;
}

export interface EpubChapterSelectionContext {
	bodyEl: HTMLElement;
	chapterId: string;
}

export function getTextLayerEl(container: HTMLElement): HTMLElement | null {
	return container.querySelector('.textLayer');
}

export function getHighlightLayerEl(container: HTMLElement): HTMLElement | null {
	return container.querySelector('.pdf-highlight-layer');
}

export function findPageContextFromSelection(
	selection: Selection | null = window.getSelection()
): PdfPageSelectionContext | null {
	if (!selection || selection.rangeCount === 0) return null;
	const range = selection.getRangeAt(0);
	const node =
		range.startContainer instanceof HTMLElement
			? range.startContainer
			: range.startContainer.parentElement;
	const wrapper = node?.closest('[data-pdf-page]') as HTMLElement;
	if (!wrapper) return null;
	const textLayer = wrapper.querySelector('.textLayer') as HTMLElement;
	const highlightLayer = wrapper.querySelector('.pdf-highlight-layer') as HTMLElement;
	if (!textLayer || !highlightLayer) return null;
	return {
		wrapper,
		textLayer,
		highlightLayer,
		page: parseInt(wrapper.dataset.pdfPage ?? '1', 10)
	};
}

export function findEpubChapterFromSelection(
	selection: Selection | null = window.getSelection()
): EpubChapterSelectionContext | null {
	if (!selection || selection.rangeCount === 0) return null;
	const range = selection.getRangeAt(0);
	const node =
		range.startContainer instanceof HTMLElement
			? range.startContainer
			: range.startContainer.parentElement;
	const wrapper = node?.closest('[data-chapter-id]') as HTMLElement;
	if (!wrapper) return null;
	const bodyEl = wrapper.querySelector('.book-body') as HTMLElement;
	if (!bodyEl) return null;
	return { bodyEl, chapterId: wrapper.dataset.chapterId ?? '' };
}

export function applyHighlightTagIndicators(
	container: HTMLElement,
	highlights: HighlightWithNoteResponse[],
	colors: HighlightColorOption[]
): void {
	container.querySelectorAll('.hl-body-tag').forEach((el) => el.remove());

	const allSpans = container.querySelectorAll<HTMLElement>('[data-highlight-id]');
	const lastSpanForId = new Map<string, HTMLElement>();
	for (const span of allSpans) {
		const id = span.dataset.highlightId;
		if (id) lastSpanForId.set(id, span);
	}

	for (const [id, span] of lastSpanForId) {
		const highlight = highlights.find((entry) => entry.id === id);
		if (!highlight?.tags?.length) continue;

		const indicator = document.createElement('span');
		indicator.className = 'hl-body-tag';

		const dot = document.createElement('span');
		dot.className = 'hl-body-tag-dot';
		const colorHex = colors.find((color) => color.name === highlight.color)?.hex;
		if (colorHex) dot.style.background = colorHex;
		indicator.appendChild(dot);

		const label =
			highlight.tags.length === 1
				? highlight.tags[0]!
				: `${highlight.tags[0]} +${highlight.tags.length - 1}`;
		indicator.appendChild(document.createTextNode(label));

		span.after(indicator);
	}
}
