import { describe, expect, it } from 'vitest';
import type { TagResponse } from '$lib/api/generated/types.gen';
import { get } from 'svelte/store';
import { t } from '$lib/i18n';
import {
	buildTagTree,
	collectDescendantIds,
	getTagCountLabel,
	parentOptions,
	rolledUpItemCounts,
	tagDisplayColor
} from '../../src/routes/(app)/tags/tag-tree';

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

describe('tag tree model', () => {
	const translate = get(t);

	it('builds a depth-first tree that respects expanded parents', () => {
		const tags = [
			tag({ id: 'a', name: 'A', item_count: 1 }),
			tag({ id: 'b', parent_id: 'a', name: 'B', item_count: 2 }),
			tag({ id: 'c', parent_id: 'b', name: 'C', item_count: 3 }),
			tag({ id: 'd', name: 'D', item_count: 4 })
		];

		expect(buildTagTree(tags, new Set()).map((node) => node.tag.id)).toEqual(['a', 'd']);
		expect(
			buildTagTree(tags, new Set(['a', 'b'])).map((node) => [node.tag.id, node.depth])
		).toEqual([
			['a', 0],
			['b', 1],
			['c', 2],
			['d', 0]
		]);
	});

	it('rolls document counts through descendants and labels active scopes', () => {
		const tags = [
			tag({ id: 'a', item_count: 1, highlight_count: 2 }),
			tag({ id: 'b', parent_id: 'a', item_count: 3, highlight_count: 4 })
		];
		const rolledUp = rolledUpItemCounts(tags);

		expect(rolledUp.get('a')).toBe(4);
		expect(getTagCountLabel(translate, tags[0], 'document', rolledUp)).toBe('4 items');
		expect(getTagCountLabel(translate, tags[0], 'highlight', rolledUp)).toBe('2 highlights');
		expect(getTagCountLabel(translate, tags[0], 'all', rolledUp)).toBe('6 items');
	});

	it('excludes self and descendants from parent options', () => {
		const tags = [tag({ id: 'a' }), tag({ id: 'b', parent_id: 'a' }), tag({ id: 'c' })];

		expect([...collectDescendantIds(tags, 'a')]).toEqual(['b']);
		expect(parentOptions(tags, tags[0]).map((option) => option.id)).toEqual(['c']);
	});

	it('sanitizes display colors and falls back to a token', () => {
		expect(tagDisplayColor(tag({ color: '#0A84FF' }))).toBe('#0A84FF');
		expect(tagDisplayColor(tag({ color: 'url(bad)' }))).toBe('var(--text-tertiary)');
	});
});
