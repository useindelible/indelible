import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { SearchResultResponse } from '$lib/api/generated/types.gen';

const mockPrepareFeedDelivery = vi.fn();
const mockGoto = vi.fn();

vi.mock('$lib/api', () => ({
	prepareFeedDelivery: (...args: unknown[]) => mockPrepareFeedDelivery(...args)
}));
vi.mock('$app/navigation', () => ({ goto: (...args: unknown[]) => mockGoto(...args) }));
vi.mock('$app/paths', () => ({ resolve: (path: string) => path }));

import { openSearchResult } from '$lib/components/search/open-result';

function result(overrides: Partial<SearchResultResponse> = {}): SearchResultResponse {
	return {
		result_kind: 'document',
		title: 'A Result',
		snippet: 'snippet',
		score: 0.5,
		content_type: 'article',
		saved_at: '2026-05-18T10:00:00Z',
		updated_at: '2026-05-18T10:00:00Z',
		...overrides
	};
}

describe('openSearchResult', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('navigates a durable result directly to the library reader', async () => {
		await openSearchResult(result({ document_id: 'doc_durable' }));

		expect(mockPrepareFeedDelivery).not.toHaveBeenCalled();
		expect(mockGoto).toHaveBeenCalledWith('/(app)/reader/[documentId]');
	});

	it('prepares a feed_preview result then navigates to the document reader', async () => {
		mockPrepareFeedDelivery.mockResolvedValue({ data: { document_id: 'doc_prepared' } });

		await openSearchResult(
			result({ result_kind: 'feed_preview', document_id: null, delivery_id: 'dlv_preview' })
		);

		expect(mockPrepareFeedDelivery).toHaveBeenCalledWith({ path: { delivery_id: 'dlv_preview' } });
		expect(mockGoto).toHaveBeenCalledWith('/(app)/reader/[documentId]');
	});

	it('does not navigate when preparation rejects', async () => {
		mockPrepareFeedDelivery.mockRejectedValue(new Error('no canonical url'));

		await openSearchResult(
			result({ result_kind: 'feed_preview', document_id: null, delivery_id: 'dlv_nourl' })
		);

		expect(mockGoto).not.toHaveBeenCalled();
	});

	it('is a no-op when a preview result has no delivery id', async () => {
		await openSearchResult(result({ document_id: null, delivery_id: null }));

		expect(mockPrepareFeedDelivery).not.toHaveBeenCalled();
		expect(mockGoto).not.toHaveBeenCalled();
	});
});
