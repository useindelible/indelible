import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import type { FeedSubscriptionResponse } from '$lib/api';

const { subscribe } = vi.hoisted(() => ({
	subscribe: vi.fn()
}));

vi.mock('$app/paths', () => ({
	resolve: (path: string) => path
}));

vi.mock('$lib/api', () => ({ subscribe }));

vi.mock('$lib/stores/auth.svelte', () => ({
	getAuth: () => ({ user: null })
}));

import AddToFeedPage from '../src/routes/(app)/preferences/add-to-feed/+page.svelte';

const existingSubscription: FeedSubscriptionResponse = {
	id: 'fed_existing',
	object: 'feed_subscription',
	input_url: 'https://example.com/feed.xml',
	title_override: null,
	auto_save: false,
	auto_save_collection_id: null,
	poll_interval_override_minutes: null,
	status: 'active',
	created_at: '2026-08-12T10:00:00Z',
	updated_at: '2026-08-12T10:00:00Z',
	source: {
		id: 'src_example',
		object: 'feed_source',
		url: 'https://example.com/feed.xml',
		poll_url: 'https://example.com/feed.xml',
		name: 'Example Feed',
		description: 'Example feed description',
		site_url: 'https://example.com',
		image_url: null,
		domain: 'example.com',
		source_kind: 'rss',
		visibility: 'public',
		provider: null,
		is_resolvable: true,
		popularity: 1,
		last_entry_added_at: null,
		last_polled_at: '2026-08-12T10:00:00Z',
		next_poll_at: '2026-08-12T10:15:00Z',
		consecutive_failures: 0,
		last_error: null
	}
};

describe('Add to Feed preferences page', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		subscribe.mockResolvedValue({
			data: { subscription: existingSubscription, is_new: false },
			error: undefined,
			response: new Response(null, { status: 200 })
		});
	});

	it('shows a friendly result when the feed is already subscribed', async () => {
		render(AddToFeedPage);
		await fireEvent.input(screen.getByPlaceholderText('https://example.com/feed.xml'), {
			target: { value: 'https://example.com/feed.xml' }
		});
		await fireEvent.click(screen.getByRole('button', { name: 'Subscribe' }));

		await waitFor(() => expect(screen.getByText('Already subscribed.')).toBeTruthy());
		const managementLink = screen.getByRole('link', { name: 'Manage in Feed Management' });
		expect(managementLink.getAttribute('href')).toBe('/preferences/feed-management');
		expect(screen.queryByText(/fed_existing/)).toBeNull();
		expect(screen.queryByText(/FeedSubscription conflict/i)).toBeNull();
	});
});
