import {
	getImport,
	listImports,
	rollbackImport,
	type ImportJobStatusResponse,
	type ImportUploadResponse
} from '$lib/api';
import { getAccessToken } from '$lib/auth-tokens';
import { get } from 'svelte/store';
import { t } from '$lib/i18n';

type ApiProblem = {
	detail?: string;
	error?: string;
	message?: string;
};

export type ApiResult<T> = { success: true; data: T } | { success: false; error: string };

export type ImportJobError = {
	status: 'error';
	httpStatus: number;
	message: string;
};

export type ImportJobLookupResult =
	{ status: 'ok'; data: ImportJobStatusResponse } | { status: 'not_found' } | ImportJobError;

export type ReadwiseImportFiles = {
	libraryCsv?: File | null;
	archiveZip?: File | null;
	feedsOpml?: File | null;
};

function extractMessage(problem: unknown, fallback: string): string {
	if (!problem || typeof problem !== 'object') {
		return fallback;
	}
	const candidate = problem as ApiProblem;
	return candidate.detail ?? candidate.message ?? candidate.error ?? fallback;
}

export async function fetchImportJob(jobId: string): Promise<ImportJobLookupResult> {
	try {
		const { data, error, response } = await getImport({ path: { slug: jobId } });
		if (data) {
			return { status: 'ok', data };
		}
		if (response?.status === 404) {
			return { status: 'not_found' };
		}
		return {
			status: 'error',
			httpStatus: response?.status ?? 0,
			message: extractMessage(error, 'Failed to load import job')
		};
	} catch {
		return { status: 'error', httpStatus: 0, message: get(t)('auth_error_unexpected') };
	}
}

export async function fetchRecentImports(
	limit = 25
): Promise<ApiResult<ImportJobStatusResponse[]>> {
	try {
		const { data, error, response } = await listImports({ query: { limit } });
		if (data) {
			return { success: true, data: data.jobs };
		}
		return {
			success: false,
			error: extractMessage(
				error,
				`Failed to load import history (${response?.status ?? 'network error'})`
			)
		};
	} catch {
		return { success: false, error: get(t)('auth_error_unexpected') };
	}
}

export async function rollbackImportJob(jobId: string): Promise<ApiResult<void>> {
	try {
		const { error, response } = await rollbackImport({ path: { slug: jobId } });
		if (response?.ok) {
			return { success: true, data: undefined };
		}
		return { success: false, error: extractMessage(error, 'Failed to roll back import') };
	} catch {
		return { success: false, error: get(t)('auth_error_unexpected') };
	}
}

// Multipart uploads bypass the generated SDK because openapi-fetch's body serializer
// runs JSON.stringify on FormData.
export async function uploadImportFile(
	slug: string,
	file: File
): Promise<ApiResult<ImportUploadResponse>> {
	const baseUrl =
		import.meta.env.VITE_API_BASE_URL?.trim() ||
		(import.meta.env.DEV ? `http://${window.location.hostname}:38473` : '');
	const token = getAccessToken();
	const formData = new FormData();
	formData.append('file', file);

	try {
		const response = await fetch(`${baseUrl}/api/v1/imports/${encodeURIComponent(slug)}`, {
			method: 'POST',
			headers: token ? { Authorization: `Bearer ${token}` } : {},
			credentials: 'include',
			body: formData
		});

		if (!response.ok) {
			const body = (await response.json().catch(() => null)) as ApiProblem | null;
			return {
				success: false,
				error: extractMessage(body, `Upload failed (${response.status})`)
			};
		}

		const data = (await response.json()) as ImportUploadResponse;
		return { success: true, data };
	} catch {
		return { success: false, error: get(t)('auth_error_unexpected') };
	}
}

const READWISE_CSV_HEADERS = [
	'title',
	'url',
	'id',
	'document tags',
	'saved date',
	'reading progress',
	'location',
	'seen'
] as const;

export function validateReadwiseCsv(text: string): string | null {
	const firstLine = text.split('\n')[0]?.trim() ?? '';
	const cols = firstLine.split(',').map((c) => c.replace(/^"|"$/g, '').trim().toLowerCase());
	const mismatch = READWISE_CSV_HEADERS.find((expected, i) => cols[i] !== expected);
	if (mismatch) {
		return `Not a Readwise export CSV — missing or unexpected column "${mismatch}". Download your library CSV from Readwise Reader and try again.`;
	}
	return null;
}

export function countCsvRows(text: string): number {
	const lines = text.split('\n');
	// subtract header row; ignore trailing empty line
	const dataLines = lines.slice(1).filter((l) => l.trim().length > 0);
	return dataLines.length;
}

export function countOpmlFeeds(text: string): number {
	// Real-world OPML exports (Readwise included) often contain unescaped `&`
	// in feed titles, which makes them invalid XML and yields a parser error
	// from DOMParser. Count outline elements directly so we still report a
	// useful number when the export is technically broken.
	const matches = text.match(/<outline\b[^>]*\btype\s*=\s*"rss"[^>]*>/gi);
	return matches?.length ?? 0;
}

export async function uploadReadwiseImportFiles(
	files: ReadwiseImportFiles
): Promise<ApiResult<ImportUploadResponse>> {
	const baseUrl =
		import.meta.env.VITE_API_BASE_URL?.trim() ||
		(import.meta.env.DEV ? `http://${window.location.hostname}:38473` : '');
	const token = getAccessToken();
	const formData = new FormData();

	if (files.libraryCsv) formData.append('library_csv', files.libraryCsv);
	if (files.archiveZip) formData.append('archive_zip', files.archiveZip);
	if (files.feedsOpml) formData.append('feeds_opml', files.feedsOpml);

	try {
		const response = await fetch(`${baseUrl}/api/v1/imports/readwise`, {
			method: 'POST',
			headers: token ? { Authorization: `Bearer ${token}` } : {},
			credentials: 'include',
			body: formData
		});

		if (!response.ok) {
			const body = (await response.json().catch(() => null)) as ApiProblem | null;
			return {
				success: false,
				error: extractMessage(body, `Upload failed (${response.status})`)
			};
		}

		const data = (await response.json()) as ImportUploadResponse;
		return { success: true, data };
	} catch {
		return { success: false, error: get(t)('auth_error_unexpected') };
	}
}
