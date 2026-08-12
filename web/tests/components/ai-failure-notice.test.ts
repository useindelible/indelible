import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import AiFailureNotice from '../../src/routes/(app)/reader/[documentId]/components/AiFailureNotice.svelte';

describe('AiFailureNotice', () => {
	it('explains a failed action without exposing provider details by default', () => {
		render(AiFailureNotice, {
			failure: {
				documentId: 'doc_1',
				action: 'tags',
				aiRunId: 'airun_1',
				message: 'raw provider error'
			},
			status: 'idle',
			onRetry: vi.fn(),
			onDismiss: vi.fn()
		});

		expect(screen.getByText("Mila couldn't suggest tags.")).toBeTruthy();
		expect(screen.getByText('Run airun_1')).toBeTruthy();
		expect(
			(screen.getByText('Technical details').closest('details') as HTMLDetailsElement).open
		).toBe(false);
		expect(screen.getByRole('link', { name: 'Open Mila settings' }).getAttribute('href')).toBe(
			'/preferences/ai'
		);
	});

	it('wires retry and dismiss while reporting retry state honestly', async () => {
		const onRetry = vi.fn();
		const onDismiss = vi.fn();
		const { rerender } = render(AiFailureNotice, {
			failure: {
				documentId: 'doc_1',
				action: 'entities',
				aiRunId: 'airun_2',
				message: 'provider rejected output'
			},
			status: 'idle',
			onRetry,
			onDismiss
		});

		await fireEvent.click(screen.getByRole('button', { name: 'Retry' }));
		await fireEvent.click(screen.getByRole('button', { name: 'Dismiss' }));
		expect(onRetry).toHaveBeenCalledOnce();
		expect(onDismiss).toHaveBeenCalledOnce();

		await rerender({
			failure: {
				documentId: 'doc_1',
				action: 'entities',
				aiRunId: 'airun_2',
				message: 'provider rejected output'
			},
			status: 'pending',
			onRetry,
			onDismiss
		});
		expect(
			(screen.getByRole('button', { name: 'Queuing retry…' }) as HTMLButtonElement).disabled
		).toBe(true);

		await rerender({
			failure: {
				documentId: 'doc_1',
				action: 'entities',
				aiRunId: 'airun_2',
				message: 'provider rejected output'
			},
			status: 'queued',
			onRetry,
			onDismiss
		});
		expect(screen.getByText('Retry queued.')).toBeTruthy();
	});
});
