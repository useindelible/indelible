import { describe, expect, it } from 'vitest';

import type { DocumentListEntry, DocumentReaderAssetResponse } from '$lib/api';

import {
	hasFailedReadableAsset,
	isReaderContentReady,
	shouldReprocessReaderPreparation
} from './reader-page-model';

function readableAsset(status: string): DocumentReaderAssetResponse {
	return {
		asset_kind: 'readable_html',
		content_type: 'text/html',
		created_at: '2026-07-12T00:00:00Z',
		failed_reason: status === 'failed' ? 'page blocked by anti-bot challenge' : null,
		id: 'asset-1',
		size_bytes: 0,
		status
	};
}

describe('shouldReprocessReaderPreparation', () => {
	it('offers retry for a failed readable extraction', () => {
		expect(shouldReprocessReaderPreparation(null, [readableAsset('failed')])).toBe(true);
	});

	it('does not offer retry for completed readable content', () => {
		expect(shouldReprocessReaderPreparation(null, [readableAsset('completed')])).toBe(false);
	});

	it('does not trust stale item readiness over a failed asset', () => {
		const staleItem = { readable_ready: true } as DocumentListEntry;

		expect(isReaderContentReady(staleItem, [readableAsset('failed')])).toBe(false);
		expect(hasFailedReadableAsset([readableAsset('failed')])).toBe(true);
	});
});
