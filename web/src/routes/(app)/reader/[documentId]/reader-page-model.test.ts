import { describe, expect, it } from 'vitest';

import type { DocumentListEntry, DocumentReaderAssetResponse } from '$lib/api';

import {
	hasFailedReadableAsset,
	isReaderContentReady,
	readerFailurePresentation,
	shouldReprocessReaderPreparation
} from './reader-page-model';

function readableAsset(
	status: string,
	failedReason: string | null = status === 'failed' ? 'page blocked by anti-bot challenge' : null
): DocumentReaderAssetResponse {
	return {
		asset_kind: 'readable_html',
		content_type: 'text/html',
		created_at: '2026-07-12T00:00:00Z',
		failed_reason: failedReason,
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

describe('readerFailurePresentation', () => {
	it.each([
		[
			'external service error from renderer: error sending request for url',
			'service',
			'Rendering service unavailable'
		],
		[
			'validation error on field `url`: renderer rejected url: {"error":"URL host is not allowed"}',
			'access_or_policy',
			'Capture blocked'
		],
		[
			'readable_html extraction: defuddle: defuddle produced too little visible readable content',
			'content',
			'No readable article found'
		],
		['unexpected preparation failure', 'unknown', 'Readable content unavailable']
	] as const)('classifies %s without inventing a source diagnosis', (reason, kind, title) => {
		expect(readerFailurePresentation([readableAsset('failed', reason)])).toEqual({
			kind,
			title,
			guidance:
				kind === 'service'
					? 'The rendering service could not prepare this page. Retry when the service is available.'
					: kind === 'access_or_policy'
						? "The page requires access, or this server's capture policy prevented the request."
						: kind === 'content'
							? 'Indelible could not find enough article text to create a readable version.'
							: 'Preparation failed, but the cause could not be determined.',
			diagnosticId: 'asset-1',
			attemptedAt: '2026-07-12T00:00:00Z',
			technicalReason: reason
		});
	});

	it.each([
		'This YouTube video is unavailable, private, or deleted.',
		'validation error on field `url`: renderer rejected url: {"error":"must be a valid URL"}',
		'validation error on field `url`: renderer rejected url: {"error":"could not resolve the URL host"}',
		'readable extraction timed out while loading the source'
	])('uses neutral copy when %s does not prove access or policy blocking', (reason) => {
		expect(readerFailurePresentation([readableAsset('failed', reason)]))?.toMatchObject({
			kind: 'unknown',
			title: 'Readable content unavailable'
		});
	});

	it.each([
		'page blocked by anti-bot challenge',
		'renderer returned HTTP 403: forbidden',
		'validation error on field `url`: renderer rejected url: {"error":"URL host is not allowed"}',
		'validation error on field `url`: renderer rejected url: {"error":"URL resolves to a private or internal address"}'
	])('uses access copy only for explicit access or policy evidence: %s', (reason) => {
		expect(readerFailurePresentation([readableAsset('failed', reason)]))?.toMatchObject({
			kind: 'access_or_policy',
			title: 'Capture blocked'
		});
	});

	it('ignores failures for assets outside the readable article path', () => {
		const extractedTextFailure: DocumentReaderAssetResponse = {
			...readableAsset('failed', 'PDF text extraction failed: no extractable text'),
			asset_kind: 'extracted_text'
		};

		expect(readerFailurePresentation([extractedTextFailure])).toBeNull();
		expect(readerFailurePresentation([readableAsset('completed', null)])).toBeNull();
	});
});
