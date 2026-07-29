import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import FeedSubscriptionsTable from '../../src/routes/(app)/preferences/feed-management/components/FeedSubscriptionsTable.svelte';
import type { Feed } from '../../src/routes/(app)/preferences/feed-management/feed-model';

function feed(overrides: Partial<Feed> = {}): Feed {
	return {
		id: 'feed_1',
		name: 'Example Daily',
		domain: 'example.com',
		initials: 'ED',
		iconKey: 'blue',
		lastFetched: '10m ago',
		status: 'active',
		enabled: true,
		autoSave: true,
		errorMessage: undefined,
		autoSaveCollectionId: null,
		pollIntervalOverride: 30,
		titleOverride: null,
		inputUrl: 'https://example.com/rss.xml',
		...overrides
	};
}

function tableProps(overrides = {}) {
	return {
		feeds: [
			feed(),
			feed({
				id: 'feed_error',
				name: 'Broken Feed',
				status: 'error',
				errorMessage: 'connection timed out'
			})
		],
		openKebabId: null,
		onToggleAutoSave: vi.fn(),
		onToggleFeed: vi.fn(),
		onToggleMenu: vi.fn(),
		onCloseMenu: vi.fn(),
		onEdit: vi.fn(),
		onRetry: vi.fn(),
		onDelete: vi.fn(),
		...overrides
	};
}

describe('FeedSubscriptionsTable', () => {
	it('renders feeds with status, schedule, and error details', () => {
		render(FeedSubscriptionsTable, { props: tableProps() });
		expect(screen.getByText('Example Daily')).toBeTruthy();
		expect(screen.getAllByText('Every 30m')).toHaveLength(2);
		expect(screen.getByText('Broken Feed')).toBeTruthy();
		expect(screen.getByText('connection timed out')).toBeTruthy();
	});

	it('uses callback props for row actions', async () => {
		const props = tableProps({ openKebabId: 'feed_error' });
		render(FeedSubscriptionsTable, { props });

		await fireEvent.click(screen.getAllByRole('switch')[0]);
		await fireEvent.click(screen.getAllByLabelText('Feed actions')[0]);
		await fireEvent.click(screen.getAllByText('Retry now')[0]);
		await fireEvent.click(screen.getByText('Unsubscribe'));

		expect(props.onToggleAutoSave).toHaveBeenCalledWith('feed_1');
		expect(props.onToggleMenu).toHaveBeenCalledWith('feed_1', expect.any(MouseEvent));
		expect(props.onRetry).toHaveBeenCalledWith('feed_error');
		expect(props.onDelete).toHaveBeenCalledWith('feed_error');
	});
});
