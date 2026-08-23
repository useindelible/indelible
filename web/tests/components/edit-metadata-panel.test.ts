import { render, screen } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { DocumentListEntry } from '$lib/api';
import { locale, setupI18nSync } from '$lib/i18n';
import fr from '$lib/i18n/locales/fr.json';

const apiMocks = vi.hoisted(() => ({
	getLibraryEntryTags: vi.fn(),
	replaceLibraryEntryTags: vi.fn(),
	listTags: vi.fn(),
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
	afterEach(() => locale.set('en'));

	beforeEach(() => {
		vi.clearAllMocks();
		apiMocks.getLibraryEntryTags.mockImplementation(({ path }) =>
			Promise.resolve({
				data: { tags: path.library_entry_id === 'lib_1' ? ['neural networks'] : [] }
			})
		);
		apiMocks.listTags.mockResolvedValue({
			data: { data: [], page: { next_cursor: null } }
		});
	});

	it('loads existing tags through the library entry identity', async () => {
		render(EditMetadataPanel, { props: { item: item('article'), onClose: vi.fn() } });

		expect(await screen.findByRole('button', { name: 'Remove tag neural networks' })).toBeTruthy();
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

	it('formats word counts in the active locale', () => {
		setupI18nSync({ fr }, 'fr');
		const entry = item('article');
		entry.word_count = 1234;

		render(EditMetadataPanel, { props: { item: entry, onClose: vi.fn() } });

		expect(screen.getByText(/1.?234 mots/)).toBeTruthy();
	});
});
