export interface PdfRect {
	x: number;
	y: number;
	width: number;
	height: number;
}

export interface PdfLocator {
	type: 'pdf';
	page: number;
	x: number;
	y: number;
	width: number;
	height: number;
	text_snapshot: string;
	rects?: PdfRect[];
}

export interface PdfSelectionResult {
	text: string;
	rects: PdfRect[];
	x: number;
	y: number;
	width: number;
	height: number;
}

export interface PdfHighlightData {
	id: string;
	color: string;
	locator: PdfLocator;
}

/**
 * Extract the per-line rects from a PDF locator, falling back to the
 * bounding box for legacy single-rect highlights.
 */
export function getHighlightRects(locator: PdfLocator): PdfRect[] {
	if (locator.rects && locator.rects.length > 0) {
		return locator.rects;
	}
	return [{ x: locator.x, y: locator.y, width: locator.width, height: locator.height }];
}

/**
 * Capture multi-rect selection geometry from the text layer.
 * The text layer must be the official pdf.js TextLayer container.
 * Coordinates are normalized to [0,1] relative to the page container.
 */
export function captureSelectionRects(
	textLayerEl: HTMLElement,
	pageContainerEl: HTMLElement
): PdfSelectionResult | null {
	const selection = window.getSelection();
	if (!selection || selection.isCollapsed || selection.rangeCount === 0) return null;

	const range = selection.getRangeAt(0);
	if (!textLayerEl.contains(range.startContainer) || !textLayerEl.contains(range.endContainer)) {
		return null;
	}

	const text = selection.toString().trim();
	if (!text) return null;

	const containerRect = pageContainerEl.getBoundingClientRect();
	if (containerRect.width <= 0 || containerRect.height <= 0) return null;

	const clientRects = Array.from(range.getClientRects()).filter((r) => r.width > 0 && r.height > 0);
	if (clientRects.length === 0) return null;

	const merged = mergeLineRects(clientRects, containerRect);

	const rects: PdfRect[] = merged.map((r) => ({
		x: clamp01(r.x / containerRect.width),
		y: clamp01(r.y / containerRect.height),
		width: clamp01(r.width / containerRect.width),
		height: clamp01(r.height / containerRect.height)
	}));

	const bbox = computeBoundingBox(rects);

	return { text, rects, ...bbox };
}

/**
 * Render highlight overlays into the dedicated highlight layer.
 */
export function renderHighlightOverlays(
	highlightLayerEl: HTMLElement,
	highlights: PdfHighlightData[],
	currentPage: number
): void {
	clearHighlightOverlays(highlightLayerEl);

	for (const hl of highlights) {
		if (hl.color === 'bookmark' || hl.locator.page !== currentPage) continue;

		const hlRects = getHighlightRects(hl.locator);
		for (const rect of hlRects) {
			const overlay = document.createElement('div');
			overlay.className = `pdf-highlight-overlay highlight-${hl.color}`;
			overlay.dataset.highlightId = hl.id;
			overlay.dataset.pdfHighlightOverlay = 'true';
			overlay.style.position = 'absolute';
			overlay.style.left = `${rect.x * 100}%`;
			overlay.style.top = `${rect.y * 100}%`;
			overlay.style.width = `${rect.width * 100}%`;
			overlay.style.height = `${rect.height * 100}%`;
			overlay.style.pointerEvents = 'none';
			highlightLayerEl.appendChild(overlay);
		}
	}
}

export function clearHighlightOverlays(container: HTMLElement): void {
	const overlays = container.querySelectorAll('[data-pdf-highlight-overlay="true"]');
	overlays.forEach((overlay) => overlay.remove());
}

/**
 * Find a highlight overlay element at a given screen point.
 */
export function findHighlightAtPoint(
	highlightLayerEl: HTMLElement,
	clientX: number,
	clientY: number
): HTMLElement | null {
	const overlays = Array.from(
		highlightLayerEl.querySelectorAll<HTMLElement>(
			'[data-pdf-highlight-overlay="true"][data-highlight-id]'
		)
	).reverse();
	for (const overlay of overlays) {
		const rect = overlay.getBoundingClientRect();
		if (
			clientX >= rect.left &&
			clientX <= rect.right &&
			clientY >= rect.top &&
			clientY <= rect.bottom
		) {
			return overlay;
		}
	}
	return null;
}

function clamp01(v: number): number {
	return Math.max(0, Math.min(1, v));
}

function computeBoundingBox(rects: PdfRect[]): {
	x: number;
	y: number;
	width: number;
	height: number;
} {
	if (rects.length === 0) return { x: 0, y: 0, width: 0, height: 0 };
	let minX = Infinity,
		minY = Infinity,
		maxX = -Infinity,
		maxY = -Infinity;
	for (const r of rects) {
		minX = Math.min(minX, r.x);
		minY = Math.min(minY, r.y);
		maxX = Math.max(maxX, r.x + r.width);
		maxY = Math.max(maxY, r.y + r.height);
	}
	return { x: minX, y: minY, width: maxX - minX, height: maxY - minY };
}

/**
 * Merge client rects that overlap vertically (same text line) into
 * single line-level rects. Coordinates are relative to the container.
 */
function mergeLineRects(
	clientRects: DOMRect[],
	containerRect: DOMRect
): Array<{ x: number; y: number; width: number; height: number }> {
	const relative = clientRects.map((r) => ({
		x: Math.max(0, r.left - containerRect.left),
		y: Math.max(0, r.top - containerRect.top),
		width: r.width,
		height: r.height
	}));

	relative.sort((a, b) => a.y - b.y || a.x - b.x);

	const lines: Array<{ x: number; y: number; width: number; height: number }> = [];

	for (const rect of relative) {
		const last = lines[lines.length - 1];
		if (last) {
			const overlapThreshold = Math.min(last.height, rect.height) * 0.5;
			const overlapY =
				Math.min(last.y + last.height, rect.y + rect.height) - Math.max(last.y, rect.y);
			if (overlapY > overlapThreshold) {
				const newX = Math.min(last.x, rect.x);
				const newY = Math.min(last.y, rect.y);
				const newRight = Math.max(last.x + last.width, rect.x + rect.width);
				const newBottom = Math.max(last.y + last.height, rect.y + rect.height);
				last.x = newX;
				last.y = newY;
				last.width = newRight - newX;
				last.height = newBottom - newY;
				continue;
			}
		}
		lines.push({ ...rect });
	}

	return lines;
}
