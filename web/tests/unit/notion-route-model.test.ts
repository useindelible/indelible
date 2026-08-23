import { describe, expect, it } from 'vitest';
import { get } from 'svelte/store';

import type { IntegrationConnectionDto } from '$lib/api';
import { locale, setupI18nSync, t } from '$lib/i18n';
import en from '$lib/i18n/locales/en.json';
import fr from '$lib/i18n/locales/fr.json';
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
		setupI18nSync({ en, fr }, 'fr');
		expect(getNotionWorkspaceName(connection())).toBe('Research Desk');
		expect(getNotionWorkspaceIcon(connection())).toBe('📚');
		expect(getNotionDatabaseLabel('Research Desk', get(t))).toBe('Research Desk · Indelible');
		expect(getNotionDatabaseLabel(null, get(t))).toBe('Bibliothèque Indelible');
		expect(getNotionWorkspaceName(undefined)).toBeNull();
	});

	it('formats hero status and last sync labels', () => {
		setupI18nSync({ en, fr }, 'fr');
		const now = new Date('2026-05-03T12:00:00Z').getTime();
		expect(formatNotionHeroLastSync(null, get(t), get(locale), now)).toBe('Jamais synchronisé');
		expect(formatNotionHeroLastSync('bad-date', get(t), get(locale), now)).toBe(
			'Jamais synchronisé'
		);
		expect(formatNotionHeroLastSync('2026-05-03T11:30:00Z', get(t), get(locale), now)).toContain(
			'il y a 30'
		);
		expect(get(t)(getNotionHeroStatus('failed'))).toBe('Attention');
		expect(get(t)(getNotionHeroStatus('syncing'))).toBe('Synchronisation');
		expect(get(t)(getNotionHeroStatus('connected'))).toBe('Connecté');
	});
});
