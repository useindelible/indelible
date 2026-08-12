import type { DocumentListEntry, DocumentReaderAssetResponse } from '$lib/api';

const IMAGE_ONLY_PDF_REASONS = [
	'PDF text extraction produced no text',
	'PDF text extraction failed: failed to extract text from PDF: no extractable text'
];

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
				IMAGE_ONLY_PDF_REASONS.includes(asset.failed_reason ?? '')
		)
	);
}
