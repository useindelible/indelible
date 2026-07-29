import { describe, expect, it } from 'vitest';

import {
	elementTopWithin,
	resolveActiveIndex,
	resolveEntryTargets
} from '../../src/lib/components/reader/toc/active-section';

function entry(index: number, id: string) {
	return { source_heading_index: index, id, title: `H${index}`, depth: 0, word_count: 100 };
}

describe('resolveActiveIndex', () => {
	it('picks the last heading at or above the scroll position (24px slack)', () => {
		const tops = [0, 400, 900];
		expect(resolveActiveIndex(tops, 0)).toBe(0);
		expect(resolveActiveIndex(tops, 375)).toBe(0);
		expect(resolveActiveIndex(tops, 376)).toBe(1);
		expect(resolveActiveIndex(tops, 5000)).toBe(2);
	});

	it('is -1 before the first heading and skips unresolvable targets', () => {
		expect(resolveActiveIndex([300, null, 900], 0)).toBe(-1);
		expect(resolveActiveIndex([300, null, 900], 950)).toBe(2);
	});
});

describe('resolveEntryTargets', () => {
	it('resolves by id inside the article only, with ordinal fallback', () => {
		document.body.innerHTML = `
			<div id="ind-toc-outside">decoy outside the article</div>
			<article id="body">
				<h2>Title-ish</h2>
				<h2 id="ind-toc-real">Real</h2>
				<h2>Bare</h2>
			</article>`;
		const articleBody = document.getElementById('body') as HTMLElement;

		// Entry 1 has an id present in the article; entry 2 falls back to the
		// heading ordinal (cached HTML without ids); an entry whose id exists
		// only OUTSIDE the article must not resolve to the decoy.
		const targets = resolveEntryTargets(articleBody, [
			entry(1, 'ind-toc-real'),
			entry(2, 'ind-toc-missing'),
			entry(9, 'ind-toc-outside')
		]);
		expect(targets[0]?.textContent).toBe('Real');
		expect(targets[1]?.textContent).toBe('Bare');
		expect(targets[2]).toBeNull();
	});

	it('mirrors the deduped-title case: first entry ordinal 1 never targets the dropped title heading', () => {
		document.body.innerHTML = `
			<article id="body"><h2>The Dropped Title</h2><h2>Supplies</h2><h2>Step 1</h2></article>`;
		const articleBody = document.getElementById('body') as HTMLElement;
		const targets = resolveEntryTargets(articleBody, [entry(1, ''), entry(2, '')]);
		expect(targets[0]?.textContent).toBe('Supplies');
		expect(targets[1]?.textContent).toBe('Step 1');
	});

	it('handles ids containing colons via CSS.escape', () => {
		document.body.innerHTML = `<article id="body"><h2 id="ind-fn:1">Note</h2></article>`;
		const articleBody = document.getElementById('body') as HTMLElement;
		const targets = resolveEntryTargets(articleBody, [entry(0, 'ind-fn:1')]);
		expect(targets[0]?.textContent).toBe('Note');
	});
});

describe('elementTopWithin', () => {
	it('offsets element position into the scroll container coordinate space', () => {
		const scrollRect = { top: 100 } as DOMRect;
		const elRect = { top: 250 } as DOMRect;
		expect(elementTopWithin(scrollRect, elRect, 40)).toBe(190);
	});
});
