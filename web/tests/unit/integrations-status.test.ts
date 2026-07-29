import { describe, it, expect } from 'vitest';
import {
	deriveConnectionState,
	detectAuthFailure,
	detectRateLimit,
	isPollingStatus,
	isTerminalImportStatus,
	normalizeImportStatus
} from '$lib/integrations/status';
import type { IntegrationConnectionDto } from '$lib/api';

function connection(status: string): IntegrationConnectionDto {
	return {
		id: 'cnx_1',
		provider: 'obsidian',
		status: status as IntegrationConnectionDto['status'],
		last_sync_at: null,
		last_error: null,
		created_at: '2026-04-25T12:00:00Z',
		updated_at: '2026-04-25T12:00:00Z'
	} as IntegrationConnectionDto;
}

describe('deriveConnectionState', () => {
	it('returns disconnected when there is no connection', () => {
		expect(deriveConnectionState(undefined)).toBe('disconnected');
	});

	it('maps active to connected', () => {
		expect(deriveConnectionState(connection('active'))).toBe('connected');
	});

	it('maps syncing to syncing', () => {
		expect(deriveConnectionState(connection('syncing'))).toBe('syncing');
	});

	it('maps error to failed', () => {
		expect(deriveConnectionState(connection('error'))).toBe('failed');
	});

	it('maps paused and disabled to unavailable', () => {
		expect(deriveConnectionState(connection('paused'))).toBe('unavailable');
		expect(deriveConnectionState(connection('disabled'))).toBe('unavailable');
	});

	it('falls back to unavailable for unknown statuses', () => {
		expect(deriveConnectionState(connection('weird'))).toBe('unavailable');
	});
});

describe('normalizeImportStatus', () => {
	it.each([
		'awaiting_provider',
		'pending',
		'running',
		'completed',
		'failed',
		'partial',
		'rolled_back'
	])('returns %s unchanged for known statuses', (status) => {
		expect(normalizeImportStatus(status)).toBe(status);
	});

	it('returns unknown for unrecognised statuses', () => {
		expect(normalizeImportStatus('weird')).toBe('unknown');
	});
});

describe('isPollingStatus', () => {
	it('returns true for queued and active import worker states', () => {
		expect(isPollingStatus('awaiting_provider')).toBe(true);
		expect(isPollingStatus('pending')).toBe(true);
		expect(isPollingStatus('running')).toBe(true);
	});

	it('returns false for terminal statuses', () => {
		for (const status of ['completed', 'failed', 'partial', 'rolled_back']) {
			expect(isPollingStatus(status)).toBe(false);
		}
	});
});

describe('isTerminalImportStatus', () => {
	it('returns true for completed, failed, partial, rolled_back', () => {
		for (const status of ['completed', 'failed', 'partial', 'rolled_back']) {
			expect(isTerminalImportStatus(status)).toBe(true);
		}
	});

	it('returns false for pending, running, awaiting_provider', () => {
		for (const status of ['pending', 'running', 'awaiting_provider']) {
			expect(isTerminalImportStatus(status)).toBe(false);
		}
	});
});

describe('detectRateLimit', () => {
	it.each([
		'429 Too Many Requests',
		'HTTP 429',
		'rate limit exceeded for workspace',
		'Retry-After: 30',
		'retry-after header set'
	])('matches %s', (text) => {
		expect(detectRateLimit(text)).toBe(true);
	});

	it('is case-insensitive', () => {
		expect(detectRateLimit('RATE LIMIT EXCEEDED')).toBe(true);
		expect(detectRateLimit('Rate Limit')).toBe(true);
	});

	it('returns false for null, undefined, and empty', () => {
		expect(detectRateLimit(null)).toBe(false);
		expect(detectRateLimit(undefined)).toBe(false);
		expect(detectRateLimit('')).toBe(false);
	});

	it('returns false for unrelated errors', () => {
		expect(detectRateLimit('Database connection lost')).toBe(false);
		expect(detectRateLimit('500 Internal Server Error')).toBe(false);
	});
});

describe('detectAuthFailure', () => {
	it.each([
		'401 Unauthorized',
		'HTTP 401: token revoked',
		'unauthorized: refresh token expired',
		'Unauthorized'
	])('matches %s', (text) => {
		expect(detectAuthFailure(text)).toBe(true);
	});

	it('returns false for null, undefined, and empty', () => {
		expect(detectAuthFailure(null)).toBe(false);
		expect(detectAuthFailure(undefined)).toBe(false);
		expect(detectAuthFailure('')).toBe(false);
	});

	it('returns false for non-auth errors', () => {
		expect(detectAuthFailure('429 Too Many Requests')).toBe(false);
		expect(detectAuthFailure('500 Internal Server Error')).toBe(false);
	});
});
