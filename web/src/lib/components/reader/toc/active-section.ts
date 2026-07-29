import type { ArticleTocEntry } from '$lib/api';

/**
 * Resolve each ToC entry to its heading element inside the article body.
 *
 * Lookup is scoped to the article (a page-global getElementById could resolve
 * an unrelated app element), and ids may contain `:` so they are CSS-escaped.
 * Entries whose id is absent — cached HTML stored before anchor injection —
 * fall back to the document-order heading ordinal the backend recorded.
 */
// jsdom (tests) lacks CSS.escape; the fallback escapes every non-ident char.
function escapeCssId(id: string): string {
	if (typeof CSS !== 'undefined' && typeof CSS.escape === 'function') return CSS.escape(id);
	return id.replace(/[^a-zA-Z0-9_-]/g, (char) => `\\${char}`);
}

export function resolveEntryTargets(
	articleBody: HTMLElement,
	entries: ArticleTocEntry[]
): (HTMLElement | null)[] {
	const headings = articleBody.querySelectorAll('h1, h2, h3, h4, h5, h6');
	return entries.map((entry) => {
		if (entry.id) {
			const el = articleBody.querySelector<HTMLElement>(`#${escapeCssId(entry.id)}`);
			if (el) return el;
		}
		return (headings[entry.source_heading_index] as HTMLElement | undefined) ?? null;
	});
}

/**
 * Index of the active entry: the last one whose top sits at or above the
 * current scroll position (with a little slack so a heading flush with the
 * viewport top counts as entered). -1 while still in the preamble.
 */
export function resolveActiveIndex(
	tops: (number | null)[],
	scrollTop: number,
	slackPx = 24
): number {
	let active = -1;
	tops.forEach((top, index) => {
		if (top !== null && top <= scrollTop + slackPx) active = index;
	});
	return active;
}

/** Element top translated into the scroll container's scrollTop space. */
export function elementTopWithin(
	scrollRect: Pick<DOMRect, 'top'>,
	elementRect: Pick<DOMRect, 'top'>,
	scrollTop: number
): number {
	return elementRect.top - scrollRect.top + scrollTop;
}

export function currentEntryTops(
	scrollEl: HTMLElement,
	targets: (HTMLElement | null)[]
): (number | null)[] {
	const scrollRect = scrollEl.getBoundingClientRect();
	return targets.map((el) =>
		el ? elementTopWithin(scrollRect, el.getBoundingClientRect(), scrollEl.scrollTop) : null
	);
}
