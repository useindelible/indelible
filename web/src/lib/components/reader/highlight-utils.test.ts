import { describe, expect, it, vi } from 'vitest';

import { getSelectionOffsets, resolveHighlightRanges } from './highlight-utils';

function container(html: string): HTMLElement {
	const el = document.createElement('div');
	el.innerHTML = html;
	document.body.appendChild(el);
	return el;
}

describe('resolveHighlightRanges', () => {
	it('keeps a locator whose text still matches', () => {
		const el = container('<p>Hello brave world.</p>');
		const { ranges, unplaced } = resolveHighlightRanges(el, [
			{
				id: 'h1',
				color: 'yellow',
				text_content: 'brave',
				locator: { start_offset: 6, end_offset: 11 }
			}
		]);
		expect(ranges[0]).toMatchObject({ id: 'h1', startOffset: 6, endOffset: 11 });
		expect(unplaced).toBe(0);
	});

	it('falls back to text search when the locator is null', () => {
		const el = container('<p>Later edited in a final form.<sup>40</sup> French theorist.</p>');
		const { ranges } = resolveHighlightRanges(el, [
			{ id: 'h2', color: 'yellow', text_content: 'edited in a final form.[35]', locator: null }
		]);
		const range = ranges[0]!;
		expect(el.textContent!.slice(range.startOffset, range.endOffset)).toBe(
			'edited in a final form.40'
		);
	});

	it('falls back when the locator points at different text', () => {
		const el = container('<p>Alpha target. Beta target.</p>');
		const { ranges } = resolveHighlightRanges(el, [
			{
				id: 'h3',
				color: 'yellow',
				text_content: 'Beta target',
				locator: { start_offset: 0, end_offset: 11 }
			}
		]);
		expect(ranges[0]!.startOffset).toBe('Alpha target. '.length);
	});

	it('counts an unhinted repeat as unplaced and logs the stage', () => {
		const debug = vi.spyOn(console, 'debug').mockImplementation(() => {});
		const el = container('<p>Alpha target. Beta target.</p>');
		const result = resolveHighlightRanges(el, [
			{ id: 'h4', color: 'yellow', text_content: 'target', locator: null }
		]);
		expect(result).toEqual({ ranges: [], unplaced: 1 });
		expect(debug).toHaveBeenCalledWith('[reader] highlight not placed', {
			id: 'h4',
			stage: 'ambiguous'
		});
		debug.mockRestore();
	});
});

describe('getSelectionOffsets', () => {
	it('returns trimmed context around the selection', () => {
		const el = container('<p>Before words. The quote here. After words.</p>');
		const textNode = el.querySelector('p')!.firstChild as Text;
		const start = el.textContent!.indexOf('The quote');
		const range = document.createRange();
		range.setStart(textNode, start);
		range.setEnd(textNode, start + 'The quote here.'.length);
		const selection = window.getSelection()!;
		selection.removeAllRanges();
		selection.addRange(range);

		const offsets = getSelectionOffsets(el);

		expect(offsets).toMatchObject({
			text: 'The quote here.',
			startOffset: start,
			prefix: 'Before words.',
			suffix: 'After words.'
		});
	});
});
