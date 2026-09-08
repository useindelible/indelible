import { describe, expect, it } from 'vitest';

import {
	getLibrarySelection,
	neighbourId,
	nextSelectedId
} from '$lib/stores/library-selection.svelte';

const items = [{ id: 'a' }, { id: 'b' }, { id: 'c' }];

describe('nextSelectedId', () => {
	it.each([
		[null, 1, 'a'],
		['a', 1, 'b'],
		['c', 1, 'c'],
		['a', -1, 'a'],
		['b', -1, 'a'],
		[null, -1, 'a']
	])('from %s by %i lands on %s', (selectedId, offset, expected) => {
		expect(nextSelectedId(items, selectedId, offset)).toBe(expected);
	});

	it('yields nothing for an empty list', () => {
		expect(nextSelectedId([], null, 1)).toBeNull();
	});
});

describe('neighbourId', () => {
	it.each([
		['a', 'b'],
		['b', 'c'],
		['c', 'b']
	])('after %s leaves, %s takes over', (leaving, expected) => {
		expect(neighbourId(items, leaving)).toBe(expected);
	});

	it('yields nothing when the row is alone or absent', () => {
		expect(neighbourId([{ id: 'a' }], 'a')).toBeNull();
		expect(neighbourId(items, 'zzz')).toBeNull();
	});
});

describe('hoverSelects', () => {
	const selection = getLibrarySelection();
	const still = { clientX: 40, clientY: 80 };
	const moved = { clientX: 41, clientY: 80 };

	it('lets a row slide under a parked pointer without stealing the keyboard selection', () => {
		selection.moveSelection(1);
		expect(selection.keyboardDriving).toBe(true);

		expect(selection.hoverSelects(still)).toBe(false);
		expect(selection.hoverSelects(still)).toBe(false);
		expect(selection.keyboardDriving).toBe(true);
	});

	it('hands selection back to the pointer on a real move', () => {
		expect(selection.hoverSelects(moved)).toBe(true);
		expect(selection.keyboardDriving).toBe(false);
		expect(selection.hoverSelects(moved)).toBe(true);
	});
});
