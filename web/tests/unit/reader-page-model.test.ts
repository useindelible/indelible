import { describe, expect, it } from 'vitest';

import type { DocumentListEntry, DocumentReaderAssetResponse } from '$lib/api';
import {
	computeArticlePdfInitialPage,
	computeAvailableReaderTabs,
	isBookReaderItem,
	isReadableReady,
	isSavedToLibrary
} from '../../src/routes/(app)/reader/[documentId]/reader-page-model';
import {
	READER_AI_EVENTS,
	readerEventItemId
} from '../../src/routes/(app)/reader/[documentId]/reader-highlights';

function asset(
	assetKind: DocumentReaderAssetResponse['asset_kind'],
	status: DocumentReaderAssetResponse['status'] = 'completed'
): DocumentReaderAssetResponse {
	return {
		id: `asset_${assetKind}`,
		asset_kind: assetKind,
		content_type: 'text/html',
		created_at: '2026-05-18T10:00:00Z',
		size_bytes: 1,
		status
	};
}

function item(overrides: Partial<DocumentListEntry> = {}): DocumentListEntry {
	return {
		id: 'doc_1',
		document_id: 'doc_1',
		title: 'Reader item',
		url: 'https://example.com',
		document_type: 'article',
		item_type: 'article',
		object: 'library_entry',
		source: 'web',
		created_at: '2026-05-18T10:00:00Z',
		updated_at: '2026-05-18T10:00:00Z',
		saved_at: '2026-05-18T10:00:00Z',
		triage_state: 'inbox',
		is_favorite: false,
		is_shortlisted: false,
		library_entry_id: null,
		saved: false,
		available_assets: [],
		readable_ready: false,
		...overrides
	};
}

describe('reader page model', () => {
	it('derives available tabs from completed assets only', () => {
		expect(
			computeAvailableReaderTabs([
				asset('readable_html'),
				asset('original_html'),
				asset('pdf', 'pending'),
				asset('screenshot')
			])
		).toEqual(['reader', 'original', 'screenshot']);
	});

	it('derives readable, saved, and book states', () => {
		expect(isBookReaderItem(item({ item_type: 'book' }))).toBe(true);
		expect(isBookReaderItem(item({ item_type: 'pdf' }))).toBe(true);
		expect(isReadableReady(item({ readable_ready: true }))).toBe(true);
		expect(
			isReadableReady(
				item({
					readable_ready: undefined,
					available_assets: ['readable_html']
				} as Partial<DocumentListEntry>)
			)
		).toBe(true);
		expect(isSavedToLibrary(item({ saved: true }))).toBe(true);
		expect(
			isSavedToLibrary(
				item({ saved: undefined, library_entry_id: 'lib_1' } as Partial<DocumentListEntry>)
			)
		).toBe(true);
	});

	it('computes the PDF initial page from locator or progress', () => {
		expect(computeArticlePdfInitialPage(10, 'page:4', 0)).toBe(3);
		expect(computeArticlePdfInitialPage(10, null, 25)).toBe(2);
		expect(computeArticlePdfInitialPage(10, 'page:99', 0)).toBe(9);
		expect(computeArticlePdfInitialPage(null, 'page:2', 50)).toBe(0);
	});

	it('routes highlight events by document id payload or aggregate id', () => {
		expect(
			readerEventItemId({
				aggregate_type: 'highlight',
				aggregate_id: 'hl_1',
				payload: { document_id: 'doc_payload' }
			})
		).toBe('doc_payload');
		expect(
			readerEventItemId({
				aggregate_type: 'document',
				aggregate_id: 'doc_aggregate',
				payload: null
			})
		).toBe('doc_aggregate');
	});

	it('subscribes the reader to AI completion and failure events', () => {
		expect([...READER_AI_EVENTS]).toEqual(['ai.output.completed', 'ai.output.failed']);
	});
});
