import { describe, expect, it, vi } from 'vitest';
import type { EmailAliasResponse, EmailSenderResponse } from '$lib/api';
import {
	domainFromAddress,
	filterSenders,
	formatIssued,
	formatRelative,
	isQuiet,
	isValidLocalPart,
	primaryAlias,
	routingValue,
	senderCounts,
	senderInitial
} from '../../src/routes/(app)/preferences/email/email-model';

function alias(overrides: Partial<EmailAliasResponse> = {}): EmailAliasResponse {
	return {
		object: 'email_alias',
		id: 'als_1',
		local_part: 'newsletters',
		address: 'newsletters@feed.useindelible.com',
		destination: 'feed',
		status: 'active',
		is_default: true,
		retire_at: null,
		retired_at: null,
		created_at: '2026-03-01T00:00:00Z',
		...overrides
	};
}

function sender(overrides: Partial<EmailSenderResponse> = {}): EmailSenderResponse {
	return {
		object: 'email_sender',
		id: 'snd_1',
		canonical_addr: 'news@example.com',
		display_name: 'Daily News',
		list_id: 'daily-news.example.com',
		render_default: 'reader',
		routing_default: 'feed',
		blocked: false,
		blocked_at: null,
		delivery_count: 12,
		first_seen_at: '2026-01-01T00:00:00Z',
		last_seen_at: '2026-06-10T13:50:00Z',
		...overrides
	};
}

describe('email settings model', () => {
	it('selects primary aliases and validates local parts', () => {
		const newer = alias({ id: 'als_2', created_at: '2026-04-01T00:00:00Z' });
		expect(primaryAlias([alias(), newer], 'feed')?.id).toBe('als_2');
		expect(primaryAlias([alias({ retire_at: '2026-05-01T00:00:00Z' })], 'feed')).toBeNull();
		expect(isValidLocalPart('daily.notes')).toBe(true);
		expect(isValidLocalPart('.bad')).toBe(false);
		expect(domainFromAddress('hello@feed.useindelible.com')).toBe('@feed.useindelible.com');
	});

	it('formats sender fields and relative dates', () => {
		vi.setSystemTime(new Date('2026-06-10T14:00:00Z'));
		expect(formatRelative('2026-06-10T13:50:00Z')).toBe('10m ago');
		expect(formatIssued('2026-03-01T00:00:00Z')).toContain('2026');
		expect(senderInitial(sender())).toBe('D');
		expect(routingValue(sender({ routing_default: null }))).toBe('default');
		vi.useRealTimers();
	});

	it('filters sender lists and counts groups', () => {
		vi.setSystemTime(new Date('2026-06-10T14:00:00Z'));
		const senders = [
			sender(),
			sender({ id: 'snd_2', routing_default: 'library', canonical_addr: 'archive@example.com' }),
			sender({ id: 'snd_3', blocked: true, canonical_addr: 'blocked@example.com' }),
			sender({
				id: 'snd_4',
				last_seen_at: '2026-04-01T00:00:00Z',
				canonical_addr: 'quiet@example.com'
			})
		];
		expect(isQuiet(senders[3])).toBe(true);
		expect(senderCounts(senders)).toEqual({ all: 4, feed: 2, library: 1, blocked: 1, quiet: 1 });
		expect(filterSenders(senders, 'library', '')).toHaveLength(1);
		expect(filterSenders(senders, 'all', 'archive')[0].id).toBe('snd_2');
		vi.useRealTimers();
	});
});
