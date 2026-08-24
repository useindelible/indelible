import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { uploadLibraryFile } from '$lib/api/uploads';
import { getMaxUploadBytes, resetUploadLimitsCache } from '$lib/api/upload-limits';

vi.mock('$lib/api/upload-limits', () => ({
	getMaxUploadBytes: vi.fn(),
	resetUploadLimitsCache: vi.fn()
}));

const getMaxUploadBytesMock = vi.mocked(getMaxUploadBytes);
const resetUploadLimitsCacheMock = vi.mocked(resetUploadLimitsCache);

class FakeXmlHttpRequest {
	static responseStatus = 422;
	static responseText = '';

	status = 0;
	responseText = '';
	withCredentials = false;
	upload = { onprogress: null } as unknown as XMLHttpRequestUpload;
	onload: XMLHttpRequest['onload'] = null;
	onerror: XMLHttpRequest['onerror'] = null;

	open(): void {}
	setRequestHeader(): void {}

	send(): void {
		this.status = FakeXmlHttpRequest.responseStatus;
		this.responseText = FakeXmlHttpRequest.responseText;
		this.onload?.call(this as unknown as XMLHttpRequest, new ProgressEvent('load'));
	}
}

const pdf = () => new File(['%PDF-1.7'], 'book.pdf', { type: 'application/pdf' });

describe('uploadLibraryFile', () => {
	beforeEach(() => {
		FakeXmlHttpRequest.responseStatus = 422;
		FakeXmlHttpRequest.responseText = '';
		getMaxUploadBytesMock.mockReset();
		getMaxUploadBytesMock.mockResolvedValue(null);
		resetUploadLimitsCacheMock.mockReset();
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('surfaces the field error when a password-protected PDF is rejected', async () => {
		FakeXmlHttpRequest.responseText = JSON.stringify({
			detail: 'validation error',
			errors: [{ field: 'file', message: 'Password-protected PDFs are not supported.' }]
		});
		vi.stubGlobal('XMLHttpRequest', FakeXmlHttpRequest);

		const result = await uploadLibraryFile(
			new File(['%PDF-1.7'], 'protected.pdf', { type: 'application/pdf' })
		);

		expect(result).toEqual({
			success: false,
			error: 'Password-protected PDFs are not supported.'
		});
	});

	it('localizes a 413 with the server limit when the lookup succeeds', async () => {
		getMaxUploadBytesMock.mockResolvedValue(50 * 1024 * 1024);
		FakeXmlHttpRequest.responseStatus = 413;
		FakeXmlHttpRequest.responseText = JSON.stringify({ detail: 'payload too large' });
		vi.stubGlobal('XMLHttpRequest', FakeXmlHttpRequest);

		const result = await uploadLibraryFile(pdf());

		expect(result).toEqual({
			success: false,
			error: 'File is too large. Maximum size is 50 MB.'
		});
	});

	it('refetches the limit after a 413 instead of reusing the stale cached one', async () => {
		// The modal cached 50 MB on open; the server has since lowered the cap to 10 MB.
		let served = 50 * 1024 * 1024;
		resetUploadLimitsCacheMock.mockImplementation(() => {
			served = 10 * 1024 * 1024;
		});
		getMaxUploadBytesMock.mockImplementation(async () => served);
		FakeXmlHttpRequest.responseStatus = 413;
		FakeXmlHttpRequest.responseText = JSON.stringify({ detail: 'payload too large' });
		vi.stubGlobal('XMLHttpRequest', FakeXmlHttpRequest);

		const result = await uploadLibraryFile(pdf());

		expect(result).toEqual({
			success: false,
			error: 'File is too large. Maximum size is 10 MB.'
		});
		expect(resetUploadLimitsCacheMock).toHaveBeenCalledTimes(1);
		expect(resetUploadLimitsCacheMock.mock.invocationCallOrder[0]).toBeLessThan(
			getMaxUploadBytesMock.mock.invocationCallOrder[0]
		);
	});

	it('localizes a 413 whose body is plain text rather than JSON', async () => {
		getMaxUploadBytesMock.mockResolvedValue(50 * 1024 * 1024);
		FakeXmlHttpRequest.responseStatus = 413;
		FakeXmlHttpRequest.responseText = 'length limit exceeded';
		vi.stubGlobal('XMLHttpRequest', FakeXmlHttpRequest);

		const result = await uploadLibraryFile(pdf());

		expect(result).toEqual({
			success: false,
			error: 'File is too large. Maximum size is 50 MB.'
		});
	});

	it('falls back to the generic message when the limit is unknown', async () => {
		getMaxUploadBytesMock.mockResolvedValue(null);
		FakeXmlHttpRequest.responseStatus = 413;
		FakeXmlHttpRequest.responseText = JSON.stringify({ detail: 'payload too large' });
		vi.stubGlobal('XMLHttpRequest', FakeXmlHttpRequest);

		const result = await uploadLibraryFile(pdf());

		expect(result).toEqual({
			success: false,
			error: 'File is too large for this server.'
		});
	});

	it('leaves non-413 failures to the problem-detail message', async () => {
		FakeXmlHttpRequest.responseStatus = 500;
		FakeXmlHttpRequest.responseText = JSON.stringify({ detail: 'An unexpected error occurred' });
		vi.stubGlobal('XMLHttpRequest', FakeXmlHttpRequest);

		const result = await uploadLibraryFile(pdf());

		expect(result).toEqual({
			success: false,
			error: 'An unexpected error occurred'
		});
		expect(getMaxUploadBytesMock).not.toHaveBeenCalled();
		expect(resetUploadLimitsCacheMock).not.toHaveBeenCalled();
	});
});
