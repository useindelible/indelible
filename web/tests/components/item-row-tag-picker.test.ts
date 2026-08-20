import { describe, it, expect, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import ItemRow from '$lib/components/library/ItemRow.svelte';
import type { DocumentListEntry } from '$lib/api';

const apiMocks = vi.hoisted(() => ({
	getLibraryEntryTags: vi.fn().mockImplementation(({ path }) =>
		Promise.resolve({
			data: { tags: path.library_entry_id === 'lib_lib' ? ['neural networks'] : [] }
		})
	),
	replaceLibraryEntryTags: vi.fn().mockResolvedValue({ data: { tags: [] } }),
	listTags: vi.fn().mockResolvedValue({ data: { data: [], page: { next_cursor: null } } })
}));

vi.mock('$lib/api', () => apiMocks);

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
	it('loads existing tags through the library entry identity', async () => {
		renderRow(item());

		await fireEvent.contextMenu(screen.getByRole('option'));
		await fireEvent.click(screen.getByText('Add Tags'));

		await waitFor(() => {
			expect(screen.getByRole('button', { name: 'Remove tag neural networks' })).toBeTruthy();
		});
	});

	it('offers Add Tags for a library-backed row', async () => {
		renderRow(item());

		await fireEvent.contextMenu(screen.getByRole('option'));

		expect(screen.getByText('Add Tags')).toBeTruthy();
	});

	it('hides Add Tags for a feed delivery row', async () => {
		renderRow(
			item({ id: 'dlv_1', library_entry_id: null, object: 'feed_delivery', source: 'feed' })
		);

		await fireEvent.contextMenu(screen.getByRole('option'));

		expect(screen.queryByText('Add Tags')).toBeNull();
	});
});
