import { beforeEach, describe, expect, it, vi } from 'vitest';

import { getMaxUploadBytes, resetUploadLimitsCache } from '$lib/api/upload-limits';
import { uploadLimits } from '$lib/api/generated/sdk.gen';

vi.mock('$lib/api/generated/sdk.gen', () => ({
	uploadLimits: vi.fn()
}));

const uploadLimitsMock = vi.mocked(uploadLimits);

describe('getMaxUploadBytes', () => {
	beforeEach(() => {
		resetUploadLimitsCache();
		uploadLimitsMock.mockReset();
	});

	it('retries after a transient failure instead of caching it', async () => {
		uploadLimitsMock
			.mockRejectedValueOnce(new Error('network down'))
			// @ts-expect-error the generated envelope carries more fields than the test needs
			.mockResolvedValueOnce({ data: { max_upload_bytes: 52_428_800 } });

		expect(await getMaxUploadBytes()).toBeNull();
		expect(await getMaxUploadBytes()).toBe(52_428_800);
		expect(uploadLimitsMock).toHaveBeenCalledTimes(2);
	});

	it('caches a successful lookup', async () => {
		// @ts-expect-error the generated envelope carries more fields than the test needs
		uploadLimitsMock.mockResolvedValue({ data: { max_upload_bytes: 52_428_800 } });

		expect(await getMaxUploadBytes()).toBe(52_428_800);
		expect(await getMaxUploadBytes()).toBe(52_428_800);
		expect(uploadLimitsMock).toHaveBeenCalledTimes(1);
	});

	it('resolves null for a malformed body and does not cache it', async () => {
		// @ts-expect-error the generated envelope carries more fields than the test needs
		uploadLimitsMock.mockResolvedValueOnce({ data: {} });
		expect(await getMaxUploadBytes()).toBeNull();

		// @ts-expect-error the generated envelope carries more fields than the test needs
		uploadLimitsMock.mockResolvedValueOnce({ data: { max_upload_bytes: 0 } });
		expect(await getMaxUploadBytes()).toBeNull();

		// @ts-expect-error the generated envelope carries more fields than the test needs
		uploadLimitsMock.mockResolvedValueOnce({ data: { max_upload_bytes: 1024 } });
		expect(await getMaxUploadBytes()).toBe(1024);
		expect(uploadLimitsMock).toHaveBeenCalledTimes(3);
	});
});
