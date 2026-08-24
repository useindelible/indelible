import { getAccessToken } from '$lib/auth-tokens';
import { getApiBaseUrl } from '$lib/api/client';
import { getMaxUploadBytes, resetUploadLimitsCache } from '$lib/api/upload-limits';
import type { LibraryEntryResponse } from '$lib/api/generated/types.gen';
import { formatMegabytes } from '$lib/format/megabytes';
import { get } from 'svelte/store';
import { t } from '$lib/i18n';

type ApiProblem = {
	detail?: string;
	error?: string;
	message?: string;
	errors?: Array<{ message?: string }>;
};

export type UploadProgress = {
	loaded: number;
	total: number;
	percent: number;
};

export type UploadFileResult =
	{ success: true; data: LibraryEntryResponse } | { success: false; error: string };

function extractMessage(problem: unknown, fallback: string): string {
	if (!problem || typeof problem !== 'object') {
		return fallback;
	}
	const candidate = problem as ApiProblem;
	const fieldMessage = candidate.errors?.find((error) => error.message?.trim())?.message;
	return fieldMessage ?? candidate.detail ?? candidate.message ?? candidate.error ?? fallback;
}

export function uploadLibraryFile(
	file: File,
	onProgress?: (progress: UploadProgress) => void
): Promise<UploadFileResult> {
	const formData = new FormData();
	formData.append('file', file);

	return new Promise((resolve) => {
		const xhr = new XMLHttpRequest();
		xhr.open('POST', `${getApiBaseUrl()}/api/v1/library/uploads`);
		xhr.withCredentials = true;

		const token = getAccessToken();
		if (token) {
			xhr.setRequestHeader('Authorization', `Bearer ${token}`);
		}

		xhr.upload.onprogress = (event) => {
			if (!event.lengthComputable) return;
			onProgress?.({
				loaded: event.loaded,
				total: event.total,
				percent: Math.round((event.loaded / event.total) * 100)
			});
		};

		xhr.onload = async () => {
			// A 413 can still reach us: the limit lookup failed, the server lowered it
			// after the page loaded, or the body tripped axum's own DefaultBodyLimit —
			// whose plain-text response would otherwise render as "Upload failed (413)".
			if (xhr.status === 413) {
				// The 413 proves any memoised limit is stale or missing, so drop it before re-asking.
				resetUploadLimitsCache();
				// A rejection here would hang the upload promise forever, so degrade to the generic message.
				const limit = await getMaxUploadBytes().catch(() => null);
				resolve({
					success: false,
					error:
						limit !== null
							? get(t)('library_upload_too_large', { values: { size: formatMegabytes(limit) } })
							: get(t)('library_upload_too_large_generic')
				});
				return;
			}

			const body = parseJson(xhr.responseText);
			if (xhr.status >= 200 && xhr.status < 300) {
				resolve({ success: true, data: body as LibraryEntryResponse });
				return;
			}
			resolve({
				success: false,
				error: extractMessage(body, `Upload failed (${xhr.status})`)
			});
		};

		xhr.onerror = () => {
			resolve({ success: false, error: get(t)('library_upload_network_error') });
		};

		xhr.send(formData);
	});
}

function parseJson(raw: string): unknown {
	try {
		return raw ? JSON.parse(raw) : null;
	} catch {
		return null;
	}
}
