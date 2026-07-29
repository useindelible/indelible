import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

const sdkMocks = vi.hoisted(() => ({
	getImport: vi.fn(),
	rollbackImport: vi.fn()
}));

vi.mock('$lib/api', () => ({
	getImport: sdkMocks.getImport,
	rollbackImport: sdkMocks.rollbackImport
}));

vi.mock('$lib/auth-tokens', () => ({
	getAccessToken: () => 'test-token'
}));

import {
	fetchImportJob,
	rollbackImportJob,
	uploadImportFile,
	uploadReadwiseImportFiles
} from '$lib/api/imports';

describe('fetchImportJob', () => {
	beforeEach(() => {
		Object.values(sdkMocks).forEach((m) => m.mockReset());
	});

	it('returns ok with data when the job exists', async () => {
		sdkMocks.getImport.mockResolvedValueOnce({
			data: { id: 'imp_1', status: 'running' },
			error: undefined,
			response: new Response(null, { status: 200 })
		});
		const result = await fetchImportJob('imp_1');
		expect(result.status).toBe('ok');
		if (result.status === 'ok') {
			expect(result.data).toEqual({ id: 'imp_1', status: 'running' });
		}
	});

	it('returns not_found when the SDK responds with 404', async () => {
		sdkMocks.getImport.mockResolvedValueOnce({
			data: undefined,
			error: undefined,
			response: new Response(null, { status: 404 })
		});
		const result = await fetchImportJob('imp_1');
		expect(result).toEqual({ status: 'not_found' });
	});

	it('returns an error result with the API problem detail on 5xx', async () => {
		sdkMocks.getImport.mockResolvedValueOnce({
			data: undefined,
			error: { detail: 'kaboom' },
			response: new Response(null, { status: 500 })
		});
		const result = await fetchImportJob('imp_1');
		expect(result).toEqual({ status: 'error', httpStatus: 500, message: 'kaboom' });
	});

	it('returns httpStatus 0 when the SDK throws', async () => {
		sdkMocks.getImport.mockRejectedValueOnce(new Error('network'));
		const result = await fetchImportJob('imp_1');
		expect(result.status).toBe('error');
		if (result.status === 'error') expect(result.httpStatus).toBe(0);
	});
});

describe('rollbackImportJob', () => {
	beforeEach(() => {
		Object.values(sdkMocks).forEach((m) => m.mockReset());
	});

	it('returns success on a 2xx response', async () => {
		sdkMocks.rollbackImport.mockResolvedValueOnce({
			data: undefined,
			error: undefined,
			response: new Response(null, { status: 204 })
		});
		const result = await rollbackImportJob('imp_1');
		expect(result.success).toBe(true);
	});

	it('returns failure with the API problem detail on non-2xx', async () => {
		sdkMocks.rollbackImport.mockResolvedValueOnce({
			data: undefined,
			error: { detail: 'too late' },
			response: new Response(null, { status: 409 })
		});
		const result = await rollbackImportJob('imp_1');
		expect(result).toEqual({ success: false, error: 'too late' });
	});
});

describe('uploadImportFile', () => {
	let originalFetch: typeof fetch;
	beforeEach(() => {
		originalFetch = globalThis.fetch;
	});
	afterEach(() => {
		globalThis.fetch = originalFetch;
	});

	it('POSTs to the imports endpoint with multipart body and bearer auth', async () => {
		const fetchMock = vi.fn().mockResolvedValue(
			new Response(JSON.stringify({ import_job_id: 'imp_1', accepted: true }), {
				status: 200,
				headers: { 'content-type': 'application/json' }
			})
		);
		globalThis.fetch = fetchMock as unknown as typeof fetch;

		const file = new File(['<root/>'], 'export.html', { type: 'text/html' });
		const result = await uploadImportFile('readwise', file);

		expect(fetchMock).toHaveBeenCalledTimes(1);
		const [url, init] = fetchMock.mock.calls[0]!;
		expect(String(url)).toContain('/api/v1/imports/readwise');
		expect(init?.method).toBe('POST');
		expect((init?.headers as Record<string, string>)?.Authorization).toBe('Bearer test-token');
		expect(init?.body).toBeInstanceOf(FormData);

		expect(result.success).toBe(true);
		if (result.success) expect(result.data).toEqual({ import_job_id: 'imp_1', accepted: true });
	});

	it('extracts the API problem detail on a non-2xx response', async () => {
		globalThis.fetch = vi.fn().mockResolvedValue(
			new Response(JSON.stringify({ detail: 'unsupported file' }), {
				status: 415,
				headers: { 'content-type': 'application/json' }
			})
		) as unknown as typeof fetch;

		const file = new File(['x'], 'export.html', { type: 'text/html' });
		const result = await uploadImportFile('readwise', file);
		expect(result).toEqual({ success: false, error: 'unsupported file' });
	});

	it('falls back to a generic error when fetch throws', async () => {
		globalThis.fetch = vi
			.fn()
			.mockRejectedValue(new Error('disconnected')) as unknown as typeof fetch;
		const file = new File(['x'], 'export.html', { type: 'text/html' });
		const result = await uploadImportFile('readwise', file);
		expect(result.success).toBe(false);
		if (!result.success) expect(result.error).toMatch(/unexpected/i);
	});
});

describe('uploadReadwiseImportFiles', () => {
	let originalFetch: typeof fetch;
	beforeEach(() => {
		originalFetch = globalThis.fetch;
	});
	afterEach(() => {
		globalThis.fetch = originalFetch;
	});

	it('POSTs named Readwise multipart fields for CSV, ZIP, and OPML', async () => {
		const fetchMock = vi.fn().mockResolvedValue(
			new Response(JSON.stringify({ import_job_id: 'imp_1', status: 'pending' }), {
				status: 202,
				headers: { 'content-type': 'application/json' }
			})
		);
		globalThis.fetch = fetchMock as unknown as typeof fetch;

		const result = await uploadReadwiseImportFiles({
			libraryCsv: new File(['csv'], 'export.csv', { type: 'text/csv' }),
			archiveZip: new File(['zip'], 'Reader_Uploaded_Files.zip', { type: 'application/zip' }),
			feedsOpml: new File(['opml'], 'feeds.opml', { type: 'text/xml' })
		});

		expect(fetchMock).toHaveBeenCalledTimes(1);
		const [url, init] = fetchMock.mock.calls[0]!;
		expect(String(url)).toContain('/api/v1/imports/readwise');
		expect(init?.method).toBe('POST');
		expect((init?.headers as Record<string, string>)?.Authorization).toBe('Bearer test-token');
		const body = init?.body as FormData;
		expect(body.get('library_csv')).toBeInstanceOf(File);
		expect(body.get('archive_zip')).toBeInstanceOf(File);
		expect(body.get('feeds_opml')).toBeInstanceOf(File);
		expect(result.success).toBe(true);
	});

	it('returns API validation errors for malformed Readwise uploads', async () => {
		globalThis.fetch = vi.fn().mockResolvedValue(
			new Response(
				JSON.stringify({
					detail: 'at least one of library_csv, archive_zip, or feeds_opml is required'
				}),
				{
					status: 400,
					headers: { 'content-type': 'application/json' }
				}
			)
		) as unknown as typeof fetch;

		const result = await uploadReadwiseImportFiles({});
		expect(result).toEqual({
			success: false,
			error: 'at least one of library_csv, archive_zip, or feeds_opml is required'
		});
	});
});
