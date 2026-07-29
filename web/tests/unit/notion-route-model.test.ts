import { describe, expect, it } from 'vitest';

import type { IntegrationConnectionDto } from '$lib/api';
import {
	formatNotionHeroLastSync,
	getNotionDatabaseLabel,
	getNotionHeroStatus,
	getNotionWorkspaceIcon,
	getNotionWorkspaceName
} from '../../src/routes/(app)/preferences/integrations/notion/notion-route-model';

function connection(overrides: Partial<IntegrationConnectionDto> = {}): IntegrationConnectionDto {
	return {
		id: 'int_1',
		provider: 'notion',
		status: 'active',
		last_sync_at: '2026-05-03T11:30:00Z',
		last_error: null,
		config: {
			provider: 'notion',
			workspace_id: 'workspace_1',
			workspace_name: 'Research Desk',
			workspace_icon: '📚',
			database_id: 'db_1',
			data_source_id: null,
			export_automatically: true,
			include_highlight_locations: true,
			compact_layout: true,
			selection_enabled: false
		},
		pending_jobs: 0,
		created_at: '2026-05-01T00:00:00Z',
		...overrides
	} as IntegrationConnectionDto;
}

describe('notion route model', () => {
	it('derives workspace and database labels from Notion connections', () => {
		expect(getNotionWorkspaceName(connection())).toBe('Research Desk');
		expect(getNotionWorkspaceIcon(connection())).toBe('📚');
		expect(getNotionDatabaseLabel('Research Desk')).toBe('Research Desk · Indelible');
		expect(getNotionDatabaseLabel(null)).toBe('Indelible Library');
		expect(getNotionWorkspaceName(undefined)).toBeNull();
	});

	it('formats hero status and last sync labels', () => {
		const now = new Date('2026-05-03T12:00:00Z').getTime();
		expect(formatNotionHeroLastSync(null, now)).toBe('Never');
		expect(formatNotionHeroLastSync('bad-date', now)).toBe('Never');
		expect(formatNotionHeroLastSync('2026-05-03T11:30:00Z', now)).toContain('30');
		expect(getNotionHeroStatus('failed')).toBe('Attention');
		expect(getNotionHeroStatus('syncing')).toBe('Syncing');
		expect(getNotionHeroStatus('connected')).toBe('Connected');
	});
});
