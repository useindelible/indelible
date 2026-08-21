/**
 * Count-up numerals.
 *
 * An element with `data-count="64"` starts at 0 and eases to 64 the first
 * time it is substantially on screen. Reduced motion writes the final value
 * immediately — the number is information, so it is never withheld.
 */

import { prefersReducedMotion } from './reveal';

const DURATION_MS = 1100;

function easeOutCubic(t: number): number {
	return 1 - Math.pow(1 - t, 3);
}

function runCount(el: HTMLElement, end: number): void {
	let start: number | null = null;

	function frame(now: number) {
		if (start === null) start = now;
		const t = Math.min((now - start) / DURATION_MS, 1);
		el.textContent = String(Math.round(end * easeOutCubic(t)));
		if (t < 1) requestAnimationFrame(frame);
	}

	requestAnimationFrame(frame);
}

export function initCounters(selector = '[data-count]'): void {
	const targets = Array.from(
		document.querySelectorAll<HTMLElement>(selector),
	).filter((el) => Number.isFinite(Number(el.dataset.count)));
	if (targets.length === 0) return;

	if (prefersReducedMotion() || !('IntersectionObserver' in window)) {
		for (const el of targets) el.textContent = el.dataset.count ?? '';
		return;
	}

	const observer = new IntersectionObserver(
		(entries) => {
			for (const entry of entries) {
				if (!entry.isIntersecting) continue;
				observer.unobserve(entry.target);
				const el = entry.target as HTMLElement;
				runCount(el, Number(el.dataset.count));
			}
		},
		{ threshold: 0.6 },
	);

	for (const el of targets) observer.observe(el);
}
