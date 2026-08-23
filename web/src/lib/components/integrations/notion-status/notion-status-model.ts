import type { IntegrationConnectionDto, NotionExportItemDto } from '$lib/api';
import { date, t, type Translate } from '$lib/i18n';
import {
	deriveConnectionState,
	detectAuthFailure,
	detectRateLimit
} from '$lib/integrations/status';
import { get } from 'svelte/store';

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
	connection: IntegrationConnectionDto | undefined,
	translate: Translate = get(t)
): NotionStatusSummary {
	const connectionState = deriveConnectionState(connection);
	const isAuthFailure = detectAuthFailure(connection?.last_error);
	const isRateLimited = detectRateLimit(connection?.last_error);
	const isSchemaError = Boolean(connection?.last_error?.toLowerCase().includes('schema'));

	return {
		connectionState,
		formattedLastSync: formatDateTime(
			connection?.last_sync_at,
			translate('integrations_notion_never_synced')
		),
		formattedConnectedOn: formatDateOnly(connection?.created_at),
		isAuthFailure,
		isRateLimited,
		isSchemaError,
		pendingJobs: connection?.pending_jobs ?? 0,
		statusLabel: statusLabel(
			connectionState,
			isAuthFailure,
			isSchemaError,
			isRateLimited,
			translate
		),
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
	query: string,
	translate: Translate = get(t)
): string {
	if (query.trim()) {
		return translate('integrations_notion_items_matching', {
			values: { count: itemCount, total: filteredCount }
		});
	}
	return translate('integrations_notion_items_documents', { values: { count: itemCount, total } });
}

export function formatExportedAt(value: string | null | undefined): string {
	return formatDateTime(value, get(t)('integrations_notion_not_exported'));
}

export function formatItemType(value: string, translate: Translate = get(t)): string {
	const keys: Record<string, Parameters<Translate>[0]> = {
		article: 'library_filter_value_article',
		book: 'library_filter_value_book',
		email: 'library_filter_value_email',
		pdf: 'library_filter_value_pdf',
		tweet: 'library_filter_value_tweet',
		video: 'library_filter_value_video'
	};
	const key = keys[value];
	return key ? translate(key) : value.replace(/_/g, ' ');
}

function formatDateTime(value: string | null | undefined, fallback: string): string {
	if (!value) return fallback;
	const parsed = new Date(value);
	if (Number.isNaN(parsed.getTime())) return fallback;
	return get(date)(parsed, { dateStyle: 'medium', timeStyle: 'short' });
}

function formatDateOnly(value: string | null | undefined): string | null {
	if (!value) return null;
	const parsed = new Date(value);
	if (Number.isNaN(parsed.getTime())) return null;
	return get(date)(parsed, { year: 'numeric', month: 'long', day: 'numeric' });
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
	isRateLimited: boolean,
	translate: Translate
): string {
	if (isAuthFailure) return translate('integrations_notion_authorization_needed');
	if (isSchemaError) return translate('integrations_notion_schema_attention');
	if (connectionState === 'failed') return translate('integrations_notion_attention');
	if (connectionState === 'syncing') return translate('integrations_notion_syncing');
	if (isRateLimited) return translate('integrations_notion_rate_limited');
	return translate('integrations_notion_healthy');
}
