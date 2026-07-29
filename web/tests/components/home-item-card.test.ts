import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import type { HomeItemResponse } from '$lib/api';
import HomeItemCard from '../../src/routes/(app)/dashboard/components/HomeItemCard.svelte';

function item(overrides: Partial<HomeItemResponse> = {}): HomeItemResponse {
	return {
		author: 'Ava Example @ava',
		created_at: '2026-06-10T10:00:00Z',
		domain: 'example.com',
		id: 'itm_card',
		item_type: 'article',
		progress_percent: 42,
		reading_time_minutes: 12,
		title: 'Designing durable reading systems',
		...overrides
	};
}

describe('HomeItemCard', () => {
	it('renders item identity, metadata, and progress', () => {
		render(HomeItemCard, {
			props: {
				item: item()
			}
		});

		expect(screen.getByText('Designing durable reading systems')).toBeTruthy();
		expect(screen.getByText('example.com')).toBeTruthy();
		expect(screen.getByText('Ava Example · 12 min read')).toBeTruthy();
		expect(screen.getByLabelText('42% read')).toBeTruthy();
	});

	it('uses callback props for opening an item', async () => {
		const onOpen = vi.fn();

		render(HomeItemCard, {
			props: {
				item: item({ id: 'itm_open' }),
				onOpen
			}
		});

		await fireEvent.click(
			screen.getByRole('button', { name: /designing durable reading systems/i })
		);
		expect(onOpen).toHaveBeenCalledWith('itm_open');
	});
});
