import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import ImportReport from '$lib/components/imports/ImportReport.svelte';
import type { ImportJobItemOutcomeDto, ImportJobStatusResponse } from '$lib/api';

function outcome(id: string, kind: string, error?: string): ImportJobItemOutcomeDto {
	return { external_id: id, outcome: kind, error: error ?? null } as ImportJobItemOutcomeDto;
}

function fixtureJob(overrides: Partial<ImportJobStatusResponse> = {}): ImportJobStatusResponse {
	const base = {
		id: 'imp_1',
		status: 'completed',
		import_source: 'readwise_import',
		import_method: 'file_upload',
		counts: {
			imported: 3,
			updated: 1,
			duplicate: 2,
			skipped_private: 0,
			failed: 1
		},
		csv_fallback_available: false,
		item_outcomes: [outcome('a', 'imported'), outcome('b', 'failed', 'oops')],
		error: null,
		created_at: '2026-04-25T12:00:00Z'
	};
	return { ...base, ...overrides } as ImportJobStatusResponse;
}

describe('ImportReport', () => {
	it('renders the summary counts', () => {
		render(ImportReport, { props: { job: fixtureJob() } });
		expect(screen.getByText(/3 imported/)).toBeTruthy();
		expect(screen.getByText(/1 failed/)).toBeTruthy();
		expect(screen.getByText(/2 duplicates/)).toBeTruthy();
	});

	it('renders per-item outcomes with error detail', () => {
		render(ImportReport, { props: { job: fixtureJob() } });
		expect(screen.getByText('a')).toBeTruthy();
		expect(screen.getByText('b')).toBeTruthy();
		expect(screen.getByText('oops')).toBeTruthy();
	});

	it('shows the rollback button when canRollback is true and status is completed', () => {
		const onRollback = vi.fn();
		render(ImportReport, {
			props: { job: fixtureJob(), canRollback: true, onRollback }
		});
		const rollback = screen.getByText('Roll back import').closest('button');
		expect(rollback).toBeTruthy();
	});

	it('shows the rollback button when status is partial', () => {
		render(ImportReport, {
			props: {
				job: fixtureJob({ status: 'partial' }),
				canRollback: true,
				onRollback: () => {}
			}
		});
		expect(screen.queryByText('Roll back import')).toBeTruthy();
	});

	it('hides the rollback button when canRollback is false', () => {
		render(ImportReport, { props: { job: fixtureJob(), canRollback: false } });
		expect(screen.queryByText('Roll back import')).toBeNull();
	});

	it('hides the rollback button when status is failed', () => {
		render(ImportReport, {
			props: {
				job: fixtureJob({ status: 'failed' }),
				canRollback: true,
				onRollback: () => {}
			}
		});
		expect(screen.queryByText('Roll back import')).toBeNull();
	});

	it('hides the rollback button when status is rolled_back', () => {
		render(ImportReport, {
			props: {
				job: fixtureJob({ status: 'rolled_back' }),
				canRollback: true,
				onRollback: () => {}
			}
		});
		expect(screen.queryByText('Roll back import')).toBeNull();
	});

	it('invokes onRollback when the button is clicked', async () => {
		const onRollback = vi.fn();
		render(ImportReport, {
			props: { job: fixtureJob(), canRollback: true, onRollback }
		});
		await fireEvent.click(screen.getByText('Roll back import'));
		expect(onRollback).toHaveBeenCalled();
	});

	it('truncates the outcomes list to initialOutcomeLimit and exposes Show all', async () => {
		const outcomes = Array.from({ length: 25 }, (_, i) => outcome(`id-${i}`, 'imported'));
		render(ImportReport, {
			props: {
				job: fixtureJob({ item_outcomes: outcomes }),
				initialOutcomeLimit: 10
			}
		});
		expect(screen.getByText('id-0')).toBeTruthy();
		expect(screen.queryByText('id-20')).toBeNull();
		const showAll = screen.getByText(/Show all 25/i);
		await fireEvent.click(showAll);
		expect(screen.getByText('id-20')).toBeTruthy();
	});

	it('renders Readwise report details including unmatched ZIP assets and embedding jobs', () => {
		render(ImportReport, {
			props: {
				job: fixtureJob({
					import_source: 'readwise_import',
					readwise_report: {
						csv_rows: 4,
						reading_progress_rows: 3,
						zip_files_total: 3,
						zip_files_matched: 2,
						zip_files_unmatched: 1,
						unmatched_zip_assets: ['Library/Loose asset (01ZIP).html'],
						archive_assets_imported: 5,
						search_reindex_jobs_enqueued: 4,
						embedding_jobs_enqueued: 4,
						opml_feeds_created: 2,
						opml_feeds_skipped: 1,
						opml_errors: []
					}
				})
			}
		});

		expect(screen.getByText('Readwise details')).toBeTruthy();
		expect(screen.getByText('Article rows')).toBeTruthy();
		expect(screen.queryByText('Highlights')).toBeNull();
		expect(screen.getByText('Progress rows')).toBeTruthy();
		expect(screen.getByText('Library/Loose asset (01ZIP).html')).toBeTruthy();
		expect(screen.getByText(/4 search jobs and 4 Mila embedding jobs/)).toBeTruthy();
	});
});
