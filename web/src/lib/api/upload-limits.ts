// Configures the generated client (base URL + auth interceptor) before any SDK call.
import '$lib/api/client';
import { uploadLimits } from '$lib/api/generated/sdk.gen';

let cached: Promise<number> | null = null;

/** Server-reported upload cap, or null when it cannot be determined — in
 *  which case callers skip the pre-check and let the server enforce it.
 *  Only a successful lookup is cached, so a transient failure does not
 *  disable client validation for the rest of the session. */
export async function getMaxUploadBytes(): Promise<number | null> {
	if (!cached) {
		cached = uploadLimits().then((response) => {
			const value = response.data?.max_upload_bytes;
			if (typeof value !== 'number' || value <= 0) {
				throw new Error('upload limit unavailable');
			}
			return value;
		});
	}
	try {
		return await cached;
	} catch {
		cached = null;
		return null;
	}
}

/** Drops the memoised limit so tests (and a re-login) start from a clean lookup. */
export function resetUploadLimitsCache(): void {
	cached = null;
}
