/**
 * Capture sequence trigger.
 *
 * Same engine as the reveals: IntersectionObserver, one shot, no scroll
 * listener. The frame's resting state is the finished one, so skipping this —
 * because the reader prefers reduced motion, or because the script never ran —
 * costs nothing.
 */

import { prefersReducedMotion } from './reveal';

export function initExtensionSequence(selector = '.xt'): void {
	const frames = Array.from(document.querySelectorAll(selector));
	if (frames.length === 0) return;
	if (prefersReducedMotion() || !('IntersectionObserver' in window)) return;

	const observer = new IntersectionObserver(
		(entries) => {
			for (const entry of entries) {
				if (!entry.isIntersecting) continue;
				entry.target.classList.add('xt-play');
				observer.unobserve(entry.target);
			}
		},
		{ threshold: 0.35 },
	);

	for (const frame of frames) observer.observe(frame);
}
