export interface SourceLocatorPayload {
	type: 'web_page_dom_range';
	url: string;
	location: string;
	offset?: number;
	text_content: string;
	prefix?: string;
	suffix?: string;
}

export interface DomRangeLocation {
	startPath: string;
	startOffset: number;
	endPath: string;
	endOffset: number;
}

export interface TextRun {
	node: Text;
	start: number;
	end: number;
}

export interface TextBoundary {
	node: Text;
	offset: number;
}

export interface TextIndex {
	text: string;
	runs: TextRun[];
}

export type TextNodePredicate = (node: Text) => boolean;

export function parseDomRangeLocation(location: string): DomRangeLocation | undefined {
	const match = /^(.*):(\d+),(.*):(\d+)$/.exec(location);
	if (!match) return undefined;

	return {
		startPath: match[1] ?? '',
		startOffset: Number(match[2]),
		endPath: match[3] ?? '',
		endOffset: Number(match[4])
	};
}

export function buildTextIndex(root: Node, shouldIndexTextNode: TextNodePredicate): TextIndex {
	const doc = root.nodeType === Node.DOCUMENT_NODE ? (root as Document) : root.ownerDocument;
	if (!doc) return { text: '', runs: [] };

	let text = '';
	const runs: TextRun[] = [];
	const walker = doc.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
		acceptNode(node) {
			return shouldIndexTextNode(node as Text)
				? NodeFilter.FILTER_ACCEPT
				: NodeFilter.FILTER_REJECT;
		}
	});

	let node = walker.nextNode();
	while (node) {
		const textNode = node as Text;
		const value = textNode.data;
		if (value.length > 0) {
			const start = text.length;
			text += value;
			runs.push({ node: textNode, start, end: text.length });
		}
		node = walker.nextNode();
	}

	return { text, runs };
}

export function boundaryAt(
	runs: TextRun[],
	offset: number,
	preferEnd: boolean
): TextBoundary | undefined {
	if (runs.length === 0) return undefined;

	for (const run of runs) {
		if (
			offset < run.end ||
			(!preferEnd && offset === run.start) ||
			(preferEnd && offset === run.end)
		) {
			return {
				node: run.node,
				offset: clamp(offset - run.start, 0, run.node.length)
			};
		}
	}

	const last = runs.at(-1);
	if (!last) return undefined;
	return { node: last.node, offset: last.node.length };
}

export interface AnchorContext {
	offset?: number;
	prefix?: string;
	suffix?: string;
}

export interface TextAnchor {
	text: string;
	hint?: { start: number; end: number };
	context?: AnchorContext;
}

export type AnchorResolution =
	| { kind: 'placed'; start: number; end: number; via: 'hint' | 'search' | 'ends' }
	| { kind: 'ambiguous' }
	| { kind: 'missing' };

/** `at` is the index in `text` the dropped token sat before. */
export interface DroppedCitation {
	at: number;
	from: number;
	to: number;
	value: string;
	bracketed: boolean;
}

/** `starts[i]`/`ends[i]` map `text[i]` to a source span; dropped citations fold into the neighbouring span. */
export interface NormalizedText {
	text: string;
	starts: number[];
	ends: number[];
	dropped: DroppedCitation[];
}

const FOLD: Record<string, string> = {
	' ': ' ',
	'’': "'",
	'‘': "'",
	'“': '"',
	'”': '"',
	'–': '-',
	'—': '-'
};
const CITATION_LEAD = new Set(['.', ',', ';', ':', '!', '?', ')', '"', "'"]);
const NO_SPACE_BEFORE = new Set(["'", ',', '.', ';', ':', '!', '?', ')']);
const BRACKET_CITATION = /^\[(\d{1,3}|[a-z]{2,3})\]/;
const BARE_CITATION = /^\d{1,3}(?=$|\s|[.,;:!?)\]])/;
const CHAINED_CITATION = /^\s*(\d{1,3})(?=$|\s|[.,;:!?)\]])/;
const ENDS_MIN_LENGTH = 120;
const ENDS_ANCHOR = 48;
const MAX_HINT_DRIFT = 400;
const MIN_SEPARATION = 64;

export function normalizeForMatch(input: string): NormalizedText {
	const text: string[] = [];
	const starts: number[] = [];
	const ends: number[] = [];
	const dropped: DroppedCitation[] = [];
	let pendingSpaceAt = -1;
	let leadingDropAt = -1;
	const drop = (from: number, to: number, value: string, bracketed: boolean) => {
		dropped.push({ at: text.length, from, to, value, bracketed });
		if (ends.length > 0) ends[ends.length - 1] = to;
		else if (leadingDropAt === -1) leadingDropAt = from;
	};
	let i = 0;
	while (i < input.length) {
		const raw = input[i]!;
		const ch = FOLD[raw] ?? raw;
		if (/\s/.test(ch)) {
			if (pendingSpaceAt === -1) pendingSpaceAt = i;
			i += 1;
			continue;
		}
		const bracket = BRACKET_CITATION.exec(input.slice(i, i + 5));
		if (bracket) {
			i = swallowCitationRun(input, i, i + bracket[0].length, bracket[1]!, true, drop);
			continue;
		}
		const last = text.at(-1);
		const beforeLast = text.at(-2);
		if (
			pendingSpaceAt === -1 &&
			/\d/.test(ch) &&
			last !== undefined &&
			CITATION_LEAD.has(last) &&
			!(beforeLast !== undefined && /\d/.test(beforeLast))
		) {
			const bare = BARE_CITATION.exec(input.slice(i, i + 16));
			if (bare) {
				i = swallowCitationRun(input, i, i + bare[0].length, bare[0], false, drop);
				continue;
			}
		}
		if (pendingSpaceAt !== -1) {
			if (text.length > 0 && !NO_SPACE_BEFORE.has(ch)) {
				text.push(' ');
				starts.push(pendingSpaceAt);
				ends.push(i);
			} else if (ends.length > 0) {
				ends[ends.length - 1] = i;
			}
			pendingSpaceAt = -1;
		}
		text.push(ch);
		starts.push(text.length === 1 && leadingDropAt !== -1 ? leadingDropAt : i);
		ends.push(i + 1);
		i += 1;
	}
	return { text: text.join(''), starts, ends, dropped };
}

function swallowCitationRun(
	input: string,
	from: number,
	firstEnd: number,
	firstValue: string,
	bracketed: boolean,
	drop: (from: number, to: number, value: string, bracketed: boolean) => void
): number {
	drop(from, firstEnd, firstValue, bracketed);
	let at = firstEnd;
	for (;;) {
		const more = CHAINED_CITATION.exec(input.slice(at, at + 16));
		if (!more) break;
		drop(at, at + more[0].length, more[1]!, false);
		at += more[0].length;
	}
	return at;
}

/** Bare numbers that differ on both sides are real values, not renumbered citations. */
function citationsCompatible(
	quote: NormalizedText,
	page: NormalizedText,
	pageStart: number,
	pageEnd: number
): boolean {
	const inSpan = page.dropped.filter((d) => d.at >= pageStart && d.at <= pageEnd);
	const pairs = Math.min(quote.dropped.length, inSpan.length);
	for (let k = 0; k < pairs; k += 1) {
		const q = quote.dropped[k]!;
		const p = inSpan[k]!;
		if (!q.bracketed && !p.bracketed && q.value !== p.value) return false;
	}
	return true;
}

interface Candidate {
	start: number;
	end: number;
	contextHits: number;
	distance: number;
}

function rankCandidates(
	src: NormalizedText,
	starts: number[],
	length: number,
	context: AnchorContext | undefined
): Candidate[] {
	const prefix = normalizeForMatch(context?.prefix ?? '').text.slice(-40);
	const suffix = normalizeForMatch(context?.suffix ?? '').text.slice(0, 40);
	const expected = context?.offset;
	return starts
		.map((start) => {
			const end = start + length;
			const before = src.text.slice(Math.max(0, start - prefix.length - 1), start).trimEnd();
			const after = src.text.slice(end, end + suffix.length + 1).trimStart();
			const contextHits =
				(prefix && before.endsWith(prefix) ? 1 : 0) + (suffix && after.startsWith(suffix) ? 1 : 0);
			const distance =
				expected === undefined ? Infinity : Math.abs((src.starts[start] ?? 0) - expected);
			return { start, end, contextHits, distance };
		})
		.sort((a, b) => b.contextHits - a.contextHits || a.distance - b.distance);
}

function pickUnique(ranked: Candidate[]): Candidate | 'ambiguous' | undefined {
	const best = ranked[0];
	if (!best) return undefined;
	if (ranked.length === 1) return best;
	const second = ranked[1]!;
	if (best.contextHits > second.contextHits) return best;
	if (best.distance <= MAX_HINT_DRIFT && second.distance >= best.distance + MIN_SEPARATION) {
		return best;
	}
	return 'ambiguous';
}

function searchExact(
	src: NormalizedText,
	quote: NormalizedText,
	context: AnchorContext | undefined
): Candidate | 'ambiguous' | undefined {
	const starts = occurrences(src.text, quote.text).filter((at) =>
		citationsCompatible(quote, src, at, at + quote.text.length)
	);
	return pickUnique(rankCandidates(src, starts, quote.text.length, context));
}

function toSource(
	src: NormalizedText,
	quote: NormalizedText,
	start: number,
	end: number
): { start: number; end: number } {
	const leading =
		quote.dropped[0]?.at === 0
			? src.dropped.find(
					(d) => d.at === start || (d.at === start - 1 && src.text[start - 1] === ' ')
				)
			: undefined;
	return { start: leading ? leading.from : src.starts[start]!, end: src.ends[end - 1]! };
}

/** `undefined` for no occurrence and for repeats that no hint separates. */
export function findBestTextMatch(
	source: string,
	needle: string,
	context?: AnchorContext
): { start: number; end: number } | undefined {
	const quote = normalizeForMatch(needle);
	if (!quote.text) return undefined;
	const src = normalizeForMatch(source);
	const found = searchExact(src, quote, context);
	return found && found !== 'ambiguous' ? toSource(src, quote, found.start, found.end) : undefined;
}

export function resolveTextAnchor(sourceText: string, anchor: TextAnchor): AnchorResolution {
	const quote = normalizeForMatch(anchor.text);
	if (!quote.text) return { kind: 'missing' };

	const hint = anchor.hint;
	if (hint && hint.end > hint.start) {
		const found = normalizeForMatch(sourceText.slice(hint.start, hint.end));
		if (found.text === quote.text && citationsCompatible(quote, found, 0, found.text.length)) {
			return { kind: 'placed', start: hint.start, end: hint.end, via: 'hint' };
		}
	}

	const context = {
		...anchor.context,
		offset: anchor.context?.offset ?? hint?.start
	};
	const src = normalizeForMatch(sourceText);
	const exact = searchExact(src, quote, context);
	if (exact === 'ambiguous') return { kind: 'ambiguous' };
	if (exact)
		return { kind: 'placed', ...toSource(src, quote, exact.start, exact.end), via: 'search' };
	const ends = matchByEnds(src, quote, context);
	if (ends === 'ambiguous') return { kind: 'ambiguous' };
	if (ends) return { kind: 'placed', ...toSource(src, quote, ends.start, ends.end), via: 'ends' };
	return { kind: 'missing' };
}

/** Long quotes drift in the middle (dropped markers, reflowed captions); anchor the two ends instead. */
function matchByEnds(
	src: NormalizedText,
	quote: NormalizedText,
	context: AnchorContext
): Candidate | 'ambiguous' | undefined {
	const needle = quote.text;
	if (needle.length < ENDS_MIN_LENGTH) return undefined;
	const head = needle.slice(0, ENDS_ANCHOR);
	const tail = needle.slice(-ENDS_ANCHOR);
	const heads = occurrences(src.text, head);
	const tails = occurrences(src.text, tail);
	const pairs: Array<Candidate & { drift: number }> = [];
	for (const h of heads) {
		for (const t of tails) {
			if (t <= h) continue;
			const end = t + tail.length;
			const ratio = (end - h) / needle.length;
			if (ratio < 0.6 || ratio > 1.5) continue;
			if (!citationsCompatible(quote, src, h, end)) continue;
			const [ranked] = rankCandidates(src, [h], end - h, context);
			pairs.push({ ...ranked!, drift: Math.abs(end - h - needle.length) });
		}
	}
	pairs.sort(
		(a, b) => b.contextHits - a.contextHits || a.distance - b.distance || a.drift - b.drift
	);
	const best = pairs[0];
	if (!best) return undefined;
	const second = pairs[1];
	if (!second || second.contextHits < best.contextHits) return best;
	const separated =
		context.offset === undefined
			? second.drift - best.drift >= needle.length * 0.05
			: second.distance >= best.distance + MIN_SEPARATION;
	return separated ? best : 'ambiguous';
}

function occurrences(haystack: string, needle: string): number[] {
	const found: number[] = [];
	for (let at = haystack.indexOf(needle); at !== -1; at = haystack.indexOf(needle, at + 1)) {
		found.push(at);
	}
	return found;
}

export function normalizeWhitespace(value: string): string {
	return value.replace(/\s+/g, ' ').trim();
}

function clamp(value: number, min: number, max: number): number {
	return Math.min(max, Math.max(min, value));
}
