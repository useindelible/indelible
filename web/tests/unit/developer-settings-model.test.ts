import { describe, expect, it, vi } from 'vitest';
import type { WebhookDelivery, WebhookEndpoint } from '$lib/api/webhooks';
import {
	countRecentDeliveries,
	deliveryRatePercent,
	formatDate,
	formatRelative,
	formatTime,
	groupCount,
	lastStatusClass,
	lastStatusLabel,
	scopeClass,
	setsEqual,
	statusClassFor
} from '../../src/routes/(app)/preferences/developer/developer-model';

function endpoint(overrides: Partial<WebhookEndpoint> = {}): WebhookEndpoint {
	return {
		id: 'wh_1',
		name: 'Automation',
		url: 'https://example.com/hook',
		events: ['library_entry.saved'],
		is_active: true,
		last_status: 'healthy',
		delivery_history: ['s2xx', 's4xx', 'pending'],
		secret_preview: 'whsec_abc...',
		created_at: '2026-06-10T12:00:00Z',
		updated_at: '2026-06-10T12:00:00Z',
		...overrides
	};
}

function delivery(overrides: Partial<WebhookDelivery> = {}): WebhookDelivery {
	return {
		id: 'del_1',
		endpoint_id: 'wh_1',
		event: 'library_entry.saved',
		target: 'Automation',
		status_code: 200,
		latency_ms: 42,
		attempt: 1,
		delivered_at: '2026-06-10T13:59:00Z',
		...overrides
	};
}

describe('developer settings model', () => {
	it('formats scopes, dates, times, and relative timestamps', () => {
		vi.setSystemTime(new Date('2026-06-10T14:00:00Z'));
		expect(scopeClass('extension')).toBe('ext');
		expect(scopeClass('obsidian_plugin')).toBe('obsidian');
		expect(formatRelative('2026-06-10T13:58:00Z')).toBe('2 minutes ago');
		expect(formatDate(null)).toBe('—');
		expect(formatDate('2026-06-10T12:00:00Z')).toContain('10');
		expect(formatTime('2026-06-10T12:34:56Z')).toMatch(/12:34:56|13:34:56|14:34:56/);
		vi.useRealTimers();
	});

	it('classifies webhook delivery and endpoint status', () => {
		expect(statusClassFor(204)).toBe('s2xx');
		expect(statusClassFor(404)).toBe('s4xx');
		expect(statusClassFor(500)).toBe('s5xx');
		expect(statusClassFor(null)).toBe('s5xx');
		expect(lastStatusClass(endpoint({ is_active: false }))).toBe('paused');
		expect(lastStatusLabel(endpoint({ last_status: 'failing' }))).toBe('Failing');
	});

	it('calculates delivery metrics and selected event counts', () => {
		vi.setSystemTime(new Date('2026-06-10T14:00:00Z'));
		const deliveries = [
			delivery(),
			delivery({ id: 'del_2', status_code: 500 }),
			delivery({ id: 'del_3', delivered_at: '2026-06-08T12:00:00Z' })
		];
		expect(countRecentDeliveries(deliveries)).toBe(2);
		expect(deliveryRatePercent(deliveries)).toBe('66.7');
		expect(groupCount(['a', 'b', 'c'], new Set(['a', 'c']))).toBe(2);
		vi.useRealTimers();
	});

	it('compares sets by value', () => {
		expect(setsEqual(new Set(['read']), new Set(['read']))).toBe(true);
		expect(setsEqual(new Set(['read']), new Set(['write']))).toBe(false);
	});
});
