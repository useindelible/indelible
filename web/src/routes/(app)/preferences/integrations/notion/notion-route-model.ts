import type { IntegrationConnectionDto } from '$lib/api';
import type { MessageKey, Translate } from '$lib/i18n';
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

export function getNotionDatabaseLabel(workspaceName: string | null, translate: Translate): string {
	return workspaceName
		? `${workspaceName} · Indelible`
		: translate('integrations_notion_default_database_name');
}

export function getNotionHeroStatus(connectionState: ConnectionState): MessageKey {
	switch (connectionState) {
		case 'failed':
			return 'integrations_notion_attention';
		case 'syncing':
			return 'integrations_notion_syncing';
		case 'disconnected':
			return 'integrations_hub_status_not_connected';
		case 'unavailable':
			return 'integrations_notion_status_unavailable';
		default:
			return 'integrations_hub_status_connected';
	}
}

export function formatNotionHeroLastSync(
	lastSyncAt: string | null | undefined,
	translate: Translate,
	locale: string | null | undefined,
	now = Date.now()
): string {
	if (!lastSyncAt) return translate('integrations_notion_never_synced');
	const parsed = new Date(lastSyncAt);
	if (Number.isNaN(parsed.getTime())) return translate('integrations_notion_never_synced');
	const activeLocale = locale ?? 'en';
	const relativeFormatter = new Intl.RelativeTimeFormat(activeLocale, { numeric: 'auto' });
	const dateFormatter = new Intl.DateTimeFormat(activeLocale, {
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
