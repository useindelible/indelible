import type { HighlightWithNoteResponse, TagResponse } from '$lib/api/generated/types.gen';
import type { HighlightAnchor } from './highlight-utils';
import type { AnchorContext } from '../../../../../shared/highlight-source';
import type { PdfHighlightData, PdfLocator } from './book/pdf/pdf-highlight-overlay';
import type { MessageKey } from '$lib/i18n';

export interface HighlightColorOption {
	name: string;
	hex: string;
}

export const HIGHLIGHT_COLORS: HighlightColorOption[] = [
	{ name: 'yellow', hex: '#FFD600' },
	{ name: 'blue', hex: '#0A84FF' },
	{ name: 'green', hex: '#34C759' },
	{ name: 'pink', hex: '#FF2D55' },
	{ name: 'purple', hex: '#AF52DE' }
];

export const HIGHLIGHT_COLOR_LABEL_KEYS: Record<string, MessageKey> = {
	blue: 'tag_color_blue',
	green: 'tag_color_green',
	pink: 'tag_color_pink',
	purple: 'tag_color_purple',
	yellow: 'tag_color_yellow'
};

export function normalizeTagName(value: string): string {
	return value.trim().toLowerCase();
}

export function filterTagSuggestions(
	allUserTags: TagResponse[],
	appliedTags: string[],
	queryValue: string
): TagResponse[] {
	const query = normalizeTagName(queryValue);
	return allUserTags.filter(
		(tag) =>
			!appliedTags.some(
				(appliedTag) => normalizeTagName(appliedTag) === normalizeTagName(tag.name)
			) &&
			(query === '' || normalizeTagName(tag.name).includes(query))
	);
}

export function getTagPickerPlacement(y: number, viewportHeight: number): boolean {
	const estimatedHeight = 300;
	return y + estimatedHeight > viewportHeight;
}

export function getPdfHighlightData(highlights: HighlightWithNoteResponse[]): PdfHighlightData[] {
	return highlights
		.filter((highlight) => highlight.color !== 'bookmark' && highlight.locator?.type === 'pdf')
		.map((highlight) => ({
			id: highlight.id,
			color: highlight.color,
			locator: highlight.locator as PdfLocator
		}));
}

export function getVisibleHighlightAnchors(
	highlights: HighlightWithNoteResponse[],
	locatorType: 'html' | 'epub',
	epubChapterId?: string
): HighlightAnchor[] {
	return highlights
		.filter((highlight) => highlight.color !== 'bookmark')
		.filter((highlight) => {
			if (locatorType === 'epub') {
				return highlight.locator?.type === 'epub' && highlight.locator.chapter === epubChapterId;
			}
			return !highlight.locator || highlight.locator.type === 'html';
		})
		.map((highlight) => ({
			id: highlight.id,
			color: highlight.color,
			text_content: highlight.text_content,
			locator:
				highlight.locator && 'start_offset' in highlight.locator
					? {
							start_offset: highlight.locator.start_offset ?? 0,
							end_offset: highlight.locator.end_offset ?? 0
						}
					: null,
			context: anchorContext(highlight.source_locator)
		}));
}

function anchorContext(
	source: HighlightWithNoteResponse['source_locator']
): AnchorContext | undefined {
	if (!source) return undefined;
	return {
		offset: source.offset ?? undefined,
		prefix: source.prefix ?? undefined,
		suffix: source.suffix ?? undefined
	};
}
