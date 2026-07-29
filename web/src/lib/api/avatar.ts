import { avatarUploadUrl, updateProfile } from '$lib/api';

const ALLOWED_TYPES = ['image/jpeg', 'image/png', 'image/webp'] as const;
type AllowedType = (typeof ALLOWED_TYPES)[number];

export const MAX_AVATAR_SIZE_BYTES = 2 * 1024 * 1024;

export type AvatarUploadError =
	| { code: 'invalid_type' }
	| { code: 'too_large' }
	| { code: 'upload_failed'; message: string }
	| { code: 'profile_update_failed'; message: string };

function isAllowedType(type: string): type is AllowedType {
	return (ALLOWED_TYPES as readonly string[]).includes(type);
}

/**
 * Validates a file, uploads it to S3 via a presigned URL, and updates the
 * user profile with the resulting stable avatar reference.
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

	const { data: urlData, error: urlError } = await avatarUploadUrl({
		body: { content_type: file.type }
	});

	if (!urlData || urlError) {
		const msg = extractMessage(urlError) ?? 'Failed to get upload URL';
		return { success: false, error: { code: 'upload_failed', message: msg } };
	}

	const putResponse = await fetch(urlData.upload_url, {
		method: 'PUT',
		body: file,
		headers: { 'Content-Type': file.type }
	});

	if (!putResponse.ok) {
		return {
			success: false,
			error: { code: 'upload_failed', message: `PUT returned ${putResponse.status}` }
		};
	}

	const { data: profileData, error: profileError } = await updateProfile({
		body: { avatar_url: urlData.object_url }
	});

	if (!profileData || profileError) {
		const msg = extractMessage(profileError) ?? 'Failed to update profile';
		return { success: false, error: { code: 'profile_update_failed', message: msg } };
	}

	return { success: true, objectUrl: profileData.avatar_url ?? urlData.object_url };
}

function extractMessage(err: unknown): string | undefined {
	if (!err || typeof err !== 'object') return undefined;
	const candidate = err as Record<string, unknown>;
	const val = candidate['detail'] ?? candidate['message'] ?? candidate['error'];
	return typeof val === 'string' ? val : undefined;
}
