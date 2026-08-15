import { describe, it, expect, vi, beforeEach } from 'vitest';
import { uploadAvatar, MAX_AVATAR_SIZE_BYTES } from '$lib/api/avatar';

vi.mock('$lib/api', () => ({
	uploadAvatar: vi.fn()
}));

import { uploadAvatar as uploadAvatarRequest } from '$lib/api';

const mockUploadAvatarRequest = vi.mocked(uploadAvatarRequest);

function makeFile(name: string, type: string, sizeBytes: number): File {
	const content = new Uint8Array(sizeBytes).fill(0);
	return new File([content], name, { type });
}

describe('uploadAvatar', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('rejects unsupported image types', async () => {
		const file = makeFile('image.gif', 'image/gif', 100);
		const result = await uploadAvatar(file);
		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe('invalid_type');
		}
		expect(mockUploadAvatarRequest).not.toHaveBeenCalled();
	});

	it('rejects files over the size limit', async () => {
		const file = makeFile('big.png', 'image/png', MAX_AVATAR_SIZE_BYTES + 1);
		const result = await uploadAvatar(file);
		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe('too_large');
		}
		expect(mockUploadAvatarRequest).not.toHaveBeenCalled();
	});

	it('uploads the file as multipart and returns the resolved avatar URL', async () => {
		const file = makeFile('photo.jpg', 'image/jpeg', 1024);

		mockUploadAvatarRequest.mockResolvedValue({
			data: {
				id: 'usr_1',
				avatar_url: 'https://api.example.com/api/v1/assets/usr_1/avatars/file.jpg'
			} as never,
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const result = await uploadAvatar(file);
		expect(result.success).toBe(true);
		if (result.success) {
			expect(result.objectUrl).toBe('https://api.example.com/api/v1/assets/usr_1/avatars/file.jpg');
		}
		expect(mockUploadAvatarRequest).toHaveBeenCalledWith({ body: { file } });
	});

	it('returns upload_failed when the API rejects the upload', async () => {
		const file = makeFile('photo.png', 'image/png', 1024);

		mockUploadAvatarRequest.mockResolvedValue({
			data: undefined,
			error: { detail: 'storage unavailable' },
			response: new Response(null, { status: 503 })
		} as never);

		const result = await uploadAvatar(file);
		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe('upload_failed');
			if (result.error.code === 'upload_failed') {
				expect(result.error.message).toBe('storage unavailable');
			}
		}
	});
});
