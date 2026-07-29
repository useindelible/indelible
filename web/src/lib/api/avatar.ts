import { uploadAvatar as uploadAvatarRequest } from '$lib/api';

const ALLOWED_TYPES = ['image/jpeg', 'image/png', 'image/webp'] as const;
type AllowedType = (typeof ALLOWED_TYPES)[number];

export const MAX_AVATAR_SIZE_BYTES = 2 * 1024 * 1024;

export type AvatarUploadError =
	| { code: 'invalid_type' }
	| { code: 'too_large' }
	| { code: 'upload_failed'; message: string };

function isAllowedType(type: string): type is AllowedType {
	return (ALLOWED_TYPES as readonly string[]).includes(type);
}

/**
 * Validates a file and uploads it through the API, which stores it and updates
 * the profile in one call.
 *
 * Returns the resolved avatar URL on success, or an AvatarUploadError on failure.
 */
export async function uploadAvatar(
	file: File
): Promise<{ success: true; objectUrl: string } | { success: false; error: AvatarUploadError }> {
	if (!isAllowedType(file.type)) {
		return { success: false, error: { code: 'invalid_type' } };
	}
	if (file.size > MAX_AVATAR_SIZE_BYTES) {
		return { success: false, error: { code: 'too_large' } };
	}

	const { data, error } = await uploadAvatarRequest({ body: { file } });

	if (!data || error) {
		const msg = extractMessage(error) ?? 'Failed to upload avatar';
		return { success: false, error: { code: 'upload_failed', message: msg } };
	}

	return { success: true, objectUrl: data.avatar_url ?? '' };
}

function extractMessage(err: unknown): string | undefined {
	if (!err || typeof err !== 'object') return undefined;
	const candidate = err as Record<string, unknown>;
	const val = candidate['detail'] ?? candidate['message'] ?? candidate['error'];
	return typeof val === 'string' ? val : undefined;
}
