import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import type { ImportJobStatusResponse } from '$lib/api';
import ImportHistoryTable from '../../src/routes/(app)/preferences/integrations/components/ImportHistoryTable.svelte';

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

describe('ImportHistoryTable', () => {
	it('renders import rows and rollback callbacks for terminal jobs', async () => {
		vi.setSystemTime(new Date('2026-06-10T14:00:00Z'));
		const onRollback = vi.fn();
		render(ImportHistoryTable, { props: { history: [job()], onRollback } });

		expect(screen.getByText('Readwise · files')).toBeTruthy();
		expect(screen.getByText('Completed')).toBeTruthy();
		expect(screen.getByText('12')).toBeTruthy();

		await fireEvent.click(screen.getByRole('button', { name: /roll back/i }));
		expect(onRollback).toHaveBeenCalledWith('imp_1');
		vi.useRealTimers();
	});

	it('renders an empty state when no imports exist', () => {
		render(ImportHistoryTable, { props: { history: [], onRollback: vi.fn() } });
		expect(screen.getByText('No imports yet. Drop a file above to start.')).toBeTruthy();
	});
});
