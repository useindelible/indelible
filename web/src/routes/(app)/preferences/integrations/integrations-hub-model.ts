import type { IntegrationConnectionDto, ImportJobStatusResponse } from '$lib/api';
import { deriveConnectionState } from '$lib/integrations/status';
import { normalizeImportStatus } from '$lib/integrations/status';

export type ImportSlot = 'readwise';
export type StatusVariant = 'active' | 'syncing' | 'attention' | 'coming';
export type SyncState = 'idle' | 'pending' | 'success' | 'error';

export interface HubConnectionStatus {
	label: string;
	variant: StatusVariant;
	pulse?: boolean;
	check?: boolean;
}

export interface StoreLink {
	label: string;
	href: string;
}

export interface RingCounts {
	total: number;
	connected: number;
	syncing: number;
	attention: number;
}

export interface RingSegment {
	dash: number;
	offset: number;
}

export interface RingDash {
	connected: RingSegment;
	syncing: RingSegment;
	attention: RingSegment;
}

export interface SevenDayDelta {
	sign: 'up' | 'down' | 'flat';
	label: string;
}

export function formatUploadLimit(bytes: number | undefined): string {
	if (!bytes) return 'Max file size set by server';
	const megabytes = bytes / (1024 * 1024);
	const label = Number.isInteger(megabytes) ? megabytes.toString() : megabytes.toFixed(1);
	return `Max ${label} MB each`;
}

export function sourceFileLabel(job: ImportJobStatusResponse): string {
	if (job.import_source.startsWith('readwise')) return 'Readwise · files';
	return job.import_source;
}

export function statusForJob(job: ImportJobStatusResponse | null | undefined): {
	label: string;
	variant: StatusVariant;
} {
	if (!job) return { label: 'Unknown', variant: 'coming' };
	switch (normalizeImportStatus(job.status)) {
		case 'completed':
			return { label: 'Completed', variant: 'active' };
		case 'partial':
			return { label: 'Partial', variant: 'attention' };
		case 'failed':
			return { label: 'Failed', variant: 'attention' };
		case 'rolled_back':
			return { label: 'Rolled back', variant: 'coming' };
		case 'pending':
		case 'awaiting_provider':
			return { label: 'Queued', variant: 'syncing' };
		case 'running':
			return { label: 'Running', variant: 'syncing' };
		default:
			return { label: 'Unknown', variant: 'coming' };
	}
}

export function notionHubStatus(
	connection: IntegrationConnectionDto | null | undefined
): HubConnectionStatus {
	if (!connection) return { label: 'Not connected', variant: 'coming' };
	const state = deriveConnectionState(connection);
	if (state === 'syncing') return { label: 'Syncing', variant: 'syncing', pulse: true };
	if (state === 'failed') return { label: 'Needs attention', variant: 'attention' };
	return { label: 'Connected', variant: 'active', check: true };
}

export function obsidianHubStatus(
	connection: IntegrationConnectionDto | null | undefined
): HubConnectionStatus {
	if (!connection) return { label: 'Not connected', variant: 'coming' };
	return { label: 'Connected', variant: 'active', check: true };
}

export function notionDatabaseLabel(
	connection: IntegrationConnectionDto | undefined
): string | null {
	if (!connection || connection.config.provider !== 'notion') return null;
	const workspace = connection.config.workspace_name;
	const databaseId = connection.config.database_id;
	if (workspace && databaseId) return `${workspace} / ${databaseId.slice(0, 8)}…`;
	if (workspace) return workspace;
	return null;
}

export function connectionRingCounts(connections: IntegrationConnectionDto[]): RingCounts {
	let connected = 0;
	let syncing = 0;
	let attention = 0;
	for (const connection of connections) {
		const state = deriveConnectionState(connection);
		if (state === 'connected') connected += 1;
		else if (state === 'syncing') syncing += 1;
		else if (state === 'failed') attention += 1;
	}
	return { total: connections.length, connected, syncing, attention };
}

export function connectionRingDash(ringCounts: RingCounts): RingDash {
	const total = ringCounts.total === 0 ? 1 : ringCounts.total;
	const circumference = 264;
	const connectedLen = (ringCounts.connected / total) * circumference;
	const syncingLen = (ringCounts.syncing / total) * circumference;
	const attentionLen = (ringCounts.attention / total) * circumference;
	return {
		connected: { dash: connectedLen, offset: 0 },
		syncing: { dash: syncingLen, offset: -connectedLen },
		attention: { dash: attentionLen, offset: -(connectedLen + syncingLen) }
	};
}

export function sevenDayItems(history: ImportJobStatusResponse[]): number {
	const cutoff = Date.now() - 7 * 24 * 60 * 60 * 1000;
	let total = 0;
	for (const job of history) {
		const ts = new Date(job.created_at).getTime();
		if (ts < cutoff) continue;
		total += job.counts.imported + job.counts.updated;
	}
	return total;
}

export function sevenDayDelta(history: ImportJobStatusResponse[]): SevenDayDelta | null {
	const current = sevenDayItems(history);
	const now = Date.now();
	const week = 7 * 24 * 60 * 60 * 1000;
	const currentCutoff = now - week;
	const priorCutoff = now - 2 * week;
	let priorTotal = 0;
	for (const job of history) {
		const ts = new Date(job.created_at).getTime();
		if (ts < priorCutoff || ts >= currentCutoff) continue;
		priorTotal += job.counts.imported + job.counts.updated;
	}
	if (priorTotal === 0) return current > 0 ? { sign: 'up', label: 'new' } : null;
	const pct = Math.round(((current - priorTotal) / priorTotal) * 100);
	if (pct === 0) return { sign: 'flat', label: '0%' };
	return { sign: pct > 0 ? 'up' : 'down', label: `${Math.abs(pct)}%` };
}

export function relativeTime(iso: string): string {
	const timestamp = new Date(iso).getTime();
	if (Number.isNaN(timestamp)) return iso;
	const diff = Date.now() - timestamp;
	if (diff < 60_000) return 'just now';
	const minutes = Math.floor(diff / 60_000);
	if (minutes < 60) return minutes === 1 ? '1 min ago' : `${minutes} min ago`;
	const hours = Math.floor(minutes / 60);
	if (hours < 24) return hours === 1 ? '1 hr ago' : `${hours} hr ago`;
	const days = Math.floor(hours / 24);
	if (days < 30) return days === 1 ? '1 day ago' : `${days} days ago`;
	const months = Math.floor(days / 30);
	return months === 1 ? '1 month ago' : `${months} months ago`;
}

export function progressPercent(job: ImportJobStatusResponse): number | null {
	const status = normalizeImportStatus(job.status);
	if (status === 'completed' || status === 'partial') return 100;
	return null;
}

export function browserStoreLink(userAgent: string | undefined): StoreLink {
	if (!userAgent) return { label: 'Get for Chrome', href: 'https://chromewebstore.google.com/' };
	if (/Firefox\//.test(userAgent))
		return { label: 'Get for Firefox', href: 'https://addons.mozilla.org/' };
	if (/Edg\//.test(userAgent)) {
		return { label: 'Get for Edge', href: 'https://microsoftedge.microsoft.com/addons/' };
	}
	if (/OPR\//.test(userAgent)) return { label: 'Get for Opera', href: 'https://addons.opera.com/' };
	if (/Safari\//.test(userAgent) && !/Chrome\//.test(userAgent) && !/Chromium\//.test(userAgent)) {
		return { label: 'Get for Safari', href: 'https://apps.apple.com/' };
	}
	return { label: 'Get for Chrome', href: 'https://chromewebstore.google.com/' };
}
