import { describe, expect, it, vi } from 'vitest';
import type {
	IntegrationConnectionDto,
	ObsidianPreviewResponse,
	ObsidianSettingsDto
} from '$lib/api';
import {
	buildObsidianSaveBody,
	formatObsidianLastSync,
	obsidianHeroState,
	obsidianHeroStatusLabel,
	previewBody,
	previewFilePath,
	previewMissingSummary,
	serializeForCompare,
	snapshotObsidianSettings
} from '../../src/routes/(app)/preferences/integrations/obsidian/obsidian-model';

function settings(overrides: Partial<ObsidianSettingsDto> = {}): ObsidianSettingsDto {
	return {
		category_folder_templates: {
			articles: 'articles',
			books: 'books',
			podcasts: 'podcasts',
			tweets: 'tweets'
		},
		export_all_reader_documents: false,
		file_name_template: '{{title}}',
		group_files_in_category_folders: true,
		highlight_header_template: 'Highlights',
		highlight_template: '- {{highlight_text}}',
		metadata_template: 'URL: {{url}}',
		page_title_template: '# {{title}}',
		properties_template: '',
		sync_notification_template: '- synced {{document_count}}',
		sync_notifications: true,
		...overrides
	};
}

function connection(overrides: Partial<IntegrationConnectionDto> = {}): IntegrationConnectionDto {
	return {
		id: 'conn_1',
		provider: 'obsidian',
		display_name: 'Obsidian',
		status: 'active',
		created_at: '2026-06-01T00:00:00Z',
		updated_at: '2026-06-01T00:00:00Z',
		last_sync_at: null,
		last_error: null,
		pending_jobs: 0,
		...overrides
	};
}

function preview(overrides: Partial<ObsidianPreviewResponse> = {}): ObsidianPreviewResponse {
	return {
		file_path: 'Indelible/articles/Sample.md',
		full_content: '# Sample\n\nBody',
		full_document_text: 'Full text body',
		full_document_text_path: 'Indelible/articles/Sample Full Text.md',
		...overrides
	};
}

describe('obsidian settings model', () => {
	it('snapshots nested folder templates and normalizes compare values', () => {
		const original = settings({ properties_template: null, file_name_template: null });
		const snap = snapshotObsidianSettings(original);
		snap.category_folder_templates.books = 'changed';

		expect(original.category_folder_templates.books).toBe('books');
		expect(serializeForCompare(original).properties_template).toBe('');
		expect(serializeForCompare(original).file_name_template).toBe('');
	});

	it('builds save bodies with blank optional templates converted to null', () => {
		const body = buildObsidianSaveBody(
			settings({
				properties_template: '   ',
				file_name_template: ''
			})
		);

		expect(body.properties_template).toBeNull();
		expect(body.file_name_template).toBeNull();
		expect(body.category_folder_templates.books).toBe('books');
		expect(body.category_folder_templates.podcasts).toBe('podcasts');
	});

	it('derives hero state and last-sync labels', () => {
		vi.setSystemTime(new Date('2026-06-10T14:00:00Z'));
		expect(obsidianHeroState(undefined, 'unavailable')).toBe('disconnected');
		expect(obsidianHeroState(connection(), 'syncing')).toBe('syncing');
		expect(obsidianHeroState(connection(), 'failed')).toBe('error');
		expect(obsidianHeroStatusLabel('error')).toBe('integrations_obsidian_last_sync_failed');
		expect(formatObsidianLastSync(connection({ last_sync_at: '2026-06-10T13:40:00Z' }))).toBe(
			'20 minutes ago'
		);
		vi.useRealTimers();
	});

	it('selects preview paths and detects missing note summaries', () => {
		const sample = preview({ full_content: '# Sample\n\nNo summary here' });

		expect(previewFilePath(sample, 'note')).toBe('Indelible/articles/Sample.md');
		expect(previewFilePath(sample, 'full')).toBe('Indelible/articles/Sample Full Text.md');
		expect(previewBody(sample, 'full')).toBe('Full text body');
		expect(previewMissingSummary(sample, 'note')).toBe(true);
		expect(previewMissingSummary(sample, 'full')).toBe(false);
	});
});
