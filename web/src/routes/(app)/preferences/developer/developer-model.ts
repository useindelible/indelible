import type { WebhookDelivery, WebhookEndpoint } from '$lib/api/webhooks';

export type ScopeKey = 'read' | 'write' | 'cli' | 'extension' | 'obsidian_plugin' | 'admin';
export type ExpiryOption = 'never' | '30' | '90' | '365';

export interface ScopeDef {
	key: ScopeKey;
	desc: string;
}

export interface TerminalLine {
	ts: string;
	method: 'GET' | 'POST' | 'DEL' | 'HOOK';
	methodClass: 'get' | 'post' | 'delete' | 'hook';
	path: string;
	target?: string;
	status: string;
	statusClass: 's2xx' | 's4xx' | 's5xx';
}

export interface WebhookEventGroup {
	key: string;
	name: string;
	events: string[];
}

export const SCOPE_DEFS: ScopeDef[] = [
	{ key: 'read', desc: 'List items, fetch highlights, read collections.' },
	{ key: 'write', desc: 'Save items, create highlights, edit collections.' },
	{ key: 'cli', desc: 'Long-lived access for the ind CLI.' },
	{ key: 'extension', desc: 'Browser save-page from the extension.' },
	{ key: 'obsidian_plugin', desc: 'Highlight + note sync to an Obsidian vault.' },
	{ key: 'admin', desc: 'Account-level operations. Use sparingly.' }
];

export const TERMINAL_LINES: TerminalLine[] = [
	{
		ts: '12:42:18',
		method: 'POST',
		methodClass: 'post',
		path: '/v1/items',
		status: '200 · 142ms',
		statusClass: 's2xx'
	},
	{
		ts: '12:41:55',
		method: 'HOOK',
		methodClass: 'hook',
		path: 'library_entry.saved',
		target: 'n8n',
		status: '200',
		statusClass: 's2xx'
	},
	{
		ts: '12:39:11',
		method: 'POST',
		methodClass: 'post',
		path: '/v1/highlights',
		status: '200 · 88ms',
		statusClass: 's2xx'
	},
	{
		ts: '12:36:02',
		method: 'GET',
		methodClass: 'get',
		path: '/v1/items?since=…',
		status: '200 · 36ms',
		statusClass: 's2xx'
	},
	{
		ts: '12:35:48',
		method: 'HOOK',
		methodClass: 'hook',
		path: 'library_entry.archived',
		target: 'obsidian',
		status: '200',
		statusClass: 's2xx'
	},
	{
		ts: '12:33:09',
		method: 'HOOK',
		methodClass: 'hook',
		path: 'feed.poll_failed',
		target: 'monitoring',
		status: '503 · retry',
		statusClass: 's5xx'
	},
	{
		ts: '12:31:44',
		method: 'GET',
		methodClass: 'get',
		path: '/v1/documents/doc_8c7Z…',
		status: '200',
		statusClass: 's2xx'
	}
];

export function setsEqual<T>(a: Set<T>, b: Set<T>): boolean {
	if (a.size !== b.size) return false;
	for (const value of a) if (!b.has(value)) return false;
	return true;
}

export function scopeClass(scope: string): string {
	if (scope === 'read' || scope === 'write' || scope === 'admin' || scope === 'cli') return scope;
	if (scope === 'extension') return 'ext';
	if (scope === 'obsidian_plugin') return 'obsidian';
	return 'cli';
}

export function formatRelative(iso: string | null | undefined): string {
	if (!iso) return 'Never';
	const ms = Date.now() - new Date(iso).getTime();
	const seconds = Math.floor(ms / 1000);
	if (seconds < 60) return `${seconds}s ago`;
	const minutes = Math.floor(seconds / 60);
	if (minutes < 60) return `${minutes} minute${minutes === 1 ? '' : 's'} ago`;
	const hours = Math.floor(minutes / 60);
	if (hours < 24) return `${hours} hour${hours === 1 ? '' : 's'} ago`;
	const days = Math.floor(hours / 24);
	return `${days} day${days === 1 ? '' : 's'} ago`;
}

export function formatDate(iso: string | null | undefined): string {
	if (!iso) return '—';
	return new Date(iso).toLocaleDateString(undefined, {
		day: '2-digit',
		month: 'short',
		year: 'numeric'
	});
}

export function formatTime(iso: string): string {
	return new Date(iso).toLocaleTimeString(undefined, {
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

export function lastStatusLabel(endpoint: WebhookEndpoint): string {
	if (!endpoint.is_active) return 'Paused';
	if (endpoint.last_status === 'failing') return 'Failing';
	return 'Healthy';
}

export function countRecentDeliveries(deliveries: WebhookDelivery[]): number {
	return deliveries.filter(
		(delivery) => Date.now() - new Date(delivery.delivered_at).getTime() < 86_400_000
	).length;
}

export function deliveryRatePercent(deliveries: WebhookDelivery[]): string {
	if (deliveries.length === 0) return '100.0';
	const successes = deliveries.filter(
		(delivery) =>
			typeof delivery.status_code === 'number' &&
			delivery.status_code >= 200 &&
			delivery.status_code < 300
	).length;
	return ((successes / deliveries.length) * 100).toFixed(1);
}

export function groupCount(events: string[], selected: Set<string>): number {
	let count = 0;
	for (const event of events) if (selected.has(event)) count++;
	return count;
}

export function isGroupAllSelected(events: string[], selected: Set<string>): boolean {
	return events.length > 0 && events.every((event) => selected.has(event));
}
