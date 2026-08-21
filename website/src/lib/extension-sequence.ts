/**
 * Capture sequence trigger.
 *
 * Same engine as the reveals: IntersectionObserver, one shot, no scroll
 * listener. The frame's resting state is the finished one, so skipping this —
 * because the reader prefers reduced motion, or because the script never ran —
 * costs nothing.
 *
 * Two steps, and the order matters. ARMING attaches the animations paused at
 * their first frame, and happens the moment this runs, long before the frame
 * is scrolled to. PLAYING releases them. Doing both at once let the finished
 * bar show until the observer fired and then snap shut, which read as a flash
 * of the extension row before the sequence that introduces it.
 */

import { prefersReducedMotion } from './reveal';

export function initExtensionSequence(selector = '.xt'): void {
	const frames = Array.from(document.querySelectorAll(selector));
	if (frames.length === 0) return;
	if (prefersReducedMotion() || !('IntersectionObserver' in window)) return;

	// Arm first: from here the frame shows its pre-save state, not its finished
	// one, so there is nothing to snap away when the sequence starts.
	for (const frame of frames) frame.classList.add('xt-armed');

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
