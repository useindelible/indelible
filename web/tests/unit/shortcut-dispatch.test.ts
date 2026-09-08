import { describe, expect, it, vi } from 'vitest';

import { resolveShortcut, type ResolvableEvent, type ScopeLayer } from '$lib/shortcuts/dispatch';

function press(key: string, modifiers: Partial<ResolvableEvent> = {}): ResolvableEvent {
	return { key, metaKey: false, ctrlKey: false, altKey: false, shiftKey: false, ...modifiers };
}

const globalLayer: ScopeLayer = {
	scope: 'global',
	handlers: { add_url: vi.fn(), add_rss: vi.fn() }
};

const libraryLayer: ScopeLayer = {
	scope: 'library',
	handlers: { triage_archive: vi.fn(), select_next: vi.fn(), select_prev: vi.fn() }
};

function resolve(event: ResolvableEvent, layers: ScopeLayer[] = [globalLayer], typing = false) {
	return resolveShortcut({ event, layers, typing });
}

describe('resolveShortcut', () => {
	it('gives a to archive and never also to the add modal', () => {
		expect(resolve(press('a'), [globalLayer, libraryLayer])?.id).toBe('triage_archive');
	});

	it('leaves a unclaimed outside the library', () => {
		expect(resolve(press('a'))).toBeNull();
	});

	it('ignores cmd+a and ctrl+a so select-all still belongs to the browser', () => {
		const layers = [globalLayer, libraryLayer];
		expect(resolve(press('a', { metaKey: true }), layers)).toBeNull();
		expect(resolve(press('a', { ctrlKey: true }), layers)).toBeNull();
	});

	it('opens the add modal on n, bare or with mod', () => {
		expect(resolve(press('n'))?.id).toBe('add_url');
		expect(resolve(press('n', { metaKey: true }))?.id).toBe('add_url');
		expect(resolve(press('n', { ctrlKey: true }))?.id).toBe('add_url');
	});

	it('claims nothing while the user is typing, modifiers included', () => {
		const layers = [globalLayer, libraryLayer];
		expect(resolve(press('a'), layers, true)).toBeNull();
		expect(resolve(press('n'), layers, true)).toBeNull();
		expect(resolve(press('n', { metaKey: true }), layers, true)).toBeNull();
	});

	it('treats letter chords as case-insensitive and shift-agnostic', () => {
		const layers = [globalLayer, libraryLayer];
		expect(resolve(press('A'), layers)?.id).toBe('triage_archive');
		expect(resolve(press('J', { shiftKey: true }), layers)?.id).toBe('select_next');
	});

	it('binds the arrow aliases alongside j and k', () => {
		const layers = [globalLayer, libraryLayer];
		expect(resolve(press('ArrowDown'), layers)?.id).toBe('select_next');
		expect(resolve(press('ArrowUp'), layers)?.id).toBe('select_prev');
	});

	it('rejects alt-modified chords that no binding declares', () => {
		expect(resolve(press('n', { altKey: true }))).toBeNull();
	});

	it('lets an exclusive layer swallow everything beneath it', () => {
		const layers: ScopeLayer[] = [
			globalLayer,
			libraryLayer,
			{ scope: 'modal', handlers: {}, exclusive: true }
		];
		expect(resolve(press('a'), layers)).toBeNull();
		expect(resolve(press('n'), layers)).toBeNull();
	});

	it('stays silent during IME composition', () => {
		expect(resolve({ ...press('n'), isComposing: true })).toBeNull();
		expect(resolve({ ...press('n'), keyCode: 229 })).toBeNull();
	});

	it('returns the handler belonging to the winning layer', () => {
		const handler = vi.fn();
		const resolved = resolve(press('a'), [
			globalLayer,
			{ scope: 'library', handlers: { triage_archive: handler } }
		]);

		resolved?.handler(new KeyboardEvent('keydown'));
		expect(handler).toHaveBeenCalledOnce();
	});
});
