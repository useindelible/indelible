import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { FeedDeliveryResponse } from '$lib/api/generated/types.gen';

// TASK-230: the Feed page reads the new feed_deliveries API. These tests drive the real
// ItemList/ItemRow to cover nullable document_id rows, the Unseen/Seen tabs, the save handoff
// to the Library from-delivery endpoint, mark-all-seen, and external-open (mark seen + open
// url, never save/prepare).

const mockListFeedDeliveries = vi.fn();
const mockSaveFromDelivery = vi.fn();
const mockMarkDeliverySeen = vi.fn();
const mockMarkAllDeliveriesSeen = vi.fn();
const mockPrepareFeedDelivery = vi.fn();
const mockWindowOpen = vi.fn();
const mockGoto = vi.fn();

class MockIntersectionObserver {
	observe = vi.fn();
	disconnect = vi.fn();
}

vi.mock('$lib/api', () => ({
	listFeedDeliveries: (...args: unknown[]) => mockListFeedDeliveries(...args),
	saveFromDelivery: (...args: unknown[]) => mockSaveFromDelivery(...args),
	markDeliverySeen: (...args: unknown[]) => mockMarkDeliverySeen(...args),
	markAllDeliveriesSeen: (...args: unknown[]) => mockMarkAllDeliveriesSeen(...args),
	prepareFeedDelivery: (...args: unknown[]) => mockPrepareFeedDelivery(...args)
}));
vi.mock('$app/navigation', () => ({ goto: (...args: unknown[]) => mockGoto(...args) }));
vi.mock('$app/paths', () => ({ resolve: (path: string) => path }));

vi.mock('$lib/components/library/DetailPanel.svelte', () => ({
	default: vi.fn((_, props: { item: { title: string } | null }) => ({
		c: vi.fn(),
		m: (target: HTMLElement) => {
			const el = document.createElement('aside');
			el.dataset.testid = 'detail-panel';
			el.textContent = props.item?.title ?? '';
			target.appendChild(el);
		},
		p: vi.fn(),
		d: vi.fn()
	}))
}));

import FeedPage from '../../src/routes/(app)/feed/+page.svelte';

function delivery(overrides: Partial<FeedDeliveryResponse> = {}): FeedDeliveryResponse {
	return {
		object: 'feed_delivery',
		delivery_id: 'dlv_1',
		source_entry_id: 'fse_1',
		subscription_id: 'fed_1',
		source_id: 'fso_1',
		title: 'Part 7: Testing with Confidence',
		url: 'https://example.com/testing',
		delivered_at: '2026-05-16T00:54:02Z',
		saved: false,
		...overrides
	};
}

describe('Feed page (deliveries)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.stubGlobal('IntersectionObserver', MockIntersectionObserver);
		vi.stubGlobal('open', mockWindowOpen);
		mockListFeedDeliveries.mockResolvedValue({
			data: { data: [delivery()], page: { has_more: false, next_cursor: null } }
		});
		mockSaveFromDelivery.mockResolvedValue({ data: { document_id: 'doc_1' } });
		mockMarkDeliverySeen.mockResolvedValue({ data: null });
		mockMarkAllDeliveriesSeen.mockResolvedValue({ data: { updated: 1 } });
	});

	it('shows only unseen and seen feed filters', async () => {
		render(FeedPage);

		expect(await screen.findByRole('tab', { name: 'Unseen' })).toBeTruthy();
		expect(screen.getByRole('tab', { name: 'Seen' })).toBeTruthy();
		expect(screen.queryByRole('tab', { name: 'All' })).toBeNull();
		expect(screen.queryByRole('tab', { name: 'Saved' })).toBeNull();
	});

	it('renders an unprepared (document_id null) delivery from its source entry', async () => {
		render(FeedPage);

		expect(await screen.findByText('Part 7: Testing with Confidence')).toBeTruthy();
		const firstCall = mockListFeedDeliveries.mock.calls[0][0] as { query: { state: string } };
		expect(firstCall.query.state).toBe('unseen');
	});

	it('opens a delivery into the canonical reader by preparing then navigating', async () => {
		// Phase 7: opening a delivery prepares it (idempotent, marks seen, enqueues render) and
		// navigates to the document reader, rather than opening the publisher URL.
		mockPrepareFeedDelivery.mockResolvedValue({ data: { document_id: 'doc_x' } });
		render(FeedPage);

		await fireEvent.click(await screen.findByText('Part 7: Testing with Confidence'));

		await waitFor(() => {
			expect(mockPrepareFeedDelivery).toHaveBeenCalledWith({ path: { delivery_id: 'dlv_1' } });
			// resolve() runs first; goto receives its resolved-path return value.
			expect(mockGoto).toHaveBeenCalledWith('/(app)/reader/[documentId]');
		});
		expect(mockSaveFromDelivery).not.toHaveBeenCalled();
		expect(screen.queryByText('Part 7: Testing with Confidence')).toBeNull();
	});

	it('loads the Seen tab when the Seen segment is clicked', async () => {
		mockListFeedDeliveries.mockImplementation((request: { query?: { state?: string | null } }) => {
			const state = request.query?.state;
			const row =
				state === 'seen'
					? delivery({ delivery_id: 'dlv_seen', title: 'Already Opened Story' })
					: delivery({ delivery_id: 'dlv_unseen', title: 'Fresh Feed Story' });
			return Promise.resolve({
				data: { data: [row], page: { has_more: false, next_cursor: null } }
			});
		});

		render(FeedPage);
		await screen.findByText('Fresh Feed Story');

		await fireEvent.click(screen.getByRole('tab', { name: 'Seen' }));

		expect(await screen.findByText('Already Opened Story')).toBeTruthy();
	});

	it('saves a delivery via Move to Later (Library from-delivery) and removes the row', async () => {
		mockListFeedDeliveries.mockResolvedValueOnce({
			data: {
				data: [delivery({ delivery_id: 'dlv_save', title: 'Save This Story' })],
				page: { has_more: false, next_cursor: null }
			}
		});

		render(FeedPage);

		const row = (await screen.findByText('Save This Story')).closest('[role="option"]');
		expect(row).not.toBeNull();
		await fireEvent.mouseEnter(row as Element);
		await fireEvent.click(await screen.findByRole('button', { name: 'Move to Later' }));

		await waitFor(() => {
			expect(mockSaveFromDelivery).toHaveBeenCalledWith({ body: { delivery_id: 'dlv_save' } });
			expect(screen.queryByText('Save This Story')).toBeNull();
		});
		expect(mockMarkDeliverySeen).not.toHaveBeenCalled();
	});

	it('marks all deliveries seen', async () => {
		render(FeedPage);
		await screen.findByText('Part 7: Testing with Confidence');

		await fireEvent.click(screen.getByRole('button', { name: 'Mark all seen' }));

		expect(mockMarkAllDeliveriesSeen).toHaveBeenCalledWith({ body: {} });
	});
});
