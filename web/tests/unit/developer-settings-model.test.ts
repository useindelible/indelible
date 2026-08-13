import { describe, expect, it, vi } from 'vitest';
import type { WebhookEndpoint } from '$lib/api/webhooks';
import {
	formatDate,
	formatRelative,
	formatTime,
	groupCount,
	issuePresetFromSearchParams,
	ISSUE_DEFAULTS,
	lastStatusClass,
	lastStatusLabel,
	nextIssuePermissions,
	PERMISSION_CATALOGUE,
	permissionClass,
	resourceAccessLevel,
	setResourceAccess,
	setsEqual,
	toggleAllPermissions,
	tokenRequest,
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

describe('developer settings model', () => {
	it('formats permissions, dates, times, and relative timestamps', () => {
		vi.setSystemTime(new Date('2026-06-10T14:00:00Z'));
		expect(permissionClass('obsidian:sync')).toBe('obsidian');
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

	it('counts selected webhook events', () => {
		expect(groupCount(['a', 'b', 'c'], new Set(['a', 'c']))).toBe(2);
	});

	it('compares sets by value', () => {
		expect(setsEqual(new Set(['read']), new Set(['read']))).toBe(true);
		expect(setsEqual(new Set(['read']), new Set(['write']))).toBe(false);
	});

	it('starts empty and offers the complete permission catalogue in API order', () => {
		expect(ISSUE_DEFAULTS.permissions).toEqual([]);
		expect(PERMISSION_CATALOGUE).toEqual([
			'library:read',
			'library:write',
			'feeds:read',
			'feeds:write',
			'integrations:read',
			'integrations:write',
			'webhooks:read',
			'webhooks:write',
			'ai:read',
			'ai:write',
			'ai:use',
			'obsidian:sync'
		]);
	});

	it('renders unknown permissions neutrally', () => {
		expect(permissionClass('admin')).toBe('other');
		expect(permissionClass('extension')).toBe('other');
		expect(permissionClass('cli')).toBe('other');
	});

	it('groups acting permissions apart from reads', () => {
		expect(permissionClass('library:read')).toBe('read');
		expect(permissionClass('library:write')).toBe('write');
		// ai:use invokes models, so it must not read as a read-only grant.
		expect(permissionClass('ai:use')).toBe('write');
		expect(permissionClass('obsidian:sync')).toBe('obsidian');
	});

	it('expands resource write levels to truthful read and write permissions', () => {
		const write = setResourceAccess(['ai:use'], 'library', 'write');
		expect(write).toEqual(['library:read', 'library:write', 'ai:use']);
		expect(resourceAccessLevel(write, 'library')).toBe('write');

		const read = setResourceAccess(write, 'library', 'read');
		expect(read).toEqual(['library:read', 'ai:use']);
		expect(resourceAccessLevel(read, 'library')).toBe('read');

		expect(setResourceAccess(read, 'library', 'none')).toEqual(['ai:use']);
	});

	it('keeps AI use and Obsidian sync additive while enforcing write includes read', () => {
		expect(nextIssuePermissions(['ai:use'], 'obsidian:sync')).toEqual(['ai:use', 'obsidian:sync']);
		expect(nextIssuePermissions(['ai:use'], 'ai:write')).toEqual(['ai:read', 'ai:write', 'ai:use']);
		expect(nextIssuePermissions(['ai:read', 'ai:write', 'ai:use'], 'ai:read')).toEqual(['ai:use']);
	});

	it('explicitly selects and clears the full permission catalogue', () => {
		expect(toggleAllPermissions([])).toEqual(PERMISSION_CATALOGUE);
		expect(toggleAllPermissions(PERMISSION_CATALOGUE)).toEqual([]);
	});

	it('serializes the exact canonical permission and expiry request', () => {
		expect(tokenRequest('  Automation  ', ['webhooks:write', 'obsidian:sync'], '365')).toEqual({
			name: 'Automation',
			permissions: ['webhooks:read', 'webhooks:write', 'obsidian:sync'],
			expires_in: 31_536_000
		});
		expect(tokenRequest('Automation', ['library:read'], '30').expires_in).toBe(2_592_000);
		expect(tokenRequest('Automation', ['library:read'], '90').expires_in).toBe(7_776_000);
		expect(tokenRequest('Automation', ['library:read'], 'never').expires_in).toBeNull();
	});

	it('preselects only Obsidian sync from the developer deep link', () => {
		expect(issuePresetFromSearchParams(new URLSearchParams('permission=obsidian%3Async'))).toEqual({
			name: 'Obsidian plugin',
			permissions: ['obsidian:sync']
		});
		expect(
			issuePresetFromSearchParams(new URLSearchParams('permission=library%3Aread'))
		).toBeNull();
		expect(issuePresetFromSearchParams(new URLSearchParams())).toBeNull();
	});
});
