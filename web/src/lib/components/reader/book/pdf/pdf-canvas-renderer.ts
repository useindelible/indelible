import type { PDFPageProxy } from 'pdfjs-dist/types/src/display/api';

export function computeScale(
	pageWidth: number,
	containerWidth: number
): { cssScale: number; dpr: number } {
	const dpr = typeof window !== 'undefined' ? window.devicePixelRatio || 1 : 1;
	const cssScale = containerWidth / pageWidth;
	return { cssScale, dpr };
}

export async function renderCanvas(
	page: PDFPageProxy,
	canvas: HTMLCanvasElement,
	cssScale: number,
	dpr: number
): Promise<void> {
	const viewport = page.getViewport({ scale: cssScale });

	canvas.width = Math.floor(viewport.width * dpr);
	canvas.height = Math.floor(viewport.height * dpr);
	canvas.style.width = `${viewport.width}px`;
	canvas.style.height = `${viewport.height}px`;

	const ctx = canvas.getContext('2d');
	if (!ctx) return;
	ctx.setTransform(1, 0, 0, 1, 0, 0);
	ctx.clearRect(0, 0, canvas.width, canvas.height);

	await page.render({
		canvasContext: ctx,
		viewport,
		transform: dpr !== 1 ? [dpr, 0, 0, dpr, 0, 0] : undefined,
		canvas
	} as Parameters<typeof page.render>[0]).promise;
}
