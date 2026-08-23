import type { WebhookEndpoint } from '$lib/api/webhooks';
import type { ApiPermissionDto, CreateApiTokenRequest } from '$lib/api/generated/types.gen';
import { date, time, type MessageKey } from '$lib/i18n';
import { relativeTime } from '$lib/utils/relative-time';
import { get } from 'svelte/store';

export type PermissionKey = ApiPermissionDto;
export type ResourcePermissionKey = 'library' | 'feeds' | 'integrations' | 'webhooks';
export type ResourceAccessLevel = 'none' | 'read' | 'write';
export type ExpiryOption = 'never' | '30' | '90' | '365';

export interface ResourcePermissionGroup {
	key: ResourcePermissionKey;
	labelKey: MessageKey;
	descKey: MessageKey;
	read: PermissionKey;
	write: PermissionKey;
}

export interface IndependentPermissionDef {
	key: PermissionKey;
	labelKey: MessageKey;
	descKey: MessageKey;
}

export interface WebhookEventGroup {
	key: string;
	name: string;
	events: string[];
}

export const PERMISSION_CATALOGUE: PermissionKey[] = [
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
];

export const RESOURCE_PERMISSION_GROUPS: ResourcePermissionGroup[] = [
	{
		key: 'library',
		labelKey: 'prefs_developer_resource_library',
		descKey: 'prefs_developer_resource_library_hint',
		read: 'library:read',
		write: 'library:write'
	},
	{
		key: 'feeds',
		labelKey: 'prefs_developer_resource_feeds',
		descKey: 'prefs_developer_resource_feeds_hint',
		read: 'feeds:read',
		write: 'feeds:write'
	},
	{
		key: 'integrations',
		labelKey: 'prefs_developer_resource_integrations',
		descKey: 'prefs_developer_resource_integrations_hint',
		read: 'integrations:read',
		write: 'integrations:write'
	},
	{
		key: 'webhooks',
		labelKey: 'prefs_developer_resource_webhooks',
		descKey: 'prefs_developer_resource_webhooks_hint',
		read: 'webhooks:read',
		write: 'webhooks:write'
	}
];

export const INDEPENDENT_PERMISSION_DEFS: IndependentPermissionDef[] = [
	{
		key: 'ai:read',
		labelKey: 'prefs_developer_permission_ai_read',
		descKey: 'prefs_developer_permission_ai_read_hint'
	},
	{
		key: 'ai:write',
		labelKey: 'prefs_developer_permission_ai_configure',
		descKey: 'prefs_developer_permission_ai_configure_hint'
	},
	{
		key: 'ai:use',
		labelKey: 'prefs_developer_permission_ai_use',
		descKey: 'prefs_developer_permission_ai_use_hint'
	},
	{
		key: 'obsidian:sync',
		labelKey: 'prefs_developer_permission_obsidian_sync',
		descKey: 'prefs_developer_permission_obsidian_sync_hint'
	}
];

export const ISSUE_DEFAULTS: {
	name: string;
	permissions: PermissionKey[];
	expiry: ExpiryOption;
} = {
	name: 'Personal MacBook',
	permissions: [],
	expiry: '90'
};

const WRITE_READ_PAIRS: ReadonlyArray<readonly [PermissionKey, PermissionKey]> = [
	['library:write', 'library:read'],
	['feeds:write', 'feeds:read'],
	['integrations:write', 'integrations:read'],
	['webhooks:write', 'webhooks:read'],
	['ai:write', 'ai:read']
];

export function setsEqual<T>(a: Set<T>, b: Set<T>): boolean {
	if (a.size !== b.size) return false;
	for (const value of a) if (!b.has(value)) return false;
	return true;
}

function canonicalPermissions(current: Iterable<PermissionKey>): PermissionKey[] {
	const selected = new Set(current);
	for (const [write, read] of WRITE_READ_PAIRS) {
		if (selected.has(write)) selected.add(read);
	}
	return PERMISSION_CATALOGUE.filter((permission) => selected.has(permission));
}

function resourceGroup(key: ResourcePermissionKey): ResourcePermissionGroup {
	return RESOURCE_PERMISSION_GROUPS.find((group) => group.key === key)!;
}

export function resourceAccessLevel(
	current: Iterable<PermissionKey>,
	resource: ResourcePermissionKey
): ResourceAccessLevel {
	const selected = new Set(current);
	const group = resourceGroup(resource);
	if (selected.has(group.write)) return 'write';
	if (selected.has(group.read)) return 'read';
	return 'none';
}

export function setResourceAccess(
	current: Iterable<PermissionKey>,
	resource: ResourcePermissionKey,
	level: ResourceAccessLevel
): PermissionKey[] {
	const selected = new Set(current);
	const group = resourceGroup(resource);
	selected.delete(group.read);
	selected.delete(group.write);
	if (level === 'read' || level === 'write') selected.add(group.read);
	if (level === 'write') selected.add(group.write);
	return canonicalPermissions(selected);
}

export function nextIssuePermissions(
	current: Iterable<PermissionKey>,
	toggled: PermissionKey
): PermissionKey[] {
	const selected = new Set(current);
	if (selected.has(toggled)) {
		selected.delete(toggled);
		if (toggled.endsWith(':read')) {
			selected.delete(toggled.replace(':read', ':write') as PermissionKey);
		}
	} else {
		selected.add(toggled);
	}
	return canonicalPermissions(selected);
}

export function toggleAllPermissions(current: Iterable<PermissionKey>): PermissionKey[] {
	const selected = new Set(current);
	return PERMISSION_CATALOGUE.every((permission) => selected.has(permission))
		? []
		: [...PERMISSION_CATALOGUE];
}

export function allPermissionsSelected(current: Iterable<PermissionKey>): boolean {
	const selected = new Set(current);
	return PERMISSION_CATALOGUE.every((permission) => selected.has(permission));
}

export function tokenRequest(
	name: string,
	permissions: Iterable<PermissionKey>,
	expiry: ExpiryOption
): CreateApiTokenRequest {
	return {
		name: name.trim(),
		permissions: canonicalPermissions(permissions),
		expires_in: expiry === 'never' ? null : Number(expiry) * 24 * 60 * 60
	};
}

export function issuePresetFromSearchParams(
	searchParams: URLSearchParams
): { name: string; permissions: PermissionKey[] } | null {
	if (searchParams.get('permission') !== 'obsidian:sync') return null;
	return { name: 'Obsidian plugin', permissions: ['obsidian:sync'] };
}

// Blue reads, amber acts, indigo leaves the building. `ai:use` groups with
// write because it acts — and because --dev-accent-soft is identical to
// --dev-scope-read-bg in dark, so a dedicated tint would read as a read grant.
export function permissionClass(permission: string): string {
	if (permission === 'obsidian:sync') return 'obsidian';
	if (permission.endsWith(':read')) return 'read';
	if (permission.endsWith(':write') || permission === 'ai:use') return 'write';
	return 'other';
}

export function formatRelative(iso: string | null | undefined): string | null {
	return relativeTime(iso);
}

export function formatDate(iso: string | null | undefined): string {
	if (!iso) return '—';
	return get(date)(new Date(iso), {
		day: '2-digit',
		month: 'short',
		year: 'numeric'
	});
}

export function formatTime(iso: string): string {
	return get(time)(new Date(iso), {
		hour: '2-digit',
		minute: '2-digit',
		second: '2-digit',
		hour12: false
	});
}

export function statusClassFor(code: number | null | undefined): 's2xx' | 's4xx' | 's5xx' {
	if (code == null || code >= 500) return 's5xx';
	if (code >= 400) return 's4xx';
	return 's2xx';
}

export function lastStatusClass(endpoint: WebhookEndpoint): 'healthy' | 'failing' | 'paused' {
	if (!endpoint.is_active) return 'paused';
	return endpoint.last_status;
}

export function lastStatusLabel(endpoint: WebhookEndpoint): MessageKey {
	if (!endpoint.is_active) return 'prefs_developer_status_paused';
	if (endpoint.last_status === 'failing') return 'prefs_developer_status_failing';
	return 'prefs_developer_status_healthy';
}

export function groupCount(events: string[], selected: Set<string>): number {
	let count = 0;
	for (const event of events) if (selected.has(event)) count++;
	return count;
}

export function isGroupAllSelected(events: string[], selected: Set<string>): boolean {
	return events.length > 0 && events.every((event) => selected.has(event));
}
