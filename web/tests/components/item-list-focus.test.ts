import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { flushSync } from 'svelte';

import ItemList from '$lib/components/library/ItemList.svelte';
import type { DocumentListEntry } from '$lib/api';

vi.mock('$lib/api', () => ({
	getLibraryEntryTags: vi.fn().mockResolvedValue({ data: { tags: [] } }),
	replaceLibraryEntryTags: vi.fn().mockResolvedValue({ data: { tags: [] } })
}));

class MockIntersectionObserver {
	observe = vi.fn();
	disconnect = vi.fn();
}
vi.stubGlobal('IntersectionObserver', MockIntersectionObserver);

function entry(id: string): DocumentListEntry {
	return {
		id,
		document_id: id,
		document_type: 'article',
		library_entry_id: `lib_${id}`,
		object: 'library_entry',
		title: `Title ${id}`,
		saved_at: '2026-05-18T10:00:00Z',
		created_at: '2026-05-18T10:00:00Z',
		updated_at: '2026-05-18T10:00:00Z',
		source: 'web',
		item_type: 'article',
		triage_state: 'inbox',
		is_favorite: false,
		is_shortlisted: false
	} as DocumentListEntry;
}

function mount(selectedId: string) {
	return render(ItemList, {
		props: {
			items: [entry('a'), entry('b')],
			loading: false,
			loadingMore: false,
			hasMore: false,
			isEmpty: false,
			selectedId,
			triageTab: 'inbox',
			onLoadMore: () => {},
			onSelect: () => {},
			onOpen: () => {},
			onTriage: () => {}
		}
	});
}

describe('ItemList focus', () => {
	it('moves focus to the newly selected row when a row already had it', async () => {
		const { rerender } = mount('a');
		const [rowA, rowB] = screen.getAllByRole('option');
		rowA!.focus();

		await rerender({ selectedId: 'b' });
		flushSync();

		expect(document.activeElement).toBe(rowB);
	});

	it('leaves focus alone when it is not on a row', async () => {
		const { rerender } = mount('a');
		(document.activeElement as HTMLElement | null)?.blur();

		await rerender({ selectedId: 'b' });
		flushSync();

		expect(document.activeElement).toBe(document.body);
	});
});
