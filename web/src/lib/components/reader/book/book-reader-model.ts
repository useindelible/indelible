import type { DocumentListEntry, DocumentReaderAssetResponse } from '$lib/api';

const IMAGE_ONLY_PDF_REASON = 'PDF text extraction produced no text';

export function isImageOnlyPdf(
	item: DocumentListEntry,
	assets: DocumentReaderAssetResponse[]
): boolean {
	return (
		item.item_type === 'pdf' &&
		assets.some(
			(asset) =>
				asset.asset_kind === 'extracted_text' &&
				asset.status === 'failed' &&
				asset.failed_reason === IMAGE_ONLY_PDF_REASON
		)
	);
}
