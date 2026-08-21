/**
 * Entrance reveals.
 *
 * One IntersectionObserver for the whole page. Elements opt in with
 * `data-rev="<variant>"`; the observer adds `.in` once and stops watching
 * them. Stagger is declared in CSS via `--i`, never scheduled here.
 *
 * There are deliberately no scroll listeners anywhere in this module.
 */

const REVEALED = 'in';

/** Elements already handed to an observer, so re-running init is harmless. */
const seen = new WeakSet<Element>();

export function prefersReducedMotion(): boolean {
	return (
		typeof matchMedia === 'function' &&
		matchMedia('(prefers-reduced-motion: reduce)').matches
	);
}

/**
 * Start revealing everything matching `selector`.
 *
 * The large positive top margin is intentional: content above the fold on a
 * restored scroll position should already be revealed rather than animating
 * in behind the reader.
 *
 * The bottom margin is deliberately 0. A negative bottom margin ("reveal a
 * little after it enters") shrinks the root, and anything sitting below that
 * shrunken edge on a page with no room left to scroll can never intersect —
 * it stays invisible forever. `threshold` already expresses "meaningfully in
 * view" and cannot create that dead zone.
 */
export function initReveal(selector = '[data-rev], .lines'): void {
	const targets = Array.from(document.querySelectorAll(selector)).filter(
		(el) => !seen.has(el),
	);
	if (targets.length === 0) return;

	// No observer, or the reader asked for stillness: show everything at once.
	if (prefersReducedMotion() || !('IntersectionObserver' in window)) {
		for (const el of targets) {
			seen.add(el);
			el.classList.add(REVEALED);
		}
		return;
	}

	const observer = new IntersectionObserver(
		(entries) => {
			for (const entry of entries) {
				if (!entry.isIntersecting) continue;
				entry.target.classList.add(REVEALED);
				observer.unobserve(entry.target);
			}
		},
		{ rootMargin: '10000px 0px 0px 0px', threshold: 0.12 },
	);

	for (const el of targets) {
		seen.add(el);
		observer.observe(el);
	}
}
