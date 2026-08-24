import type { IntegrationConnectionDto, ImportJobStatusResponse } from '$lib/api';
import { deriveConnectionState } from '$lib/integrations/status';
import { formatMegabytes } from '$lib/format/megabytes';
import { normalizeImportStatus } from '$lib/integrations/status';
import type { MessageKey, Translate } from '$lib/i18n';

export type ImportSlot = 'readwise';
export type StatusVariant = 'active' | 'syncing' | 'attention' | 'coming';
export type SyncState = 'idle' | 'pending' | 'success' | 'error';

export interface HubConnectionStatus {
	labelKey: MessageKey;
	variant: StatusVariant;
	pulse?: boolean;
	check?: boolean;
}

export interface StoreLink {
	labelKey: MessageKey;
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

export function formatUploadLimit(bytes: number | undefined, translate: Translate): string {
	if (!bytes) return translate('integrations_hub_max_file_server');
	return translate('integrations_hub_max_file_each', { values: { size: formatMegabytes(bytes) } });
}

export function sourceFileLabel(job: ImportJobStatusResponse, translate: Translate): string {
	if (job.import_source.startsWith('readwise')) return translate('integrations_hub_readwise_files');
	return job.import_source;
}

export function statusForJob(job: ImportJobStatusResponse | null | undefined): {
	labelKey: MessageKey;
	variant: StatusVariant;
} {
	if (!job) return { labelKey: 'imports_status_unknown', variant: 'coming' };
	switch (normalizeImportStatus(job.status)) {
		case 'completed':
			return { labelKey: 'imports_status_completed', variant: 'active' };
		case 'partial':
			return { labelKey: 'imports_status_partial', variant: 'attention' };
		case 'failed':
			return { labelKey: 'imports_status_failed', variant: 'attention' };
		case 'rolled_back':
			return { labelKey: 'imports_status_rolled_back', variant: 'coming' };
		case 'pending':
		case 'awaiting_provider':
			return { labelKey: 'imports_status_queued', variant: 'syncing' };
		case 'running':
			return { labelKey: 'imports_status_running', variant: 'syncing' };
		default:
			return { labelKey: 'imports_status_unknown', variant: 'coming' };
	}
}

/// Availability as reported by the integrations list. A server that predates
/// the field reports nothing — fail open so the authorize endpoint's own
/// error remains the backstop.
export function isOauthProviderAvailable(
	available: string[] | null | undefined,
	providerId: string
): boolean {
	if (available == null) return true;
	return available.includes(providerId);
}

export function notionHubStatus(
	connection: IntegrationConnectionDto | null | undefined
): HubConnectionStatus {
	if (!connection) return { labelKey: 'integrations_hub_status_not_connected', variant: 'coming' };
	const state = deriveConnectionState(connection);
	if (state === 'syncing')
		return { labelKey: 'integrations_hub_status_syncing', variant: 'syncing', pulse: true };
	if (state === 'failed')
		return { labelKey: 'integrations_hub_status_needs_attention', variant: 'attention' };
	return { labelKey: 'integrations_hub_status_connected', variant: 'active', check: true };
}

export function obsidianHubStatus(
	connection: IntegrationConnectionDto | null | undefined
): HubConnectionStatus {
	if (!connection) return { labelKey: 'integrations_hub_status_not_connected', variant: 'coming' };
	return { labelKey: 'integrations_hub_status_connected', variant: 'active', check: true };
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

export function progressPercent(job: ImportJobStatusResponse): number | null {
	const status = normalizeImportStatus(job.status);
	if (status === 'completed' || status === 'partial') return 100;
	return null;
}

export function browserStoreLink(userAgent: string | undefined): StoreLink {
	if (!userAgent)
		return { labelKey: 'integrations_hub_get_chrome', href: 'https://chromewebstore.google.com/' };
	if (/Firefox\//.test(userAgent))
		return { labelKey: 'integrations_hub_get_firefox', href: 'https://addons.mozilla.org/' };
	if (/Edg\//.test(userAgent)) {
		return {
			labelKey: 'integrations_hub_get_edge',
			href: 'https://microsoftedge.microsoft.com/addons/'
		};
	}
	if (/OPR\//.test(userAgent))
		return { labelKey: 'integrations_hub_get_opera', href: 'https://addons.opera.com/' };
	if (/Safari\//.test(userAgent) && !/Chrome\//.test(userAgent) && !/Chromium\//.test(userAgent)) {
		return { labelKey: 'integrations_hub_get_safari', href: 'https://apps.apple.com/' };
	}
	return { labelKey: 'integrations_hub_get_chrome', href: 'https://chromewebstore.google.com/' };
}
