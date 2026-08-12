import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { DocumentReaderAssetResponse } from '$lib/api';

const mockReprocessDocument = vi.fn();

vi.mock('$lib/api', () => ({
	reprocessDocument: (...args: unknown[]) => mockReprocessDocument(...args)
}));

import { ReaderRetryController } from './reader-retry.svelte';

const failedAsset: DocumentReaderAssetResponse = {
	id: 'asset_failed',
	asset_kind: 'readable_html',
	content_type: 'text/html',
	created_at: '2026-08-12T05:00:00Z',
	failed_reason: 'external service error from renderer: error sending request for url',
	size_bytes: 0,
	status: 'failed'
};

async function queueRetry(controller: ReaderRetryController) {
	await controller.submit({
		documentId: 'doc_1',
		item: null,
		assets: [failedAsset],
		onRetryPolling: vi.fn()
	});
}

beforeEach(() => {
	vi.useFakeTimers();
	mockReprocessDocument.mockReset();
	mockReprocessDocument.mockResolvedValue({
		data: { queued: true, job_type: 'document.reprocess' }
	});
});

afterEach(() => {
	vi.useRealTimers();
});

describe('ReaderRetryController completion outcome', () => {
	it('does not announce success for content that was already ready', () => {
		const controller = new ReaderRetryController();

		controller.onPreparationReady(true);

		expect(controller.outcome).toBeNull();
	});

	it('announces when a queued retry becomes ready', async () => {
		const controller = new ReaderRetryController();
		await queueRetry(controller);

		controller.onPreparationReady(true);

		expect(controller.outcome).toBe('Readable content is ready.');
	});

	it('keeps the completion armed after the queue button cooldown expires', async () => {
		const controller = new ReaderRetryController();
		await queueRetry(controller);
		await vi.advanceTimersByTimeAsync(5 * 60 * 1000);

		controller.onPreparationReady(true);

		expect(controller.outcome).toBe('Readable content is ready.');
	});

	it('does not announce completion after enqueue failure', async () => {
		mockReprocessDocument.mockRejectedValueOnce(new Error('network'));
		const controller = new ReaderRetryController();
		await queueRetry(controller);

		controller.onPreparationReady(true);

		expect(controller.outcome).toBeNull();
	});

	it('clears an earlier completion when a new retry begins', async () => {
		const controller = new ReaderRetryController();
		await queueRetry(controller);
		controller.onPreparationReady(true);

		await queueRetry(controller);

		expect(controller.outcome).toBeNull();
	});

	it('clears a queued result when the reader changes documents', async () => {
		const controller = new ReaderRetryController();
		await queueRetry(controller);

		controller.reset();
		controller.onPreparationReady(true);

		expect(controller.state).toBe('idle');
		expect(controller.status).toBeNull();
		expect(controller.error).toBeNull();
		expect(controller.outcome).toBeNull();
	});

	it('ignores a reprocess response that resolves after the reader changes documents', async () => {
		let resolveReprocess!: (value: { data: { queued: boolean; job_type: string } }) => void;
		mockReprocessDocument.mockReturnValueOnce(
			new Promise((resolve) => {
				resolveReprocess = resolve;
			})
		);
		const controller = new ReaderRetryController();
		const onRetryPolling = vi.fn();
		const submit = controller.submit({
			documentId: 'doc_1',
			item: null,
			assets: [failedAsset],
			onRetryPolling
		});

		controller.reset();
		resolveReprocess({ data: { queued: true, job_type: 'document.reprocess' } });
		await submit;

		expect(controller.state).toBe('idle');
		expect(controller.status).toBeNull();
		expect(controller.outcome).toBeNull();
		expect(onRetryPolling).not.toHaveBeenCalled();
	});
});
