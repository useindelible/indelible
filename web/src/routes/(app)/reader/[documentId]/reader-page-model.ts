import type { DocumentListEntry, DocumentReaderAssetResponse } from '$lib/api';
import type { ViewTab } from '$lib/components/reader/ViewTabs.svelte';

export const READER_VIEW_TABS: ViewTab[] = ['reader', 'original', 'pdf', 'screenshot'];
const REPROCESSABLE_ASSET_STATUSES = new Set(['failed', 'degraded']);

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
