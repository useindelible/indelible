import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { TagResponse } from '$lib/api/generated/types.gen';

const mocks = vi.hoisted(() => ({
	createTag: vi.fn(),
	listTags: vi.fn(),
	goto: vi.fn()
}));

vi.mock('$lib/api', () => ({
	createTag: (...args: unknown[]) => mocks.createTag(...args),
	listTags: (...args: unknown[]) => mocks.listTags(...args)
}));
vi.mock('$app/navigation', () => ({
	goto: (...args: unknown[]) => mocks.goto(...args)
}));
vi.mock('$app/paths', () => ({
	resolve: (_route: string, params: { id: string }) => `/tags/${params.id}`
}));

import TagsPage from '../../src/routes/(app)/tags/+page.svelte';

function tag(overrides: Partial<TagResponse> = {}): TagResponse {
	return {
		aliases: [],
		color: null,
		created_at: '2026-08-12T00:00:00Z',
		highlight_count: 0,
		id: 'tag_existing',
		item_count: 1,
		name: 'qa-web',
		object: 'tag',
		parent_id: null,
		...overrides
	};
}

function tagPage(tags: TagResponse[]) {
	return {
		data: {
			data: tags,
			page: { has_more: false, next_cursor: null }
		}
	};
}

async function openCreateDialog(name: string): Promise<HTMLInputElement> {
	await fireEvent.click(screen.getByRole('button', { name: 'New Tag' }));
	const input = screen.getByPlaceholderText('Tag name…') as HTMLInputElement;
	await fireEvent.input(input, { target: { value: name } });
	return input;
}

describe('tags page creation', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mocks.listTags.mockResolvedValue(tagPage([tag()]));
	});

	it('keeps a duplicate name selected and offers the existing tag', async () => {
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

		render(TagsPage);
		await screen.findByText('qa-web');
		const input = await openCreateDialog('QA-WEB');

		await fireEvent.click(screen.getByRole('button', { name: 'Create tag' }));

		expect((await screen.findByRole('alert')).textContent).toContain(
			"Tag conflict: tag with name 'QA-WEB' already exists"
		);
		expect(input.value).toBe('QA-WEB');
		await waitFor(() => {
			expect(document.activeElement).toBe(input);
			expect(input.selectionStart).toBe(0);
			expect(input.selectionEnd).toBe(6);
		});

		await fireEvent.click(screen.getByRole('button', { name: 'Open qa-web' }));
		expect(mocks.goto).toHaveBeenCalledWith('/tags/tag_existing');
		expect(screen.queryByRole('dialog', { name: 'New tag' })).toBeNull();

		await fireEvent.click(screen.getByRole('button', { name: 'New Tag' }));
		expect((screen.getByPlaceholderText('Tag name…') as HTMLInputElement).value).toBe('');
		expect(screen.queryByRole('alert')).toBeNull();
	});

	it('clears a creation error when the name changes', async () => {
		mocks.createTag.mockResolvedValue({
			data: undefined,
			error: { status: 409, detail: "Tag conflict: tag with name 'QA-WEB' already exists" }
		});

		render(TagsPage);
		await screen.findByText('qa-web');
		const input = await openCreateDialog('QA-WEB');
		await fireEvent.click(screen.getByRole('button', { name: 'Create tag' }));
		await screen.findByRole('alert');

		await fireEvent.input(input, { target: { value: 'qa-web-notes' } });

		expect(input.value).toBe('qa-web-notes');
		expect(screen.queryByRole('alert')).toBeNull();
	});

	it('closes and navigates after a successful creation', async () => {
		const created = tag({ id: 'tag_new', name: 'New research' });
		mocks.createTag.mockResolvedValue({ data: created, error: undefined });
		mocks.listTags
			.mockResolvedValueOnce(tagPage([tag()]))
			.mockResolvedValueOnce(tagPage([tag(), created]));

		render(TagsPage);
		await screen.findByText('qa-web');
		await openCreateDialog(created.name);

		await fireEvent.click(screen.getByRole('button', { name: 'Create tag' }));

		await waitFor(() => {
			expect(mocks.goto).toHaveBeenCalledWith('/tags/tag_new');
			expect(screen.queryByRole('dialog', { name: 'New tag' })).toBeNull();
		});
	});

	it('offers an existing tag hidden by the active scope', async () => {
		const existing = tag({ item_count: 0, highlight_count: 1 });
		const visibleDocumentTag = tag({ id: 'tag_document', name: 'Document research' });
		mocks.listTags.mockImplementation((request: { query: { scope?: string } }) =>
			Promise.resolve(
				tagPage(
					request.query.scope === 'document' ? [visibleDocumentTag] : [existing, visibleDocumentTag]
				)
			)
		);
		mocks.createTag.mockResolvedValue({
			data: undefined,
			error: { status: 409, detail: "Tag conflict: tag with name 'QA-WEB' already exists" }
		});

		render(TagsPage);
		await screen.findByText('qa-web');
		await fireEvent.click(screen.getByRole('button', { name: 'Document' }));
		await screen.findByText('Document research');
		expect(screen.queryByText('qa-web')).toBeNull();

		await openCreateDialog('QA-WEB');
		await fireEvent.click(screen.getByRole('button', { name: 'Create tag' }));

		await screen.findByRole('alert');
		await fireEvent.click(await screen.findByRole('button', { name: 'Open qa-web' }));

		expect(mocks.goto).toHaveBeenCalledWith('/tags/tag_existing');
		expect(mocks.listTags).toHaveBeenCalledTimes(1);
		expect(mocks.listTags.mock.calls[0][0].query.scope).toBeUndefined();
	});
});
