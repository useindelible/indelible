import { describe, it, expect, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import ItemRow from '$lib/components/library/ItemRow.svelte';
import type { DocumentListEntry } from '$lib/api';

vi.mock('$lib/api', () => ({
	getDocumentEntryTags: vi.fn().mockResolvedValue({ data: { tags: [] } }),
	replaceDocumentEntryTags: vi.fn().mockResolvedValue({ data: { tags: [] } })
}));

function item(overrides: Partial<DocumentListEntry> = {}): DocumentListEntry {
	return {
		id: 'doc_lib',
		document_id: 'doc_lib',
		document_type: 'article',
		library_entry_id: 'lib_lib',
		object: 'library_entry',
		title: 'A Library Article',
		saved_at: '2026-05-18T10:00:00Z',
		created_at: '2026-05-18T10:00:00Z',
		updated_at: '2026-05-18T10:00:00Z',
		source: 'web',
		item_type: 'article',
		triage_state: 'later',
		is_favorite: false,
		is_shortlisted: false,
		...overrides
	} as DocumentListEntry;
}

function renderRow(entry: DocumentListEntry) {
	return render(ItemRow, {
		props: {
			item: entry,
			selected: false,
			onSelect: () => {},
			onOpen: () => {},
			onTriage: () => {},
			onDelete: () => {}
		}
	});
}

describe('ItemRow tag picker gating', () => {
	it('offers Add Tags for a library-backed row', async () => {
		renderRow(item());

		await fireEvent.contextMenu(screen.getByRole('option'));

		expect(screen.getByText('Add Tags')).toBeTruthy();
	});

	it('hides Add Tags for a feed delivery row', async () => {
		renderRow(item({ id: 'dlv_1', object: 'feed_delivery', source: 'feed' }));

		await fireEvent.contextMenu(screen.getByRole('option'));

		expect(screen.queryByText('Add Tags')).toBeNull();
	});
});
