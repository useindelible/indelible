/**
 * Reveal attributes for markup.
 *
 * Spread onto any element instead of wrapping it:
 *
 *   <div {...reveal('up', 2)}>…</div>
 *
 * Keeping this a prop helper rather than a <Reveal> component means the
 * motion system never inserts a div of its own, so it cannot disturb a grid
 * or flex layout it is applied inside.
 */

export type RevealVariant =
	| 'up'
	| 'fade'
	| 'scale'
	| 'left'
	| 'right'
	| 'clip'
	| 'pop'
	| 'group';

export interface RevealAttrs {
	'data-rev': RevealVariant;
	style?: string;
}

/** @param index stagger position; each step adds 68ms (see motion.css). */
export function reveal(variant: RevealVariant = 'up', index = 0): RevealAttrs {
	const attrs: RevealAttrs = { 'data-rev': variant };
	if (index) attrs.style = `--i:${index}`;
	return attrs;
}
