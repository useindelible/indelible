import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('./generated', () => ({
	getDocumentReader: vi.fn(),
	getLibraryEntry: vi.fn()
}));

import * as generated from './generated';
import { getDocumentEntry, listAssets } from './index';

const getDocumentReader = vi.mocked(generated.getDocumentReader);
const getLibraryEntry = vi.mocked(generated.getLibraryEntry);

function readerResponse(documentId: string, status = 200) {
	return {
		data: {
			document_id: documentId,
			document_type: 'article',
			library_entry_id: null,
			available_assets: [],
			assets: []
		},
		response: { status } as Response
	};
}

function deferred<T>() {
	let resolve!: (value: T) => void;
	const promise = new Promise<T>((r) => {
		resolve = r;
	});
	return { promise, resolve };
}

beforeEach(() => {
	vi.clearAllMocks();
});

describe('getDocumentEntry in-flight deduplication', () => {
	it('collapses concurrent requests for the same document into one fetch', async () => {
		const pending = deferred<ReturnType<typeof readerResponse>>();
		getDocumentReader.mockReturnValue(pending.promise as never);

		const a = getDocumentEntry({ path: { document_id: 'doc_dedup' } });
		const b = getDocumentEntry({ path: { document_id: 'doc_dedup' } });

		expect(getDocumentReader).toHaveBeenCalledTimes(1);

		pending.resolve(readerResponse('doc_dedup'));
		const [ra, rb] = await Promise.all([a, b]);

		expect(ra.data?.id).toBe('doc_dedup');
		expect(rb.data?.id).toBe('doc_dedup');
		expect(getDocumentReader).toHaveBeenCalledTimes(1);
	});

	it('refetches once the previous request has settled', async () => {
		getDocumentReader.mockResolvedValue(readerResponse('doc_seq') as never);

		await getDocumentEntry({ path: { document_id: 'doc_seq' } });
		await getDocumentEntry({ path: { document_id: 'doc_seq' } });

		expect(getDocumentReader).toHaveBeenCalledTimes(2);
	});

	it('does not share a promise across distinct document ids', async () => {
		getDocumentReader.mockImplementation(((opts: { path: { document_id: string } }) =>
			Promise.resolve(readerResponse(opts.path.document_id))) as never);

		await Promise.all([
			getDocumentEntry({ path: { document_id: 'doc_a' } }),
			getDocumentEntry({ path: { document_id: 'doc_b' } })
		]);

		expect(getDocumentReader).toHaveBeenCalledTimes(2);
	});
});

describe('getDocumentEntry status classification', () => {
	it('reports a 404 with no data so callers can drop a gone document', async () => {
		getDocumentReader.mockResolvedValue({
			data: undefined,
			response: { status: 404 } as Response
		} as never);

		const result = await getDocumentEntry({ path: { document_id: 'doc_gone' } });

		expect(result.data).toBeUndefined();
		expect(result.status).toBe(404);
	});

	it('reports a 429 with no data so callers can keep a still-valid document', async () => {
		getDocumentReader.mockResolvedValue({
			data: undefined,
			response: { status: 429 } as Response
		} as never);

		const result = await getDocumentEntry({ path: { document_id: 'doc_limited' } });

		expect(result.data).toBeUndefined();
		expect(result.status).toBe(429);
	});

	it('reports undefined status on a network-level failure', async () => {
		getDocumentReader.mockResolvedValue({
			data: undefined,
			response: undefined
		} as never);

		const result = await getDocumentEntry({ path: { document_id: 'doc_offline' } });

		expect(result.data).toBeUndefined();
		expect(result.status).toBeUndefined();
	});
});

describe('getLibraryEntry is unused for reader-backed documents', () => {
	it('does not call getLibraryEntry when the reader has no library entry', async () => {
		getDocumentReader.mockResolvedValue(readerResponse('doc_no_entry') as never);

		await getDocumentEntry({ path: { document_id: 'doc_no_entry' } });

		expect(getLibraryEntry).not.toHaveBeenCalled();
	});
});

describe('listAssets reader metadata', () => {
	it('returns failed and degraded reader assets so retry can reprocess them', async () => {
		getDocumentReader.mockResolvedValue({
			data: {
				...readerResponse('doc_failed').data,
				assets: [
					{
						id: 'asset_readable',
						asset_kind: 'readable_html',
						content_type: 'text/html',
						created_at: '2026-07-08T12:00:00Z',
						failed_reason: 'readable extraction failed',
						size_bytes: 0,
						status: 'failed'
					},
					{
						id: 'asset_pdf',
						asset_kind: 'pdf',
						content_type: 'application/pdf',
						created_at: '2026-07-08T12:00:01Z',
						failed_reason: 'no text layer',
						size_bytes: 123,
						status: 'degraded'
					}
				]
			},
			response: { status: 200 } as Response
		} as never);

		const result = await listAssets({ path: { document_id: 'doc_failed' } });

		expect(result.data?.data).toMatchObject([
			{
				id: 'asset_readable',
				asset_kind: 'readable_html',
				status: 'failed',
				failed_reason: 'readable extraction failed'
			},
			{
				id: 'asset_pdf',
				asset_kind: 'pdf',
				status: 'degraded',
				failed_reason: 'no text layer'
			}
		]);
	});
});
