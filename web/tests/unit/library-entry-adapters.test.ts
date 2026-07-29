import { beforeEach, describe, expect, it, vi } from 'vitest';
import type {
	LibraryEntryResponse,
	PaginatedResponseLibraryEntryResponse
} from '$lib/api/generated/types.gen';

const mockListCollectionEntries = vi.fn();
const mockListTagEntries = vi.fn();

vi.mock('$lib/api/generated', async () => {
	const actual = await vi.importActual<typeof import('$lib/api/generated')>('$lib/api/generated');
	return {
		...actual,
		listCollectionEntries: (...args: unknown[]) => mockListCollectionEntries(...args),
		listTagEntries: (...args: unknown[]) => mockListTagEntries(...args)
	};
});

import { listCollectionEntries, listTagEntries } from '$lib/api';

function entry(overrides: Partial<LibraryEntryResponse> = {}): LibraryEntryResponse {
	return {
		created_at: '2026-05-01T00:00:00Z',
		document_id: 'doc_abc',
		document_type: 'article',
		is_favorite: false,
		is_shortlisted: false,
		library_entry_id: 'lib_abc',
		object: 'library_entry',
		saved_at: '2026-05-01T00:00:00Z',
		source: 'web',
		title: 'A Saved Article',
		triage_state: 'later',
		updated_at: '2026-05-01T00:00:00Z',
		...overrides
	};
}

function page(entries: LibraryEntryResponse[]): PaginatedResponseLibraryEntryResponse {
	return { data: entries, page: { has_more: false, next_cursor: null } };
}

describe('library entry collection/tag adapters', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('maps collection entries to document list entries with id and item_type', async () => {
		mockListCollectionEntries.mockResolvedValue({ data: page([entry()]) });

		const { data } = await listCollectionEntries({ path: { id: 'col_1' } });

		expect(mockListCollectionEntries).toHaveBeenCalledWith({ path: { id: 'col_1' } });
		const mapped = data?.data[0];
		expect(mapped?.id).toBe('doc_abc');
		expect(mapped?.item_type).toBe('article');
	});

	it('maps tag entries to document list entries with id and item_type', async () => {
		mockListTagEntries.mockResolvedValue({
			data: page([entry({ document_id: 'doc_xyz', document_type: 'pdf' })])
		});

		const { data } = await listTagEntries({ path: { id: 'tag_1' } });

		const mapped = data?.data[0];
		expect(mapped?.id).toBe('doc_xyz');
		expect(mapped?.item_type).toBe('pdf');
	});

	it('returns undefined data when the generated call yields none', async () => {
		mockListCollectionEntries.mockResolvedValue({ data: undefined });

		const { data } = await listCollectionEntries({ path: { id: 'col_1' } });

		expect(data).toBeUndefined();
	});
});
