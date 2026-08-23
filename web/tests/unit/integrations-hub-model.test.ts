import { describe, expect, it, vi } from 'vitest';
import type { ImportJobStatusResponse } from '$lib/api';
import type { Translate } from '$lib/i18n';
import en from '$lib/i18n/locales/en.json';
import {
	formatUploadLimit,
	progressPercent,
	sevenDayDelta,
	sevenDayItems,
	sourceFileLabel,
	statusForJob,
	isOauthProviderAvailable
} from '../../src/routes/(app)/preferences/integrations/integrations-hub-model';

const translate: Translate = (key, options) =>
	en[key].replace('{size}', String(options?.values?.size ?? ''));

function job(overrides: Partial<ImportJobStatusResponse> = {}): ImportJobStatusResponse {
	return {
		id: 'imp_1',
		import_source: 'readwise_reader',
		import_method: 'upload',
		status: 'completed',
		created_at: '2026-06-10T12:00:00Z',
		started_at: '2026-06-10T12:00:00Z',
		finished_at: '2026-06-10T12:05:00Z',
		error: null,
		item_outcomes: [],
		readwise_report: null,
		counts: {
			imported: 10,
			updated: 2,
			duplicate: 3,
			failed: 0,
			skipped_private: 0
		},
		...overrides
	};
}

describe('integrations hub model', () => {
	it('formats upload limits', () => {
		expect(formatUploadLimit(undefined, translate)).toBe('Max file size set by server');
		expect(formatUploadLimit(10 * 1024 * 1024, translate)).toBe('Max 10 MB each');
		expect(formatUploadLimit(1.5 * 1024 * 1024, translate)).toBe('Max 1.5 MB each');
	});

	it('maps import job status and progress display state', () => {
		expect(statusForJob(job()).labelKey).toBe('imports_status_completed');
		expect(statusForJob(job({ status: 'running' })).variant).toBe('syncing');
		expect(statusForJob(job({ status: 'rolled_back' })).labelKey).toBe(
			'imports_status_rolled_back'
		);
		expect(progressPercent(job())).toBe(100);
		expect(progressPercent(job({ status: 'running' }))).toBeNull();
	});

	it('summarizes recent import activity and source labels', () => {
		vi.setSystemTime(new Date('2026-06-10T14:00:00Z'));
		const jobs = [
			job(),
			job({
				id: 'imp_2',
				created_at: '2026-06-03T15:00:00Z',
				counts: { ...job().counts, imported: 4, updated: 0 }
			}),
			job({
				id: 'imp_3',
				created_at: '2026-06-01T12:00:00Z',
				counts: { ...job().counts, imported: 8, updated: 0 }
			})
		];
		expect(sevenDayItems(jobs)).toBe(16);
		expect(sevenDayDelta(jobs)).toEqual({ sign: 'up', label: '100%' });
		expect(sourceFileLabel(job(), translate)).toBe('Readwise · files');
		expect(sourceFileLabel(job({ import_source: 'other_source' }), translate)).toBe('other_source');
		vi.useRealTimers();
	});
});

describe('isOauthProviderAvailable', () => {
	it('fails open when the server does not report availability', () => {
		expect(isOauthProviderAvailable(undefined, 'notion')).toBe(true);
		expect(isOauthProviderAvailable(null, 'notion')).toBe(true);
	});

	it('reads the reported provider list', () => {
		expect(isOauthProviderAvailable([], 'notion')).toBe(false);
		expect(isOauthProviderAvailable(['notion'], 'notion')).toBe(true);
	});
});
