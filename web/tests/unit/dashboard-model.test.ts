import { describe, expect, it } from 'vitest';
import type { HomeItemResponse } from '$lib/api';
import {
	coverColor,
	domainInitial,
	greetingLine,
	longReadItems,
	readingMeta,
	reorder
} from '../../src/routes/(app)/dashboard/dashboard-model';

function item(overrides: Partial<HomeItemResponse> = {}): HomeItemResponse {
	return {
		created_at: '2026-06-10T10:00:00Z',
		domain: 'example.com',
		id: 'itm_1',
		item_type: 'article',
		title: 'A careful read',
		...overrides
	};
}

describe('dashboard model', () => {
	it('reorders items without mutating the original list', () => {
		const source = ['continue', 'quick', 'recent'];

		expect(reorder(source, 0, 2)).toEqual(['quick', 'recent', 'continue']);
		expect(source).toEqual(['continue', 'quick', 'recent']);
	});

	it('builds stable greeting copy', () => {
		expect(greetingLine('Sam', 8)).toBe('Good morning, Sam.');
		expect(greetingLine('Sam', 14)).toBe('Good afternoon, Sam.');
		expect(greetingLine(null, 22)).toBe('Good evening.');
	});

	it('derives card metadata from home items', () => {
		const withAuthor = item({
			author: 'Alex Reader @alex',
			domain: 'news.example',
			reading_time_minutes: 75
		});

		expect(readingMeta(withAuthor)).toBe('Alex Reader · 1h 15m read');
		expect(domainInitial(withAuthor.domain)).toBe('N');
		expect(coverColor(withAuthor.domain)).toBe('teal');
	});

	it('selects long reads from recent items', () => {
		const short = item({ id: 'short', reading_time_minutes: 8 });
		const long = item({ id: 'long', reading_time_minutes: 20 });
		const longer = item({ id: 'longer', reading_time_minutes: 42 });

		expect(longReadItems([short, long, longer]).map((entry) => entry.id)).toEqual([
			'long',
			'longer'
		]);
	});
});
