import { describe, expect, it, vi } from 'vitest';
import type { ImportJobStatusResponse } from '$lib/api';
import {
	formatUploadLimit,
	progressPercent,
	relativeTime,
	sevenDayDelta,
	sevenDayItems,
	sourceFileLabel,
	statusForJob
} from '../../src/routes/(app)/preferences/integrations/integrations-hub-model';

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
	it('formats upload limits and relative time labels', () => {
		vi.setSystemTime(new Date('2026-06-10T14:00:00Z'));
		expect(formatUploadLimit(undefined)).toBe('Max file size set by server');
		expect(formatUploadLimit(10 * 1024 * 1024)).toBe('Max 10 MB each');
		expect(formatUploadLimit(1.5 * 1024 * 1024)).toBe('Max 1.5 MB each');
		expect(relativeTime('2026-06-10T12:30:00Z')).toBe('1 hr ago');
		vi.useRealTimers();
	});

	it('maps import job status and progress display state', () => {
		expect(statusForJob(job()).label).toBe('Completed');
		expect(statusForJob(job({ status: 'running' })).variant).toBe('syncing');
		expect(statusForJob(job({ status: 'rolled_back' })).label).toBe('Rolled back');
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
		expect(sourceFileLabel(job())).toBe('Readwise · files');
		expect(sourceFileLabel(job({ import_source: 'other_source' }))).toBe('other_source');
		vi.useRealTimers();
	});
});
