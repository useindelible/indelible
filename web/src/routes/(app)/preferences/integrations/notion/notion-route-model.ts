import type { IntegrationConnectionDto } from '$lib/api';
import type { ConnectionState } from '$lib/integrations/status';

export function getNotionWorkspaceName(
	connection: IntegrationConnectionDto | undefined
): string | null {
	if (!connection || connection.config.provider !== 'notion') return null;
	return connection.config.workspace_name ?? null;
}

export function getNotionWorkspaceIcon(
	connection: IntegrationConnectionDto | undefined
): string | null {
	if (!connection || connection.config.provider !== 'notion') return null;
	return connection.config.workspace_icon ?? null;
}

export function getNotionDatabaseLabel(workspaceName: string | null): string {
	return workspaceName ? `${workspaceName} · Indelible` : 'Indelible Library';
}

export function getNotionHeroStatus(connectionState: ConnectionState): string {
	switch (connectionState) {
		case 'failed':
			return 'Attention';
		case 'syncing':
			return 'Syncing';
		case 'disconnected':
			return 'Disconnected';
		case 'unavailable':
			return 'Unavailable';
		default:
			return 'Connected';
	}
}

export function formatNotionHeroLastSync(
	lastSyncAt: string | null | undefined,
	now = Date.now(),
	locale?: string | string[]
): string {
	if (!lastSyncAt) return 'Never';
	const parsed = new Date(lastSyncAt);
	if (Number.isNaN(parsed.getTime())) return 'Never';
	const relativeFormatter = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' });
	const dateFormatter = new Intl.DateTimeFormat(locale, {
		month: 'short',
		day: 'numeric'
	});
	const diffMs = parsed.getTime() - now;
	const diffMin = Math.round(diffMs / 60000);
	if (Math.abs(diffMin) < 60) return relativeFormatter.format(diffMin, 'minute');
	const diffHour = Math.round(diffMs / 3600000);
	if (Math.abs(diffHour) < 24) return relativeFormatter.format(diffHour, 'hour');
	return dateFormatter.format(parsed);
}
