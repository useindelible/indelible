import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import ImportProgressCard from '$lib/components/imports/ImportProgressCard.svelte';
import type { ImportJobStatusResponse } from '$lib/api';

function fixtureJob(overrides: Partial<ImportJobStatusResponse> = {}): ImportJobStatusResponse {
	const base = {
		id: 'imp_1',
		status: 'running',
		import_source: 'readwise_import',
		import_method: 'file_upload',
		counts: {
			imported: 1,
			updated: 0,
			duplicate: 0,
			skipped_private: 0,
			failed: 0,
			pending_ingest: 0
		},
		item_outcomes: [],
		error: null,
		created_at: '2026-04-25T12:00:00Z'
	};
	return { ...base, ...overrides } as ImportJobStatusResponse;
}

describe('ImportProgressCard', () => {
	it('renders source/method and counts for a running job', () => {
		const job = fixtureJob({
			status: 'running',
			counts: {
				imported: 7,
				updated: 1,
				duplicate: 2,
				skipped_private: 0,
				failed: 1,
				pending_ingest: 0
			}
		});
		render(ImportProgressCard, { props: { job } });
		expect(screen.getByText(/Readwise/)).toBeTruthy();
		expect(screen.getByText(/file_upload/)).toBeTruthy();
		expect(screen.getByText('Running')).toBeTruthy();
		expect(screen.getByText('7')).toBeTruthy();
		expect(screen.getByText('2')).toBeTruthy();
	});

	it('renders Queued copy and badge for pending jobs', () => {
		const job = fixtureJob({ status: 'pending' });
		render(ImportProgressCard, { props: { job } });
		expect(screen.getByText('Queued')).toBeTruthy();
	});

	it('shows queued copy while the route keeps polling awaiting_provider jobs', () => {
		const job = fixtureJob({
			status: 'awaiting_provider'
		});
		render(ImportProgressCard, { props: { job } });
		expect(screen.getByText('Queued')).toBeTruthy();
	});

	it('does not show queued copy for other statuses', () => {
		render(ImportProgressCard, { props: { job: fixtureJob() } });
		expect(screen.queryByText(/^Queued$/i)).toBeNull();
	});

	it('renders the job error when present', () => {
		const job = fixtureJob({
			status: 'failed',
			error: 'parse failed'
		});
		render(ImportProgressCard, { props: { job } });
		expect(screen.getByRole('alert').textContent).toMatch(/parse failed/);
	});

	it('renders awaiting_provider as queued while route polling continues', () => {
		render(ImportProgressCard, {
			props: { job: fixtureJob({ status: 'awaiting_provider' }) }
		});
		expect(screen.getByText('Queued')).toBeTruthy();
	});
});
