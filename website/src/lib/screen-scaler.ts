/**
 * Product screens are authored at a fixed design size and scaled to whatever
 * column they land in.
 *
 * Authoring at 1366x900 (or 390x844 for phones) means every screen's internals
 * can use real pixel values taken from the app, instead of each one inventing
 * its own responsive rules. A ResizeObserver — not a resize or scroll handler —
 * keeps the transform in step with the container.
 */

export const DESIGN_WIDTH = { desk: 1366, phone: 390 } as const;

export type ScreenDevice = keyof typeof DESIGN_WIDTH;

function fit(frame: HTMLElement): void {
	const inner = frame.querySelector<HTMLElement>('[data-screen-inner]');
	if (!inner) return;

	const device = (frame.dataset.d ?? 'desk') as ScreenDevice;
	const design = DESIGN_WIDTH[device] ?? DESIGN_WIDTH.desk;
	inner.style.transform = `scale(${frame.clientWidth / design})`;
}

export function initScreenScaler(selector = '[data-screen]'): void {
	const frames = Array.from(document.querySelectorAll<HTMLElement>(selector));
	if (frames.length === 0) return;

	if (!('ResizeObserver' in window)) {
		// Older engines get a correct first paint; they simply do not re-fit.
		for (const frame of frames) fit(frame);
		return;
	}

	const observer = new ResizeObserver((entries) => {
		for (const entry of entries) fit(entry.target as HTMLElement);
	});

	for (const frame of frames) {
		observer.observe(frame);
		fit(frame);
	}
}
