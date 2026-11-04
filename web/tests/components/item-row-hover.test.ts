import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';

import ItemRow from '$lib/components/library/ItemRow.svelte';
import type { DocumentListEntry } from '$lib/api';

vi.mock('$lib/api', () => ({
	getLibraryEntryTags: vi.fn().mockResolvedValue({ data: { tags: [] } }),
	replaceLibraryEntryTags: vi.fn().mockResolvedValue({ data: { tags: [] } })
}));

const item = {
	id: 'doc_1',
	document_id: 'doc_1',
	document_type: 'article',
	library_entry_id: 'lib_1',
	object: 'library_entry',
	title: 'Hovered',
	saved_at: '2026-05-18T10:00:00Z',
	created_at: '2026-05-18T10:00:00Z',
	updated_at: '2026-05-18T10:00:00Z',
	source: 'web',
	item_type: 'article',
	triage_state: 'inbox',
	is_favorite: false,
	is_shortlisted: false
} as DocumentListEntry;

describe('ItemRow pointer reporting', () => {
	it('reports entry and movement with the pointer coordinates', async () => {
		const onSelect = vi.fn();
		render(ItemRow, {
			props: { item, selected: false, onSelect, onOpen: () => {}, onTriage: () => {} }
		});
		const row = screen.getByRole('option');

		await fireEvent.mouseEnter(row, { clientX: 10, clientY: 20 });
		await fireEvent.mouseMove(row, { clientX: 12, clientY: 20 });

		expect(onSelect).toHaveBeenCalledTimes(2);
		expect(onSelect.mock.calls.map(([e]) => (e as MouseEvent).clientX)).toEqual([10, 12]);
	});
});
