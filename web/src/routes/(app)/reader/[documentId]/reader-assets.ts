import type { ViewTab } from '$lib/components/reader/ViewTabs.svelte';

export const READER_ASSET_KIND_BY_TAB: Record<ViewTab, string> = {
	reader: 'readable_html',
	original: 'original_html',
	pdf: 'pdf',
	screenshot: 'screenshot'
};

export function shouldLoadReaderAsset(
	tab: ViewTab,
	readerHtmlContent: string,
	assetUrls: Partial<Record<ViewTab, string>>
): boolean {
	if (tab === 'reader') return readerHtmlContent.length === 0;
	return !assetUrls[tab];
}

export function revokeReaderAssetUrls(assetUrls: Partial<Record<ViewTab, string>>): void {
	for (const url of Object.values(assetUrls)) {
		if (url) URL.revokeObjectURL(url);
	}
}
