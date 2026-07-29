import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';

import type { TagResponse } from '$lib/api/generated/types.gen';
import HighlightTagPicker from '$lib/components/reader/HighlightTagPicker.svelte';

function tag(overrides: Partial<TagResponse> = {}): TagResponse {
	return {
		id: 'tag_1',
		object: 'tag',
		name: 'Mila',
		color: '#0A84FF',
		aliases: [],
		item_count: 0,
		highlight_count: 3,
		parent_id: null,
		created_at: '2026-01-01T00:00:00Z',
		...overrides
	};
}

describe('HighlightTagPicker', () => {
	it('renders applied tags and removes them through a callback', async () => {
		const onRemoveTag = vi.fn();

		render(HighlightTagPicker, {
			props: {
				x: 100,
				y: 200,
				above: false,
				tags: ['Research'],
				tagInput: '',
				suggestions: [],
				suggestionIndex: 0,
				onTagInputChange: vi.fn(),
				onSuggestionIndexChange: vi.fn(),
				onAddTag: vi.fn(),
				onRemoveTag,
				onClose: vi.fn()
			}
		});

		await fireEvent.click(screen.getByRole('button', { name: 'Remove Research' }));

		expect(onRemoveTag).toHaveBeenCalledWith('Research');
	});

	it('selects a suggested tag through a callback', async () => {
		const onAddTag = vi.fn();

		render(HighlightTagPicker, {
			props: {
				x: 100,
				y: 200,
				above: true,
				tags: [],
				tagInput: 'mi',
				suggestions: [tag()],
				suggestionIndex: 0,
				onTagInputChange: vi.fn(),
				onSuggestionIndexChange: vi.fn(),
				onAddTag,
				onRemoveTag: vi.fn(),
				onClose: vi.fn()
			}
		});

		await fireEvent.click(screen.getByRole('button', { name: /Mila/ }));

		expect(onAddTag).toHaveBeenCalledWith('Mila');
	});
});
