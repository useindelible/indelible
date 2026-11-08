import { describe, expect, it } from 'vitest';
import { textPositionAt } from './restore-position';

function body(html: string): HTMLElement {
	const el = document.createElement('div');
	el.innerHTML = html;
	return el;
}

describe('textPositionAt', () => {
	it('finds the character inside a text node', () => {
		const position = textPositionAt(body('<p>Hello world</p>'), 6);

		expect(position?.node.textContent).toBe('Hello world');
		expect(position?.offset).toBe(6);
	});

	it('moves an offset in whitespace between blocks to the next text', () => {
		const el = body('\n<h1>Chapter 2</h1>\n<p>Introduction</p>');

		expect(textPositionAt(el, 1)?.node.textContent).toBe('Chapter 2');
		expect(textPositionAt(el, 11)?.node.textContent).toBe('Introduction');
		expect(textPositionAt(el, 11)?.offset).toBe(0);
	});

	it('returns null past the end of the text', () => {
		expect(textPositionAt(body('<p>Short</p>\n'), 50)).toBeNull();
	});
});
