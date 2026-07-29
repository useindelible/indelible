import { describe, expect, it } from 'vitest';
import type { IntegrationConnectionDto, NotionExportItemDto } from '$lib/api';
import {
	formatExportedAt,
	formatItemType,
	notionConnectionDetails,
	notionExportItemsMeta,
	notionStatusSummary,
	selectedExportItemCount
} from '../../src/lib/components/integrations/notion-status/notion-status-model';

function connection(overrides: Partial<IntegrationConnectionDto> = {}): IntegrationConnectionDto {
	return {
		id: 'int_notion',
		provider: 'notion',
		status: 'active',
		last_sync_at: '2026-04-25T12:00:00Z',
		last_error: null,
		config: {
			provider: 'notion',
			workspace_id: 'wsp_1',
			workspace_name: 'Research HQ',
			database_id: 'db_1',
			data_source_id: 'ds_1',
			workspace_icon: 'N',
			export_automatically: true,
			include_highlight_locations: true,
			compact_layout: true,
			selection_enabled: false
		},
		pending_jobs: 2,
		created_at: '2026-04-17T12:00:00Z',
		...overrides
	} as IntegrationConnectionDto;
}

function item(overrides: Partial<NotionExportItemDto> = {}): NotionExportItemDto {
	return {
		library_entry_id: 'lib_1',
		title: 'Exported essay',
		item_type: 'article',
		url: 'https://example.com',
		selected: true,
		exported_page_id: 'page_1',
		last_synced_at: '2026-04-25T12:00:00Z',
		last_error: null,
		...overrides
	};
}

describe('notion status model', () => {
	it('derives connection details from notion provider config', () => {
		expect(notionConnectionDetails(connection())).toEqual({
			workspaceName: 'Research HQ',
			workspaceIcon: 'N',
			databaseId: 'db_1',
			dataSourceId: 'ds_1'
		});
		expect(notionConnectionDetails(undefined).workspaceName).toBeNull();
	});

	it('derives status summary tones from connection errors', () => {
		expect(notionStatusSummary(connection()).statusLabel).toBe('Healthy');
		expect(
			notionStatusSummary(connection({ last_error: '429 Too Many Requests' })).statusTone
		).toBe('warning');
		expect(notionStatusSummary(connection({ last_error: '401 Unauthorized' })).statusLabel).toBe(
			'Authorization needed'
		);
		expect(notionStatusSummary(connection({ last_error: 'schema mismatch' })).statusLabel).toBe(
			'Schema attention'
		);
	});

	it('formats item table metadata', () => {
		expect(
			selectedExportItemCount([item(), item({ library_entry_id: 'lib_2', selected: false })])
		).toBe(1);
		expect(notionExportItemsMeta(10, 20, 0, '')).toBe('10 of 20 documents');
		expect(notionExportItemsMeta(3, 20, 4, ' essay ')).toBe('3 of 4 matching');
		expect(formatExportedAt(null)).toBe('Not exported');
		expect(formatItemType('pdf_document')).toBe('pdf document');
	});
});
