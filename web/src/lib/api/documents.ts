import { getAccessToken } from '$lib/auth-tokens';

import { resolveDocumentId } from './document-ids';
import * as generated from './generated';
import type { DocumentAssetListResponse } from './compat-types';
import type {
	CreateHighlightBody,
	DocumentReaderAssetResponse,
	DocumentReprocessResponse,
	DocumentUpsertNoteBody,
	UpdateDocumentProgressBody
} from './generated/types.gen';

export async function listAssets(options: { path: { document_id: string } }) {
	const document_id = resolveDocumentId(options.path.document_id);
	const { data: reader } = await generated.getDocumentReader({ path: { document_id } });
	if (reader?.assets?.length) {
		return {
			data: {
				data: reader.assets
			} satisfies DocumentAssetListResponse
		};
	}
	const now = new Date().toISOString();
	const assets: DocumentReaderAssetResponse[] = (reader?.available_assets ?? []).map(
		(asset_kind) => ({
			asset_kind,
			content_type:
				asset_kind === 'readable_html' || asset_kind === 'original_html'
					? 'text/html'
					: 'application/octet-stream',
			created_at: now,
			id: `${document_id}:${asset_kind}`,
			size_bytes: 0,
			status: 'completed'
		})
	);
	return { data: { data: assets } satisfies DocumentAssetListResponse };
}

export async function reprocessDocument(options: {
	path: { document_id: string };
}): Promise<{ data: DocumentReprocessResponse | undefined }> {
	const document_id = resolveDocumentId(options.path.document_id);
	const { data } = await generated.reprocessDocument({ path: { document_id } });
	return { data };
}

export function streamAsset(options: {
	path: { document_id: string; asset_kind: string };
	parseAs: 'blob';
}): Promise<{ data: Blob | undefined }>;
export function streamAsset(options: {
	path: { document_id: string; asset_kind: string };
	parseAs?: 'text';
}): Promise<{ data: string | undefined }>;
export async function streamAsset(options: {
	path: { document_id: string; asset_kind: string };
	parseAs?: 'text' | 'blob';
}) {
	const document_id = resolveDocumentId(options.path.document_id);
	const { data } = await generated.getDocumentAsset({
		path: { document_id, asset_kind: options.path.asset_kind }
	});
	if (!data?.download_url) return { data: undefined };
	// download_url targets the API asset proxy, which requires auth. Bearer
	// beats cookies here: on a presigned-mode 302 to S3 the browser strips the
	// Authorization header for the cross-origin hop, whereas a credentialed
	// fetch would fail the CORS check against S3 (no Allow-Credentials).
	const token = getAccessToken();
	const response = await fetch(data.download_url, {
		credentials: 'same-origin',
		headers: token ? { Authorization: `Bearer ${token}` } : undefined
	});
	if (!response.ok) return { data: undefined };
	return { data: options.parseAs === 'blob' ? await response.blob() : await response.text() };
}

export async function listHighlights(options: { path: { document_id: string } }) {
	const document_id = resolveDocumentId(options.path.document_id);
	return generated.listDocumentHighlights({ path: { document_id } });
}

export async function createHighlight(options: {
	path: { document_id: string };
	body: CreateHighlightBody;
}) {
	const document_id = resolveDocumentId(options.path.document_id);
	return generated.createDocumentHighlight({ path: { document_id }, body: options.body });
}

export async function getDocumentEntryNote(options: { path: { document_id: string } }) {
	const document_id = resolveDocumentId(options.path.document_id);
	return generated.getDocumentNote({ path: { document_id } });
}

export async function upsertDocumentEntryNote(options: {
	path: { document_id: string };
	body: DocumentUpsertNoteBody;
}) {
	const document_id = resolveDocumentId(options.path.document_id);
	return generated.upsertDocumentNote({ path: { document_id }, body: options.body });
}

export async function updateProgress(options: {
	path: { document_id: string };
	body: UpdateDocumentProgressBody;
}) {
	const document_id = resolveDocumentId(options.path.document_id);
	return generated.updateDocumentProgress({ path: { document_id }, body: options.body });
}

export async function markDocumentUnread(options: { path: { document_id: string } }) {
	const document_id = resolveDocumentId(options.path.document_id);
	return generated.markDocumentUnread({ path: { document_id }, throwOnError: true });
}

export async function listDocumentEntities(options: { path: { document_id: string } }) {
	const document_id = resolveDocumentId(options.path.document_id);
	return generated.listDocumentEntities({ path: { document_id } });
}

export async function getEpubToc(options: {
	path: { document_id: string };
	parseAs?: 'json' | 'text';
}) {
	const document_id = resolveDocumentId(options.path.document_id);
	return generated.getEpubToc({
		path: { document_id },
		parseAs: options.parseAs ?? 'json'
	});
}

export async function getEpubChapter(options: {
	path: { document_id: string; chapter_index: number };
	parseAs?: 'json' | 'text';
}) {
	const document_id = resolveDocumentId(options.path.document_id);
	return generated.getEpubChapter({
		path: { document_id, chapter_index: options.path.chapter_index },
		parseAs: options.parseAs ?? 'text'
	});
}

export async function exportHighlights(options: { path: { document_id: string } }) {
	const document_id = resolveDocumentId(options.path.document_id);
	const { data } = await generated.listDocumentHighlights({ path: { document_id } });
	const markdown = (data?.highlights ?? [])
		.map((highlight) => `> ${highlight.text_content}`)
		.join('\n\n');
	return { data: markdown };
}

export type ArticleTocEntry = generated.ArticleTocEntryResponse;
export type ArticleTocStatus = generated.ArticleTocResponseStatus;

export async function getArticleToc(options: {
	path: { document_id: string };
}): Promise<{ data: generated.ArticleTocResponse | undefined }> {
	const document_id = resolveDocumentId(options.path.document_id);
	const { data } = await generated.getArticleToc({ path: { document_id } });
	return { data };
}
