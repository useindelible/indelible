import { describe, it, expect, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import ItemRow from '$lib/components/library/ItemRow.svelte';
import type { DocumentListEntry } from '$lib/api';

vi.mock('$lib/api', () => ({
	getLibraryEntryTags: vi.fn().mockResolvedValue({ data: { tags: [] } }),
	replaceLibraryEntryTags: vi.fn().mockResolvedValue({ data: { tags: [] } })
}));

const REASON = 'renderer rejected a private or internal address';

function item(overrides: Partial<DocumentListEntry> = {}): DocumentListEntry {
	return {
		id: 'doc_x',
		document_id: 'doc_x',
		document_type: 'article',
		library_entry_id: 'lib_x',
		object: 'library_entry',
		title: 'A Saved URL',
		saved_at: '2026-05-18T10:00:00Z',
		created_at: '2026-05-18T10:00:00Z',
		updated_at: '2026-05-18T10:00:00Z',
		source: 'web',
		item_type: 'article',
		triage_state: 'inbox',
		is_favorite: false,
		is_shortlisted: false,
		...overrides
	} as DocumentListEntry;
}

function renderRow(entry: DocumentListEntry, onDelete?: () => void) {
	return render(ItemRow, {
		props: {
			item: entry,
			selected: false,
			onSelect: () => {},
			onOpen: () => {},
			onTriage: () => {},
			onDelete
		}
	});
}

describe('failed-ingestion row state', () => {
	it('renders a Failed badge and the renderer reason', () => {
		renderRow(item({ pipeline_status: 'failed', pipeline_error: REASON }));

		expect(screen.getByText('Failed')).toBeTruthy();
		expect(screen.getByText(REASON)).toBeTruthy();
	});

	it('renders no failure chrome for a healthy row', () => {
		renderRow(item());

		expect(screen.queryByText('Failed')).toBeNull();
		expect(screen.queryByText(REASON)).toBeNull();
	});

	it('still offers Delete so a failed row can be removed', async () => {
		const onDelete = vi.fn();
		renderRow(item({ pipeline_status: 'failed', pipeline_error: REASON }), onDelete);

		const row = screen.getByText('A Saved URL').closest('.item-row') as HTMLElement;
		await fireEvent.contextMenu(row);
		await fireEvent.click(screen.getByText('Delete'));

		expect(onDelete).toHaveBeenCalled();
	});
});
