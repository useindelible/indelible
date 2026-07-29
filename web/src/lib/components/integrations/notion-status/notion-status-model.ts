import type { IntegrationConnectionDto, NotionExportItemDto } from '$lib/api';
import {
	deriveConnectionState,
	detectAuthFailure,
	detectRateLimit
} from '$lib/integrations/status';

export type NotionStatusTone = 'success' | 'warning' | 'error' | 'info';

export interface NotionConnectionDetails {
	workspaceName: string | null;
	workspaceIcon: string | null;
	databaseId: string | null;
	dataSourceId: string | null;
}

export interface NotionStatusSummary {
	connectionState: ReturnType<typeof deriveConnectionState>;
	formattedLastSync: string;
	formattedConnectedOn: string | null;
	isAuthFailure: boolean;
	isRateLimited: boolean;
	isSchemaError: boolean;
	pendingJobs: number;
	statusLabel: string;
	statusTone: NotionStatusTone;
}

const dateTimeFormatter = new Intl.DateTimeFormat(undefined, {
	dateStyle: 'medium',
	timeStyle: 'short'
});

const dateOnlyFormatter = new Intl.DateTimeFormat(undefined, {
	year: 'numeric',
	month: 'long',
	day: 'numeric'
});

export function notionConnectionDetails(
	connection: IntegrationConnectionDto | undefined
): NotionConnectionDetails {
	if (!connection || connection.config.provider !== 'notion') {
		return {
			workspaceName: null,
			workspaceIcon: null,
			databaseId: null,
			dataSourceId: null
		};
	}

	return {
		workspaceName: connection.config.workspace_name ?? null,
		workspaceIcon: connection.config.workspace_icon ?? null,
		databaseId: connection.config.database_id ?? null,
		dataSourceId: connection.config.data_source_id ?? null
	};
}

export function notionStatusSummary(
	connection: IntegrationConnectionDto | undefined
): NotionStatusSummary {
	const connectionState = deriveConnectionState(connection);
	const isAuthFailure = detectAuthFailure(connection?.last_error);
	const isRateLimited = detectRateLimit(connection?.last_error);
	const isSchemaError = Boolean(connection?.last_error?.toLowerCase().includes('schema'));

	return {
		connectionState,
		formattedLastSync: formatDateTime(connection?.last_sync_at, 'Never synced'),
		formattedConnectedOn: formatDateOnly(connection?.created_at),
		isAuthFailure,
		isRateLimited,
		isSchemaError,
		pendingJobs: connection?.pending_jobs ?? 0,
		statusLabel: statusLabel(connectionState, isAuthFailure, isSchemaError, isRateLimited),
		statusTone: statusTone(connectionState, isAuthFailure, isSchemaError, isRateLimited)
	};
}

export function selectedExportItemCount(items: NotionExportItemDto[]): number {
	return items.filter((item) => item.selected).length;
}

export function notionExportItemsMeta(
	itemCount: number,
	total: number,
	filteredCount: number,
	query: string
): string {
	if (query.trim()) return `${itemCount} of ${filteredCount} matching`;
	return `${itemCount} of ${total} documents`;
}

export function formatExportedAt(value: string | null | undefined): string {
	return formatDateTime(value, 'Not exported');
}

export function formatItemType(value: string): string {
	return value.replace(/_/g, ' ');
}

function formatDateTime(value: string | null | undefined, fallback: string): string {
	if (!value) return fallback;
	const parsed = new Date(value);
	if (Number.isNaN(parsed.getTime())) return fallback;
	return dateTimeFormatter.format(parsed);
}

function formatDateOnly(value: string | null | undefined): string | null {
	if (!value) return null;
	const parsed = new Date(value);
	if (Number.isNaN(parsed.getTime())) return null;
	return dateOnlyFormatter.format(parsed);
}

function statusTone(
	connectionState: ReturnType<typeof deriveConnectionState>,
	isAuthFailure: boolean,
	isSchemaError: boolean,
	isRateLimited: boolean
): NotionStatusTone {
	if (isAuthFailure || isSchemaError || connectionState === 'failed') return 'error';
	if (isRateLimited) return 'warning';
	if (connectionState === 'syncing') return 'info';
	return 'success';
}

function statusLabel(
	connectionState: ReturnType<typeof deriveConnectionState>,
	isAuthFailure: boolean,
	isSchemaError: boolean,
	isRateLimited: boolean
): string {
	if (isAuthFailure) return 'Authorization needed';
	if (isSchemaError) return 'Schema attention';
	if (connectionState === 'failed') return 'Attention';
	if (connectionState === 'syncing') return 'Syncing';
	if (isRateLimited) return 'Rate limited';
	return 'Healthy';
}
