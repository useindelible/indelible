const MAX_ITEMS = 2000;

interface PaginatedResult<T> {
	data: Array<T>;
	page: { next_cursor: string | null };
}

type PaginatedFetcher<T> = (cursor: string | null) => Promise<PaginatedResult<T> | undefined>;

export async function fetchAllPages<T>(fetcher: PaginatedFetcher<T>): Promise<T[]> {
	const all: T[] = [];
	let cursor: string | null = null;

	for (;;) {
		const result = await fetcher(cursor);
		if (!result) break;

		all.push(...result.data);

		if (!result.page.next_cursor || all.length >= MAX_ITEMS) break;
		cursor = result.page.next_cursor;
	}

	return all;
}

export interface CollectionNode {
	collection: import('$lib/api/generated/types.gen').CollectionResponse;
	children: CollectionNode[];
	expanded: boolean;
}

export function buildCollectionTree(
	collections: import('$lib/api/generated/types.gen').CollectionResponse[],
	expandedState: Record<string, boolean>
): CollectionNode[] {
	const byParent = new Map<string | null, CollectionNode[]>();

	for (const col of collections) {
		const parentKey = col.parent_id ?? null;
		const node: CollectionNode = {
			collection: col,
			children: [],
			expanded: expandedState[col.id] ?? false
		};
		const siblings = byParent.get(parentKey);
		if (siblings) {
			siblings.push(node);
		} else {
			byParent.set(parentKey, [node]);
		}
	}

	function attachChildren(parentId: string | null): CollectionNode[] {
		const nodes = byParent.get(parentId) ?? [];
		for (const node of nodes) {
			node.children = attachChildren(node.collection.id);
		}
		return nodes;
	}

	return attachChildren(null);
}

export function buildBreadcrumbPath(
	collectionId: string,
	collections: import('$lib/api/generated/types.gen').CollectionResponse[]
): import('$lib/api/generated/types.gen').CollectionResponse[] {
	const byId = new Map(collections.map((c) => [c.id, c]));
	const path: import('$lib/api/generated/types.gen').CollectionResponse[] = [];
	let current = byId.get(collectionId);
	const seen = new Set<string>();

	while (current && !seen.has(current.id)) {
		seen.add(current.id);
		path.unshift(current);
		current = current.parent_id ? byId.get(current.parent_id) : undefined;
	}

	return path;
}
