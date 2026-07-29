import { describe, it, expect, vi, beforeEach } from 'vitest';
import { uploadAvatar, MAX_AVATAR_SIZE_BYTES } from '$lib/api/avatar';

vi.mock('$lib/api', () => ({
	avatarUploadUrl: vi.fn(),
	updateProfile: vi.fn()
}));

import { avatarUploadUrl, updateProfile } from '$lib/api';

const mockAvatarUploadUrl = vi.mocked(avatarUploadUrl);
const mockUpdateProfile = vi.mocked(updateProfile);

function makeFile(name: string, type: string, sizeBytes: number): File {
	const content = new Uint8Array(sizeBytes).fill(0);
	return new File([content], name, { type });
}

describe('uploadAvatar', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		globalThis.fetch = vi.fn();
	});

	it('rejects unsupported image types', async () => {
		const file = makeFile('image.gif', 'image/gif', 100);
		const result = await uploadAvatar(file);
		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe('invalid_type');
		}
		expect(mockAvatarUploadUrl).not.toHaveBeenCalled();
	});

	it('rejects files over the size limit', async () => {
		const file = makeFile('big.png', 'image/png', MAX_AVATAR_SIZE_BYTES + 1);
		const result = await uploadAvatar(file);
		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe('too_large');
		}
		expect(mockAvatarUploadUrl).not.toHaveBeenCalled();
	});

	it('accepts jpeg files within the size limit', async () => {
		const file = makeFile('photo.jpg', 'image/jpeg', 1024);

		mockAvatarUploadUrl.mockResolvedValue({
			data: {
				upload_url: 'https://s3.example.com/put-here',
				object_url: 'usr_1/avatars/file.jpg',
				expires_at: '2026-03-23T12:05:00Z'
			},
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		vi.mocked(globalThis.fetch).mockResolvedValue(new Response(null, { status: 200 }));

		mockUpdateProfile.mockResolvedValue({
			data: { id: 'usr_1', avatar_url: 'https://cdn.example.com/avatars/usr_1/file.jpg' } as never,
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const result = await uploadAvatar(file);
		expect(result.success).toBe(true);
		if (result.success) {
			expect(result.objectUrl).toBe('https://cdn.example.com/avatars/usr_1/file.jpg');
		}
		expect(mockUpdateProfile).toHaveBeenCalledWith({
			body: { avatar_url: 'usr_1/avatars/file.jpg' }
		});
	});

	it('returns upload_failed when presigned URL request fails', async () => {
		const file = makeFile('photo.png', 'image/png', 1024);

		mockAvatarUploadUrl.mockResolvedValue({
			data: undefined,
			error: { detail: 'storage unavailable' },
			response: new Response(null, { status: 503 })
		} as never);

		const result = await uploadAvatar(file);
		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe('upload_failed');
		}
	});

	it('returns upload_failed when PUT to S3 fails', async () => {
		const file = makeFile('photo.webp', 'image/webp', 512);

		mockAvatarUploadUrl.mockResolvedValue({
			data: {
				upload_url: 'https://s3.example.com/put-here',
				object_url: 'usr_1/avatars/file.webp',
				expires_at: '2026-03-23T12:05:00Z'
			},
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		vi.mocked(globalThis.fetch).mockResolvedValue(new Response(null, { status: 403 }));

		const result = await uploadAvatar(file);
		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe('upload_failed');
			if (result.error.code === 'upload_failed') {
				expect(result.error.message).toContain('403');
			}
		}
	});

	it('returns profile_update_failed when PATCH /me fails', async () => {
		const file = makeFile('photo.png', 'image/png', 1024);

		mockAvatarUploadUrl.mockResolvedValue({
			data: {
				upload_url: 'https://s3.example.com/put-here',
				object_url: 'usr_1/avatars/file.png',
				expires_at: '2026-03-23T12:05:00Z'
			},
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		vi.mocked(globalThis.fetch).mockResolvedValue(new Response(null, { status: 200 }));

		mockUpdateProfile.mockResolvedValue({
			data: undefined,
			error: { detail: 'auth required' },
			response: new Response(null, { status: 401 })
		} as never);

		const result = await uploadAvatar(file);
		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe('profile_update_failed');
		}
	});

	it('sends PUT request with correct Content-Type header', async () => {
		const file = makeFile('photo.png', 'image/png', 256);

		mockAvatarUploadUrl.mockResolvedValue({
			data: {
				upload_url: 'https://s3.example.com/put-here',
				object_url: 'usr_1/avatars/file.png',
				expires_at: '2026-03-23T12:05:00Z'
			},
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const fetchMock = vi
			.mocked(globalThis.fetch)
			.mockResolvedValue(new Response(null, { status: 200 }));

		mockUpdateProfile.mockResolvedValue({
			data: { id: 'usr_1', avatar_url: 'https://cdn.example.com/avatars/usr_1/file.png' } as never,
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		await uploadAvatar(file);

		expect(fetchMock).toHaveBeenCalledWith(
			'https://s3.example.com/put-here',
			expect.objectContaining({
				method: 'PUT',
				headers: { 'Content-Type': 'image/png' }
			})
		);
	});
});
