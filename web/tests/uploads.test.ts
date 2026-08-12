import { afterEach, describe, expect, it, vi } from 'vitest';
import { uploadLibraryFile } from '$lib/api/uploads';

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

describe('uploadLibraryFile', () => {
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
});
