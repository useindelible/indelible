import type { OpmlImportResponse } from '$lib/api';
import { getAccessToken } from '$lib/auth-tokens';

export type OpmlUploadResult =
	{ ok: true; data: OpmlImportResponse } | { ok: false; error: string };

export async function uploadOpml(file: File): Promise<OpmlUploadResult> {
	const baseUrl =
		import.meta.env.VITE_API_BASE_URL?.trim() ||
		(import.meta.env.DEV ? `http://${window.location.hostname}:38473` : '');
	const token = getAccessToken();
	const fd = new FormData();
	fd.append('file', file);

	const resp = await fetch(`${baseUrl}/api/v1/feeds/subscriptions/opml`, {
		method: 'POST',
		headers: token ? { Authorization: `Bearer ${token}` } : {},
		credentials: 'include',
		body: fd
	});

	if (!resp.ok) {
		const body = await resp.json().catch(() => null);
		const problem = body as Record<string, unknown> | null;
		const errors = Array.isArray(problem?.errors) ? problem.errors : [];
		const firstError = errors[0] as Record<string, unknown> | undefined;
		const detail =
			(resp.status === 422 && typeof firstError?.message === 'string'
				? firstError.message
				: undefined) ??
			(typeof problem?.detail === 'string' ? problem.detail : undefined) ??
			(typeof problem?.message === 'string' ? problem.message : undefined) ??
			`Upload failed (${resp.status})`;
		if (resp.status === 422) {
			return {
				ok: false,
				error: `${file.name}: ${detail} — Choose a valid OPML file and try again.`
			};
		}
		return { ok: false, error: detail };
	}

	const data = (await resp.json()) as OpmlImportResponse;
	return { ok: true, data };
}
