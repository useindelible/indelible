import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import type { DocumentListEntry } from '$lib/api';
import BookDetailPanel from '$lib/components/reader/book/BookDetailPanel.svelte';

vi.mock('$lib/components/library/ChatTab.svelte', () => ({
	default: vi.fn(() => ({ c: vi.fn(), m: vi.fn(), p: vi.fn(), d: vi.fn() }))
}));

function item(): DocumentListEntry {
	return {
		id: 'doc_pdf',
		document_id: 'doc_pdf',
		title: 'Scanned field notes',
		document_type: 'pdf',
		item_type: 'pdf',
		object: 'library_entry',
		source: 'manual',
		created_at: '2026-08-12T00:00:00Z',
		updated_at: '2026-08-12T00:00:00Z',
		saved_at: '2026-08-12T00:00:00Z',
		triage_state: 'inbox',
		is_favorite: false,
		is_shortlisted: false
	} as DocumentListEntry;
}

const bookMetadata = {
	title: 'Scanned field notes',
	author: 'Indelible',
	totalChapters: 4
};

describe('BookDetailPanel text availability', () => {
	it('keeps Info and Notebook but omits Chat when extracted text is unavailable', () => {
		render(BookDetailPanel, {
			props: { item: item(), bookMetadata, progress: 10, textAvailable: false }
		});

		expect(screen.getByRole('tab', { name: 'Info' })).toBeTruthy();
		expect(screen.getByRole('tab', { name: 'Notebook' })).toBeTruthy();
		expect(screen.queryByRole('tab', { name: 'Chat' })).toBeNull();
	});

	it('resets Chat to Info if text becomes unavailable while Chat is active', async () => {
		const rendered = render(BookDetailPanel, {
			props: { item: item(), bookMetadata, progress: 10, textAvailable: true }
		});

		await fireEvent.click(screen.getByRole('tab', { name: 'Chat' }));
		await rendered.rerender({
			item: item(),
			bookMetadata,
			progress: 10,
			textAvailable: false
		});

		await waitFor(() =>
			expect(screen.getByRole('tab', { name: 'Info' }).getAttribute('aria-selected')).toBe('true')
		);
		expect(screen.queryByRole('tab', { name: 'Chat' })).toBeNull();
	});
});
