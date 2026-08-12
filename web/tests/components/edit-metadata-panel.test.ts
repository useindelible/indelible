import { render, screen } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { DocumentListEntry } from '$lib/api';

const apiMocks = vi.hoisted(() => ({
	getDocumentEntryTags: vi.fn(),
	replaceDocumentEntryTags: vi.fn(),
	updateDocumentEntry: vi.fn()
}));

vi.mock('$lib/api', () => apiMocks);
vi.mock('$lib/stores/library.svelte', () => ({
	getLibrary: () => ({ updateItemInList: vi.fn() })
}));

import EditMetadataPanel from '$lib/components/library/EditMetadataPanel.svelte';

function item(itemType: string): DocumentListEntry {
	return {
		author: null,
		canonical_url: 'https://example.com/entry',
		created_at: '2026-08-12T00:00:00Z',
		document_id: 'doc_1',
		document_type: itemType,
		excerpt: null,
		id: 'doc_1',
		is_favorite: false,
		is_shortlisted: false,
		item_type: itemType,
		library_entry_id: 'lib_1',
		object: 'library_entry',
		published_at: null,
		saved_at: '2026-08-12T00:00:00Z',
		source: 'library',
		title: 'Example entry',
		triage_state: 'inbox',
		updated_at: '2026-08-12T00:00:00Z',
		url: 'https://example.com/entry'
	};
}

describe('EditMetadataPanel content types', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		apiMocks.getDocumentEntryTags.mockResolvedValue({ data: { tags: [] } });
	});

	it('does not offer Podcast when editing an ordinary item', () => {
		render(EditMetadataPanel, { props: { item: item('article'), onClose: vi.fn() } });

		expect(screen.queryByRole('option', { name: 'Podcast' })).toBeNull();
	});

	it('keeps a selected legacy Podcast option when editing a podcast item', () => {
		render(EditMetadataPanel, { props: { item: item('podcast'), onClose: vi.fn() } });

		const option = screen.getByRole('option', { name: 'Podcast' }) as HTMLOptionElement;
		expect(option.selected).toBe(true);
	});
});
