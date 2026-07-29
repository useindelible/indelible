import { describe, expect, it, vi } from 'vitest';
import type { FeedSubscriptionResponse } from '$lib/api';
import {
	deriveInitials,
	extractDomain,
	filterFeeds,
	formatSchedule,
	mapSubscription,
	pollIntervalToMinutes,
	snapshotFeeds
} from '../../src/routes/(app)/preferences/feed-management/feed-model';

function subscription(overrides: Partial<FeedSubscriptionResponse> = {}): FeedSubscriptionResponse {
	return {
		id: 'feed_1',
		input_url: 'https://example.com/rss.xml',
		status: 'active',
		auto_save: true,
		auto_save_collection_id: null,
		poll_interval_override_minutes: null,
		title_override: null,
		source: {
			id: 'src_1',
			name: 'Example Daily',
			feed_url: 'https://example.com/rss.xml',
			site_url: 'https://example.com',
			domain: 'example.com',
			last_polled_at: '2026-06-10T13:30:00Z',
			last_error: null,
			consecutive_failures: 0
		},
		created_at: '2026-06-10T12:00:00Z',
		updated_at: '2026-06-10T12:00:00Z',
		...overrides
	} as FeedSubscriptionResponse;
}

describe('feed management model', () => {
	it('formats feed labels and schedules', () => {
		expect(deriveInitials('Example Daily')).toBe('ED');
		expect(deriveInitials('RSS')).toBe('RS');
		expect(extractDomain('https://www.example.com/feed')).toBe('example.com');
		expect(formatSchedule(null)).toBe('Default');
		expect(formatSchedule(30)).toBe('Every 30m');
		expect(formatSchedule(1440)).toBe('Daily');
		expect(pollIntervalToMinutes('default')).toBeNull();
		expect(pollIntervalToMinutes('60')).toBe(60);
	});

	it('maps subscriptions into the route feed view model', () => {
		vi.setSystemTime(new Date('2026-06-10T14:00:00Z'));
		const feed = mapSubscription(subscription());
		expect(feed).toMatchObject({
			id: 'feed_1',
			name: 'Example Daily',
			domain: 'example.com',
			initials: 'ED',
			status: 'active',
			enabled: true,
			autoSave: true,
			lastFetched: '30m ago'
		});
		vi.useRealTimers();
	});

	it('marks failed and paused subscriptions correctly', () => {
		expect(
			mapSubscription(subscription({ status: 'paused' as FeedSubscriptionResponse['status'] }))
				.status
		).toBe('paused');
		expect(
			mapSubscription(
				subscription({
					source: {
						...subscription().source,
						last_error: 'timeout',
						consecutive_failures: 2
					}
				})
			).status
		).toBe('error');
	});

	it('filters feeds by status, name, and domain', () => {
		const active = mapSubscription(subscription());
		const paused = { ...active, id: 'feed_2', name: 'Slow Dispatch', status: 'paused' as const };
		const feeds = [active, paused];
		expect(filterFeeds(feeds, 'paused', '')).toEqual([paused]);
		expect(filterFeeds(feeds, 'all', 'daily')).toEqual([active]);
		expect(filterFeeds(feeds, 'all', 'missing')).toEqual([]);
	});

	it('snapshots only save/discard fields', () => {
		const feed = mapSubscription(subscription());
		expect(JSON.parse(snapshotFeeds([feed]))).toEqual([
			{ id: 'feed_1', autoSave: true, autoSaveCollectionId: null }
		]);
	});
});
