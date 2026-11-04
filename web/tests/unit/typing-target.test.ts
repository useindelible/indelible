import { describe, expect, it } from 'vitest';

import { isActivationTarget, isTypingTarget } from '$lib/shortcuts/typing-target';

function element(html: string): HTMLElement {
	const host = document.createElement('div');
	host.innerHTML = html;
	return host.firstElementChild as HTMLElement;
}

describe('isTypingTarget', () => {
	it.each([
		['<input />'],
		['<textarea></textarea>'],
		['<select></select>'],
		['<div contenteditable="true"></div>'],
		['<div role="textbox"></div>'],
		['<div role="combobox"></div>'],
		['<div role="searchbox"></div>']
	])('claims %s', (html) => {
		expect(isTypingTarget(element(html))).toBe(true);
	});

	it.each([
		['<div></div>'],
		['<button></button>'],
		['<a href="/"></a>'],
		['<div role="option"></div>']
	])('leaves %s to the shortcuts', (html) => {
		expect(isTypingTarget(element(html))).toBe(false);
	});

	it('claims a descendant of a contenteditable region', () => {
		const editor = element('<div contenteditable="true"><span>word</span></div>');
		document.body.append(editor);

		expect(isTypingTarget(editor.querySelector('span'))).toBe(true);

		editor.remove();
	});

	it('ignores a missing or non-element target', () => {
		expect(isTypingTarget(null)).toBe(false);
		expect(isTypingTarget(new EventTarget())).toBe(false);
	});
});

describe('isActivationTarget', () => {
	it.each([
		['<button></button>'],
		['<a href="/x"></a>'],
		['<div role="option"></div>'],
		['<div role="tab"></div>'],
		['<button><span>inner</span></button>']
	])('claims %s', (html) => {
		const el = element(html);
		expect(isActivationTarget(el.querySelector('span') ?? el)).toBe(true);
	});

	it.each([['<div></div>'], ['<a></a>'], ['<span></span>'], ['<input />']])(
		'leaves %s alone',
		(html) => {
			expect(isActivationTarget(element(html))).toBe(false);
		}
	);
});
