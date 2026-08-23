import type { TagResponse } from '$lib/api/generated/types.gen';
import type { Translate } from '$lib/i18n';
import { sanitizeColor } from '$lib/utils/color';

export type TagScope = 'all' | 'document' | 'highlight';

export interface TagNode {
	tag: TagResponse;
	depth: number;
	hasChildren: boolean;
}

export function buildTagTree(tags: TagResponse[], expandedParents: Set<string>): TagNode[] {
	const tagIds = new Set(tags.map((tag) => tag.id));
	const childrenMap = new Map<string, TagResponse[]>();
	for (const tag of tags) {
		if (tag.parent_id && tagIds.has(tag.parent_id)) {
			const children = childrenMap.get(tag.parent_id) ?? [];
			children.push(tag);
			childrenMap.set(tag.parent_id, children);
		}
	}

	const roots = tags.filter((tag) => !tag.parent_id || !tagIds.has(tag.parent_id));
	const nodes: TagNode[] = [];
	const visited = new Set<string>();

	function visit(tag: TagResponse, depth: number) {
		if (visited.has(tag.id)) return;
		visited.add(tag.id);

		const children = childrenMap.get(tag.id) ?? [];
		nodes.push({ tag, depth, hasChildren: children.length > 0 });
		if (expandedParents.has(tag.id)) {
			for (const child of children) visit(child, depth + 1);
		}
	}

	for (const root of roots) visit(root, 0);
	if (roots.length === 0) {
		for (const tag of tags) visit(tag, 0);
	}
	return nodes;
}

export function rolledUpItemCounts(tags: TagResponse[]): Map<string, number> {
	const childrenMap = new Map<string, string[]>();
	for (const tag of tags) {
		if (!tag.parent_id) continue;
		const children = childrenMap.get(tag.parent_id) ?? [];
		children.push(tag.id);
		childrenMap.set(tag.parent_id, children);
	}

	const directCount = new Map<string, number>(tags.map((tag) => [tag.id, tag.item_count]));
	const memo = new Map<string, number>();
	const visiting = new Set<string>();

	function sum(id: string): number {
		if (memo.has(id)) return memo.get(id)!;
		if (visiting.has(id)) return directCount.get(id) ?? 0;
		visiting.add(id);
		const total =
			(directCount.get(id) ?? 0) +
			(childrenMap.get(id) ?? []).reduce((acc, childId) => acc + sum(childId), 0);
		memo.set(id, total);
		visiting.delete(id);
		return total;
	}

	for (const tag of tags) memo.set(tag.id, sum(tag.id));
	return memo;
}

export function getTagCount(
	tag: TagResponse,
	activeScope: TagScope,
	rolledUpCounts: Map<string, number>
): number {
	if (activeScope === 'highlight') return tag.highlight_count;
	if (activeScope === 'document') return rolledUpCounts.get(tag.id) ?? tag.item_count;
	return (rolledUpCounts.get(tag.id) ?? tag.item_count) + tag.highlight_count;
}

export function getTagCountLabel(
	translate: Translate,
	tag: TagResponse,
	activeScope: TagScope,
	rolledUpCounts: Map<string, number>
): string {
	const count = getTagCount(tag, activeScope, rolledUpCounts);
	return translate(activeScope === 'highlight' ? 'tag_highlight_count' : 'tag_item_count', {
		values: { count }
	});
}

export function collectDescendantIds(tags: TagResponse[], rootId: string): Set<string> {
	const descendants = new Set<string>();
	const queue = [rootId];

	while (queue.length > 0) {
		const currentId = queue.shift();
		if (!currentId) continue;

		for (const tag of tags) {
			if (tag.parent_id !== currentId || descendants.has(tag.id)) continue;
			descendants.add(tag.id);
			queue.push(tag.id);
		}
	}

	return descendants;
}

export function parentOptions(tags: TagResponse[], tag: TagResponse): TagResponse[] {
	const descendantIds = collectDescendantIds(tags, tag.id);
	return tags.filter((candidate) => candidate.id !== tag.id && !descendantIds.has(candidate.id));
}

export function tagDisplayColor(tag: TagResponse): string {
	return sanitizeColor(tag.color) ?? 'var(--text-tertiary)';
}
