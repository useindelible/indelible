/**
 * In-place match highlighting, the way the live search renders it.
 *
 * Returns alternating plain and matched fragments so the caller emits real
 * markup rather than injecting HTML — the query is data, and building a string
 * with it would be an injection waiting to happen.
 */

export interface Fragment {
	text: string;
	match: boolean;
}

export function highlight(text: string, term: string): Fragment[] {
	if (!term) return [{ text, match: false }];

	const out: Fragment[] = [];
	const lower = text.toLowerCase();
	const needle = term.toLowerCase();
	let i = 0;

	for (;;) {
		const j = lower.indexOf(needle, i);
		if (j < 0) {
			if (i < text.length) out.push({ text: text.slice(i), match: false });
			return out;
		}
		if (j > i) out.push({ text: text.slice(i, j), match: false });
		out.push({ text: text.slice(j, j + term.length), match: true });
		i = j + term.length;
	}
}
