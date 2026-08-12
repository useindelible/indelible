import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import type { ItemResponse } from '$lib/api/generated/types.gen';
import DetailPanel from '$lib/components/library/DetailPanel.svelte';

vi.mock('$app/paths', () => ({
	resolve: (path: string) => path
}));

vi.mock('$lib/api', () => ({
	getMilaConfigApi: vi.fn(async () => ({ data: { enabled: false } })),
	listMilaSessions: vi.fn(async () => ({ data: { sessions: [] } })),
	createMilaSession: vi.fn(async () => ({ data: { id: 'mil_1' } })),
	getMilaSessionMessages: vi.fn(async () => ({ data: { messages: [] } })),
	listDocumentEntities: vi.fn(async () => ({ data: [] })),
	streamMilaChat: vi.fn()
}));

function item(id: string, title: string): ItemResponse {
	return {
		created_at: '2026-05-15T00:00:00Z',
		id,
		is_favorite: false,
		is_shortlisted: false,
		item_type: 'article',
		object: 'item',
		saved_at: '2026-05-15T00:00:00Z',
		source: 'extension',
		title,
		triage_state: 'inbox',
		updated_at: '2026-05-15T00:00:00Z'
	};
}

describe('DetailPanel collection chat scope', () => {
	it('uses collection chat directly on collection pages', async () => {
		render(DetailPanel, {
			props: {
				item: item('itm_1', 'Nested Item'),
				collectionId: 'col_1',
				collectionName: 'Research'
			}
		});

		expect(screen.queryByRole('tab', { name: 'Notebook' })).toBeNull();

		await fireEvent.click(screen.getByRole('tab', { name: 'Chat' }));

		expect(screen.queryByRole('tab', { name: 'Item' })).toBeNull();
		expect(screen.queryByRole('tab', { name: 'Collection' })).toBeNull();
		expect(screen.getByPlaceholderText('Ask Mila about this collection…')).toBeTruthy();
	});

	it('keeps collection chat active when the selected item changes', async () => {
		const rendered = render(DetailPanel, {
			props: {
				item: item('itm_1', 'First Item'),
				collectionId: 'col_1',
				collectionName: 'Research'
			}
		});

		await fireEvent.click(screen.getByRole('tab', { name: 'Chat' }));
		expect(screen.getByPlaceholderText('Ask Mila about this collection…')).toBeTruthy();

		await rendered.rerender({
			item: item('itm_2', 'Second Item'),
			collectionId: 'col_1',
			collectionName: 'Research'
		});

		expect(screen.getByPlaceholderText('Ask Mila about this collection…')).toBeTruthy();
	});

	it('keeps item chat and notebook tabs outside collection pages', async () => {
		render(DetailPanel, {
			props: {
				item: item('itm_1', 'Standalone Item')
			}
		});

		expect(screen.getByRole('tab', { name: 'Notebook' })).toBeTruthy();

		await fireEvent.click(screen.getByRole('tab', { name: 'Chat' }));

		expect(screen.getByPlaceholderText('Ask Mila about this article...')).toBeTruthy();
	});

	it('keeps Info and Notebook but removes Chat when document text is unavailable', () => {
		render(DetailPanel, {
			props: {
				item: item('itm_video', 'Metadata-only video'),
				chatAvailable: false
			}
		});

		expect(screen.getByRole('tab', { name: 'Info' })).toBeTruthy();
		expect(screen.getByRole('tab', { name: 'Notebook' })).toBeTruthy();
		expect(screen.queryByRole('tab', { name: 'Chat' })).toBeNull();
	});

	it('returns to Info when text becomes unavailable while Chat is active', async () => {
		const rendered = render(DetailPanel, {
			props: { item: item('itm_video', 'Metadata-only video') }
		});
		await fireEvent.click(screen.getByRole('tab', { name: 'Chat' }));
		expect(screen.getByPlaceholderText('Ask Mila about this article...')).toBeTruthy();

		await rendered.rerender({
			item: item('itm_video', 'Metadata-only video'),
			chatAvailable: false
		});

		expect(screen.queryByRole('tab', { name: 'Chat' })).toBeNull();
		expect(screen.getByRole('tab', { name: 'Info' }).getAttribute('aria-selected')).toBe('true');
	});
});
