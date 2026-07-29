import { describe, it, expect, beforeEach } from 'vitest';
import {
	getSelectionOffsets,
	applyHighlights,
	getTotalTextLength,
	type HighlightRange
} from '$lib/components/reader/highlight-utils';
import {
	filterTagSuggestions,
	normalizeTagName
} from '$lib/components/reader/highlight-toolbar-model';

function createContainer(html: string): HTMLDivElement {
	const el = document.createElement('div');
	el.innerHTML = html;
	document.body.appendChild(el);
	return el;
}

function selectRange(
	container: HTMLElement,
	startNode: Node,
	startOffset: number,
	endNode: Node,
	endOffset: number
): void {
	const range = document.createRange();
	range.setStart(startNode, startOffset);
	range.setEnd(endNode, endOffset);
	const selection = window.getSelection();
	selection?.removeAllRanges();
	selection?.addRange(range);
}

describe('highlight-utils', () => {
	beforeEach(() => {
		document.body.innerHTML = '';
		window.getSelection()?.removeAllRanges();
	});

	describe('getSelectionOffsets', () => {
		it('returns null when no selection exists', () => {
			const container = createContainer('<p>Hello world</p>');
			const result = getSelectionOffsets(container);
			expect(result).toBeNull();
		});

		it('returns null when selection is outside container', () => {
			const container = createContainer('<p>Inside</p>');
			const outside = createContainer('<p>Outside</p>');
			const textNode = outside.querySelector('p')!.firstChild!;
			selectRange(outside, textNode, 0, textNode, 7);
			const result = getSelectionOffsets(container);
			expect(result).toBeNull();
		});

		it('calculates offsets for a simple text selection', () => {
			const container = createContainer('<p>Hello world</p>');
			const textNode = container.querySelector('p')!.firstChild!;
			selectRange(container, textNode, 0, textNode, 5);

			const result = getSelectionOffsets(container);
			expect(result).not.toBeNull();
			expect(result!.text).toBe('Hello');
			expect(result!.startOffset).toBe(0);
			expect(result!.endOffset).toBe(5);
		});

		it('calculates offsets spanning multiple paragraphs', () => {
			const container = createContainer('<p>First paragraph</p><p>Second paragraph</p>');
			const firstText = container.querySelectorAll('p')[0]!.firstChild!;
			const secondText = container.querySelectorAll('p')[1]!.firstChild!;
			selectRange(container, firstText, 6, secondText, 6);

			const result = getSelectionOffsets(container);
			expect(result).not.toBeNull();
			expect(result!.startOffset).toBe(6);
			// "First paragraph" = 15 chars, "Second" starts at offset 15 + 6 = 21
			expect(result!.endOffset).toBe(21);
		});

		it('returns null for collapsed (empty) selection', () => {
			const container = createContainer('<p>Hello</p>');
			const textNode = container.querySelector('p')!.firstChild!;
			selectRange(container, textNode, 3, textNode, 3);

			const result = getSelectionOffsets(container);
			expect(result).toBeNull();
		});
	});

	describe('applyHighlights', () => {
		it('wraps text in mark elements', () => {
			const container = createContainer('<p>Hello world test</p>');
			const highlights: HighlightRange[] = [
				{ id: 'hl1', color: 'yellow', startOffset: 6, endOffset: 11 }
			];

			applyHighlights(container, highlights);

			const marks = container.querySelectorAll('mark');
			expect(marks.length).toBe(1);
			expect(marks[0]!.textContent).toBe('world');
			expect(marks[0]!.className).toBe('highlight-yellow');
			expect(marks[0]!.dataset.highlightId).toBe('hl1');
		});

		it('handles multiple non-overlapping highlights', () => {
			const container = createContainer('<p>Hello world test case</p>');
			const highlights: HighlightRange[] = [
				{ id: 'hl1', color: 'yellow', startOffset: 0, endOffset: 5 },
				{ id: 'hl2', color: 'blue', startOffset: 12, endOffset: 16 }
			];

			applyHighlights(container, highlights);

			const marks = container.querySelectorAll('mark');
			expect(marks.length).toBe(2);
			expect(marks[0]!.textContent).toBe('Hello');
			expect(marks[0]!.className).toBe('highlight-yellow');
			expect(marks[1]!.textContent).toBe('test');
			expect(marks[1]!.className).toBe('highlight-blue');
		});

		it('clears existing highlights before applying new ones', () => {
			const container = createContainer('<p>Hello world</p>');
			applyHighlights(container, [{ id: 'hl1', color: 'yellow', startOffset: 0, endOffset: 5 }]);
			expect(container.querySelectorAll('mark').length).toBe(1);

			applyHighlights(container, [{ id: 'hl2', color: 'blue', startOffset: 6, endOffset: 11 }]);
			const marks = container.querySelectorAll('mark');
			expect(marks.length).toBe(1);
			expect(marks[0]!.textContent).toBe('world');
			expect(marks[0]!.className).toBe('highlight-blue');
		});

		it('handles empty highlights array', () => {
			const container = createContainer('<p>Hello world</p>');
			applyHighlights(container, [{ id: 'hl1', color: 'yellow', startOffset: 0, endOffset: 5 }]);
			expect(container.querySelectorAll('mark').length).toBe(1);

			applyHighlights(container, []);
			expect(container.querySelectorAll('mark').length).toBe(0);
			expect(container.textContent).toBe('Hello world');
		});

		it('handles highlight spanning multiple elements', () => {
			const container = createContainer('<p>Hello</p><p>world</p>');
			const highlights: HighlightRange[] = [
				{ id: 'hl1', color: 'green', startOffset: 3, endOffset: 8 }
			];

			applyHighlights(container, highlights);

			const marks = container.querySelectorAll('mark');
			expect(marks.length).toBe(2);
			expect(marks[0]!.textContent).toBe('lo');
			expect(marks[1]!.textContent).toBe('wor');
		});
	});

	describe('getTotalTextLength', () => {
		it('returns total text length of container', () => {
			const container = createContainer('<p>Hello</p><p>World</p>');
			expect(getTotalTextLength(container)).toBe(10);
		});

		it('returns 0 for empty container', () => {
			const container = createContainer('');
			expect(getTotalTextLength(container)).toBe(0);
		});

		it('includes text from nested elements', () => {
			const container = createContainer('<p>Hello <strong>bold</strong> text</p>');
			expect(getTotalTextLength(container)).toBe(15);
		});

		it('ignores injected body tag labels when measuring reader text', () => {
			const container = createContainer(
				'<p>Hello <span class="hl-body-tag">Research</span>world</p>'
			);
			expect(getTotalTextLength(container)).toBe(11);
		});
	});

	describe('highlight toolbar tag helpers', () => {
		it('normalizes tags before matching or comparing them', () => {
			expect(normalizeTagName('  Research Notes  ')).toBe('research notes');
		});

		it('filters suggestions by query and excludes applied tags case-insensitively', () => {
			const suggestions = filterTagSuggestions(
				[
					{
						id: 'tag_1',
						object: 'tag',
						name: 'Research',
						color: '#facc15',
						aliases: [],
						item_count: 0,
						highlight_count: 2,
						created_at: '2026-01-01T00:00:00Z'
					},
					{
						id: 'tag_2',
						object: 'tag',
						name: 'Reading Queue',
						color: null,
						aliases: [],
						item_count: 0,
						highlight_count: 0,
						created_at: '2026-01-01T00:00:00Z'
					},
					{
						id: 'tag_3',
						object: 'tag',
						name: 'Mila',
						color: null,
						aliases: [],
						item_count: 0,
						highlight_count: 0,
						created_at: '2026-01-01T00:00:00Z'
					}
				],
				['research'],
				'read'
			);

			expect(suggestions.map((tag) => tag.name)).toEqual(['Reading Queue']);
		});
	});
});
