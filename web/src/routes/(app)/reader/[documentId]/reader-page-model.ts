import type { DocumentListEntry, DocumentReaderAssetResponse } from '$lib/api';
import type { ViewTab } from '$lib/components/reader/ViewTabs.svelte';
import type { MessageKey, Translate } from '$lib/i18n';

export const READER_VIEW_TABS: ViewTab[] = ['reader', 'original', 'pdf', 'screenshot'];
const REPROCESSABLE_ASSET_STATUSES = new Set(['failed', 'degraded']);
const YOUTUBE_TRANSCRIPT_UNAVAILABLE = 'YouTube transcript unavailable or empty';

export type ReaderFailureKind = 'service' | 'access_or_policy' | 'content' | 'unknown';

export interface ReaderFailurePresentation {
	kind: ReaderFailureKind;
	title: string;
	guidance: string;
	diagnosticId: string;
	attemptedAt: string;
	technicalReason: string | null;
}

const FAILURE_COPY_KEYS: Record<ReaderFailureKind, { title: MessageKey; guidance: MessageKey }> = {
	service: {
		title: 'reader_failure_service_title',
		guidance: 'reader_failure_service_guidance'
	},
	access_or_policy: {
		title: 'reader_failure_access_title',
		guidance: 'reader_failure_access_guidance'
	},
	content: {
		title: 'reader_failure_content_title',
		guidance: 'reader_failure_content_guidance'
	},
	unknown: {
		title: 'reader_failure_unknown_title',
		guidance: 'reader_failure_unknown_guidance'
	}
};

function classifyReaderFailure(reason: string | null | undefined): ReaderFailureKind {
	const normalized = reason?.toLowerCase() ?? '';
	if (
		normalized.includes('page blocked by anti-bot challenge') ||
		normalized.includes('renderer returned http 401') ||
		normalized.includes('renderer returned http 403') ||
		normalized.includes('renderer returned http 451') ||
		normalized.includes('url host is not allowed') ||
		normalized.includes('url resolves to a private or internal address')
	) {
		return 'access_or_policy';
	}
	if (
		normalized.includes('renderer returned no readable_html artifact') ||
		normalized.includes('without visible readable text') ||
		normalized.includes('defuddle produced empty content') ||
		normalized.includes('defuddle produced too little visible readable content')
	) {
		return 'content';
	}
	if (
		normalized.includes('external service error from renderer: error sending request') ||
		normalized.includes('failed to acquire browser page') ||
		normalized.includes('readable_html upload:') ||
		normalized.includes('renderer returned http 502') ||
		normalized.includes('renderer returned http 503') ||
		normalized.includes('renderer returned http 504')
	) {
		return 'service';
	}
	return 'unknown';
}

export function readerFailurePresentation(
	translate: Translate,
	assets: DocumentReaderAssetResponse[]
): ReaderFailurePresentation | null {
	const asset = assets.find(
		(candidate) =>
			candidate.asset_kind === 'readable_html' &&
			REPROCESSABLE_ASSET_STATUSES.has(candidate.status.toLowerCase())
	);
	if (!asset) return null;

	const kind = classifyReaderFailure(asset.failed_reason);
	const copy = FAILURE_COPY_KEYS[kind];
	return {
		kind,
		title: translate(copy.title),
		guidance: translate(copy.guidance),
		diagnosticId: asset.id,
		attemptedAt: asset.created_at,
		technicalReason: asset.failed_reason ?? null
	};
}

export function computeAvailableReaderTabs(assets: DocumentReaderAssetResponse[]): ViewTab[] {
	const tabs: ViewTab[] = [];
	const kinds = new Map(assets.map((asset) => [asset.asset_kind, asset]));
	if (kinds.get('readable_html')?.status === 'completed') tabs.push('reader');
	if (kinds.get('original_html')?.status === 'completed') tabs.push('original');
	if (kinds.get('pdf')?.status === 'completed') tabs.push('pdf');
	if (kinds.get('screenshot')?.status === 'completed') tabs.push('screenshot');
	return tabs;
}

export function isBookReaderItem(item: DocumentListEntry | null): boolean {
	return item != null && (item.item_type === 'book' || item.item_type === 'pdf');
}

export function isTranscriptUnavailableVideo(
	item: DocumentListEntry | null,
	assets: DocumentReaderAssetResponse[]
): boolean {
	return (
		item?.item_type === 'video' &&
		assets.some(
			(asset) =>
				asset.asset_kind === 'extracted_text' &&
				asset.status.toLowerCase() === 'failed' &&
				asset.failed_reason === YOUTUBE_TRANSCRIPT_UNAVAILABLE
		)
	);
}

export function isReadableReady(item: DocumentListEntry | null): boolean {
	return item?.readable_ready ?? item?.available_assets?.includes('readable_html') ?? false;
}

export function isReaderContentReady(
	item: DocumentListEntry | null,
	assets: DocumentReaderAssetResponse[]
): boolean {
	if (assets.length > 0) {
		return assets.some(
			(asset) => asset.asset_kind === 'readable_html' && asset.status.toLowerCase() === 'completed'
		);
	}
	return isReadableReady(item);
}

export function hasFailedReadableAsset(assets: DocumentReaderAssetResponse[]): boolean {
	return assets.some(
		(asset) =>
			asset.asset_kind === 'readable_html' &&
			REPROCESSABLE_ASSET_STATUSES.has(asset.status.toLowerCase())
	);
}

export function isSavedToLibrary(item: DocumentListEntry | null): boolean {
	return item?.saved ?? item?.library_entry_id != null;
}

export function shouldReprocessReaderPreparation(
	item: DocumentListEntry | null,
	assets: DocumentReaderAssetResponse[]
): boolean {
	if (assets.some((asset) => REPROCESSABLE_ASSET_STATUSES.has(asset.status.toLowerCase()))) {
		return true;
	}
	if (item?.pipeline_status?.toLowerCase() === 'failed') {
		return true;
	}
	return !isReaderContentReady(item, assets) && Boolean(item?.pipeline_error);
}

export function computeArticlePdfInitialPage(
	totalChapters: number | null | undefined,
	chapterLocator: string | null | undefined,
	progress: number
): number {
	if (!totalChapters || totalChapters <= 0) return 0;
	if (chapterLocator?.startsWith('page:')) {
		const pageNum = parseInt(chapterLocator.slice(5), 10);
		if (!isNaN(pageNum) && pageNum > 0) {
			return Math.min(pageNum - 1, Math.max(0, totalChapters - 1));
		}
	}
	if (progress <= 0) return 0;
	const totalPages = Math.max(1, totalChapters);
	return Math.min(totalPages - 1, Math.floor((progress / 100) * totalPages));
}
