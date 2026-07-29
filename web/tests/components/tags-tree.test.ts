import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import type { TagResponse } from '$lib/api/generated/types.gen';
import TagsTree from '../../src/routes/(app)/tags/components/TagsTree.svelte';
import { buildTagTree, rolledUpItemCounts } from '../../src/routes/(app)/tags/tag-tree';

function tag(overrides: Partial<TagResponse> = {}): TagResponse {
	return {
		aliases: [],
		color: null,
		created_at: '2026-06-10T00:00:00Z',
		highlight_count: 0,
		id: 'tag_1',
		item_count: 0,
		name: 'Tag',
		object: 'tag',
		parent_id: null,
		...overrides
	};
}

describe('TagsTree', () => {
	it('renders rows and forwards row actions', async () => {
		const tags = [
			tag({ id: 'a', name: 'Articles', item_count: 2 }),
			tag({ id: 'b', parent_id: 'a', name: 'Research', item_count: 1 })
		];
		const onOpen = vi.fn();
		const onToggleExpand = vi.fn();
		const onToggleSelect = vi.fn();

		render(TagsTree, {
			props: {
				activeScope: 'all',
				expandedParents: new Set(['a']),
				loading: false,
				isEmpty: false,
				nodes: buildTagTree(tags, new Set(['a'])),
				rolledUpCounts: rolledUpItemCounts(tags),
				selectedIds: new Set(),
				totalCount: 2,
				fetchError: null,
				onContextMenu: vi.fn(),
				onCreate: vi.fn(),
				onOpen,
				onToggleExpand,
				onToggleSelect
			}
		});

		expect(screen.getByText('Articles')).toBeTruthy();
		expect(screen.getByText('Research')).toBeTruthy();
		expect(screen.getByText('3 items')).toBeTruthy();

		await fireEvent.click(screen.getByRole('button', { name: /expand|collapse/i }));
		expect(onToggleExpand).toHaveBeenCalledWith('a');

		await fireEvent.click(screen.getByRole('button', { name: /select articles/i }));
		expect(onToggleSelect).toHaveBeenCalledWith('a');

		await fireEvent.click(screen.getByText('Research'));
		expect(onOpen).toHaveBeenCalledWith('b');
	});

	it('renders the empty state create action', async () => {
		const onCreate = vi.fn();

		render(TagsTree, {
			props: {
				activeScope: 'all',
				expandedParents: new Set(),
				loading: false,
				isEmpty: true,
				nodes: [],
				rolledUpCounts: new Map(),
				selectedIds: new Set(),
				totalCount: 0,
				fetchError: null,
				onContextMenu: vi.fn(),
				onCreate,
				onOpen: vi.fn(),
				onToggleExpand: vi.fn(),
				onToggleSelect: vi.fn()
			}
		});

		await fireEvent.click(screen.getByRole('button', { name: /create your first tag/i }));
		expect(onCreate).toHaveBeenCalledOnce();
	});
});
