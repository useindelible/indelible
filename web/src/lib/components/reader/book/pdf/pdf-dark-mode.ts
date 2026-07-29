export type PdfThemeMode = 'light' | 'dark' | 'sepia';

/**
 * Apply a theme-specific LUT pixel remap to the PDF canvas.
 *
 * - dark:  white (#fff) → #1c1c1e, black → #fff (inverted, gamma-curved)
 * - sepia: white (#fff) → #f5edda, black → #5b4636 (warm tint)
 * - light: no-op
 *
 * We use pixel remapping rather than CSS filters because filters
 * can't target specific colors (e.g. invert() maps white to #000,
 * not to the app's dark background).
 */
export function applyThemeRemap(canvas: HTMLCanvasElement, mode: PdfThemeMode): void {
	if (mode === 'light') return;

	const ctx = canvas.getContext('2d');
	if (!ctx) return;
	const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
	const data = imageData.data;

	const lutR = new Uint8Array(256);
	const lutG = new Uint8Array(256);
	const lutB = new Uint8Array(256);

	if (mode === 'dark') {
		const bg = 28; // #1c1c1e
		const fg = 255;
		for (let i = 0; i < 256; i++) {
			const norm = i / 255; // 0 = black text, 1 = white page
			const curved = Math.pow(norm, 0.6);
			const v = (fg - (fg - bg) * curved) | 0;
			lutR[i] = v;
			lutG[i] = v;
			lutB[i] = v;
		}
	} else {
		// sepia: black text (#000) → #5b4636, white page (#fff) → #f5edda
		const bgR = 245,
			bgG = 237,
			bgB = 218; // #f5edda
		const fgR = 91,
			fgG = 70,
			fgB = 54; // #5b4636
		for (let i = 0; i < 256; i++) {
			const norm = i / 255; // 0 = black text, 1 = white page
			lutR[i] = (fgR + (bgR - fgR) * norm) | 0;
			lutG[i] = (fgG + (bgG - fgG) * norm) | 0;
			lutB[i] = (fgB + (bgB - fgB) * norm) | 0;
		}
	}

	for (let i = 0; i < data.length; i += 4) {
		data[i] = lutR[data[i]!]!;
		data[i + 1] = lutG[data[i + 1]!]!;
		data[i + 2] = lutB[data[i + 2]!]!;
	}

	ctx.putImageData(imageData, 0, 0);
}
