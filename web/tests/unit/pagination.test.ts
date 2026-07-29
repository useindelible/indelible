import { describe, it, expect } from 'vitest';
import { fetchAllPages, buildCollectionTree, buildBreadcrumbPath } from '$lib/api/pagination';
import type { CollectionResponse } from '$lib/api/generated/types.gen';

describe('fetchAllPages', () => {
	it('fetches single page when no next_cursor', async () => {
		const fetcher = async () => ({
			data: [{ id: '1' }, { id: '2' }],
			page: { next_cursor: null }
		});

		const result = await fetchAllPages(fetcher);
		expect(result).toHaveLength(2);
		expect(result[0]).toEqual({ id: '1' });
	});

	it('exhausts multiple pages', async () => {
		let calls = 0;
		const fetcher = async (cursor: string | null) => {
			calls++;
			if (cursor === null) {
				return {
					data: [{ id: '1' }],
					page: { next_cursor: 'page2' }
				};
			}
			return {
				data: [{ id: '2' }],
				page: { next_cursor: null }
			};
		};

		const result = await fetchAllPages(fetcher);
		expect(result).toHaveLength(2);
		expect(calls).toBe(2);
	});

	it('returns empty array when fetcher returns undefined', async () => {
		const fetcher = async () => undefined;
		const result = await fetchAllPages(fetcher);
		expect(result).toHaveLength(0);
	});

	it('stops at safety cap of 2000 items', async () => {
		let calls = 0;
		const fetcher = async () => {
			calls++;
			const items = Array.from({ length: 200 }, (_, i) => ({ id: `${calls}-${i}` }));
			return {
				data: items,
				page: { next_cursor: `page${calls + 1}` }
			};
		};

		const result = await fetchAllPages(fetcher);
		expect(result.length).toBe(2000);
		expect(calls).toBe(10);
	});
});

function makeCollection(
	id: string,
	name: string,
	parentId: string | null = null
): CollectionResponse {
	return {
		id,
		object: 'collection',
		name,
		description: null,
		icon: null,
		color: null,
		parent_id: parentId,
		item_count: 0,
		child_count: 0,
		sort_order: 0,
		created_at: '2024-01-01T00:00:00Z',
		updated_at: '2024-01-01T00:00:00Z'
	} as CollectionResponse;
}

describe('buildCollectionTree', () => {
	it('builds flat list into root nodes', () => {
		const collections = [makeCollection('a', 'Alpha'), makeCollection('b', 'Beta')];

		const tree = buildCollectionTree(collections, {});
		expect(tree).toHaveLength(2);
		expect(tree[0]!.collection.name).toBe('Alpha');
		expect(tree[1]!.collection.name).toBe('Beta');
	});

	it('nests children under parents', () => {
		const collections = [
			makeCollection('root', 'Root'),
			makeCollection('child1', 'Child 1', 'root'),
			makeCollection('child2', 'Child 2', 'root')
		];

		const tree = buildCollectionTree(collections, {});
		expect(tree).toHaveLength(1);
		expect(tree[0]!.children).toHaveLength(2);
		expect(tree[0]!.children[0]!.collection.name).toBe('Child 1');
	});

	it('respects expanded state', () => {
		const collections = [makeCollection('root', 'Root'), makeCollection('child', 'Child', 'root')];

		const tree = buildCollectionTree(collections, { root: true });
		expect(tree[0]!.expanded).toBe(true);
		expect(tree[0]!.children[0]!.expanded).toBe(false);
	});

	it('handles deeply nested trees', () => {
		const collections = [
			makeCollection('a', 'A'),
			makeCollection('b', 'B', 'a'),
			makeCollection('c', 'C', 'b')
		];

		const tree = buildCollectionTree(collections, {});
		expect(tree).toHaveLength(1);
		expect(tree[0]!.children[0]!.children[0]!.collection.name).toBe('C');
	});
});

describe('buildBreadcrumbPath', () => {
	it('returns path from root to target', () => {
		const collections = [
			makeCollection('root', 'Root'),
			makeCollection('mid', 'Mid', 'root'),
			makeCollection('leaf', 'Leaf', 'mid')
		];

		const path = buildBreadcrumbPath('leaf', collections);
		expect(path).toHaveLength(3);
		expect(path[0]!.name).toBe('Root');
		expect(path[1]!.name).toBe('Mid');
		expect(path[2]!.name).toBe('Leaf');
	});

	it('returns single item for root collection', () => {
		const collections = [makeCollection('root', 'Root')];

		const path = buildBreadcrumbPath('root', collections);
		expect(path).toHaveLength(1);
		expect(path[0]!.name).toBe('Root');
	});

	it('returns empty for unknown id', () => {
		const path = buildBreadcrumbPath('unknown', []);
		expect(path).toHaveLength(0);
	});

	it('stops when it encounters a cycle', () => {
		const collections = [makeCollection('a', 'A', 'b'), makeCollection('b', 'B', 'a')];

		const path = buildBreadcrumbPath('a', collections);
		expect(path.map((collection) => collection.id)).toEqual(['b', 'a']);
	});
});
