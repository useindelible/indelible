import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
	createTag: vi.fn(),
	listTags: vi.fn()
}));

vi.mock('$lib/api', () => mocks);

import type { TagResponse } from '$lib/api';
import { getTags } from './tags.svelte';

function tag(overrides: Partial<TagResponse> = {}): TagResponse {
	return {
		aliases: [],
		color: null,
		created_at: '2026-08-12T00:00:00Z',
		highlight_count: 0,
		id: 'tag_new',
		item_count: 0,
		name: 'New tag',
		object: 'tag',
		parent_id: null,
		...overrides
	};
}

describe('tag creation', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('returns the server detail when the generated client resolves a conflict', async () => {
		mocks.createTag.mockResolvedValue({
			data: undefined,
			error: {
				type: 'https://indelible.app/problems/conflict',
				title: 'Conflict',
				status: 409,
				detail: "Tag conflict: tag with name 'QA-WEB' already exists",
				code: 'conflict'
			},
			response: new Response(null, { status: 409 })
		});

		const result = await getTags().createTag({ name: 'QA-WEB' });

		expect(result).toEqual({
			ok: false,
			error: "Tag conflict: tag with name 'QA-WEB' already exists"
		});
	});

	it('prefers a field error over a generic detail', async () => {
		mocks.createTag.mockResolvedValue({
			data: undefined,
			error: {
				detail: 'validation error',
				errors: [{ field: 'name', message: 'A tag with this name already exists.' }]
			}
		});

		await expect(getTags().createTag({ name: 'Duplicate' })).resolves.toEqual({
			ok: false,
			error: 'A tag with this name already exists.'
		});
	});

	it('returns the created tag on success', async () => {
		const created = tag();
		mocks.createTag.mockResolvedValue({ data: created, error: undefined });

		await expect(getTags().createTag({ name: created.name })).resolves.toEqual({
			ok: true,
			data: created
		});
	});

	it('looks up an existing tag from an unscoped catalogue during initial loading', async () => {
		const existing = tag({ id: 'tag_existing', name: 'qa-web' });
		mocks.listTags.mockResolvedValue({
			data: {
				data: [existing],
				page: { has_more: false, next_cursor: null }
			}
		});

		await expect(getTags().findTagByExactName('QA-WEB')).resolves.toEqual(existing);
		expect(mocks.listTags).toHaveBeenCalledWith({
			query: { cursor: null, limit: 100 }
		});
	});
});
