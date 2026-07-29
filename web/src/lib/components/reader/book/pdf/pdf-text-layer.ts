import { TextLayer } from 'pdfjs-dist';
import type { PDFPageProxy } from 'pdfjs-dist/types/src/display/api';
import type { PageViewport } from 'pdfjs-dist/types/src/display/display_utils';

export async function renderPdfTextLayer(
	page: PDFPageProxy,
	container: HTMLDivElement,
	viewport: PageViewport
): Promise<{ hasText: boolean; cancel: () => void }> {
	container.replaceChildren();
	container.className = 'textLayer';

	const textContent = await page.getTextContent();

	const textLayer = new TextLayer({
		textContentSource: textContent,
		container,
		viewport
	});

	await textLayer.render();

	const hasText = textLayer.textContentItemsStr.some((s) => s.trim().length > 0);

	container.parentElement?.dispatchEvent(new CustomEvent('pdf-text-layer-rendered'));

	return {
		hasText,
		cancel: () => textLayer.cancel()
	};
}
