import { describe, it, expect, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import ItemRow from '$lib/components/library/ItemRow.svelte';
import type { DocumentListEntry } from '$lib/api';

vi.mock('$lib/api', () => ({
	getLibraryEntryTags: vi.fn().mockResolvedValue({ data: { tags: [] } }),
	replaceLibraryEntryTags: vi.fn().mockResolvedValue({ data: { tags: [] } })
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

function renderRow(handlers: { onTriage?: () => void; onDelete?: () => void }) {
	return render(ItemRow, {
		props: {
			item: item(),
			selected: false,
			onSelect: () => {},
			onOpen: () => {},
			onTriage: handlers.onTriage ?? (() => {}),
			onDelete: handlers.onDelete
		}
	});
}

describe('context-menu delete gating', () => {
	it('invokes onDelete and never routes through triage', async () => {
		const onDelete = vi.fn();
		const onTriage = vi.fn();
		renderRow({ onDelete, onTriage });

		await fireEvent.contextMenu(screen.getByRole('option'));
		await fireEvent.click(screen.getByText('Delete'));

		expect(onDelete).toHaveBeenCalledTimes(1);
		expect(onTriage).not.toHaveBeenCalled();
	});

	it('hides Delete when no handler is provided', async () => {
		renderRow({});

		await fireEvent.contextMenu(screen.getByRole('option'));

		expect(screen.getByText('Add Tags')).toBeTruthy();
		expect(screen.queryByText('Delete')).toBeNull();
	});
});
