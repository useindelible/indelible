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
	| { kind: 'placed'; start: number; end: number; via: 'hint' | 'search' }
	| { kind: 'ambiguous' }
	| { kind: 'missing' };

/** `starts[i]`/`ends[i]` map `text[i]` to a source span; a dropped citation folds into the span before it. */
export interface NormalizedText {
	text: string;
	starts: number[];
	ends: number[];
}

const FOLD: Record<string, string> = {
	' ': ' ',
	'’': "'",
	'‘': "'",
	'“': '"',
	'”': '"',
	'–': '-',
	'—': '-'
};
const CITATION_LEAD = new Set(['.', ',', ';', ':', '!', '?', ')', '"', "'"]);
const NO_SPACE_BEFORE = new Set(["'", ',', '.', ';', ':', '!', '?', ')']);
const BRACKET_CITATION = /^\[\d{1,3}\]/;
const BARE_CITATION = /^\d{1,3}(?=$|\s|[.,;:!?)\]])/;
const MAX_HINT_DRIFT = 400;
const MIN_SEPARATION = 64;

export function normalizeForMatch(input: string): NormalizedText {
	const text: string[] = [];
	const starts: number[] = [];
	const ends: number[] = [];
	let pendingSpaceAt = -1;
	const extendLast = (to: number) => {
		if (ends.length > 0) ends[ends.length - 1] = to;
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
			extendLast(i + bracket[0].length);
			i += bracket[0].length;
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
				extendLast(i + bare[0].length);
				i += bare[0].length;
				continue;
			}
		}
		if (pendingSpaceAt !== -1) {
			if (text.length > 0 && !NO_SPACE_BEFORE.has(ch)) {
				text.push(' ');
				starts.push(pendingSpaceAt);
				ends.push(i);
			} else {
				extendLast(i);
			}
			pendingSpaceAt = -1;
		}
		text.push(ch);
		starts.push(i);
		ends.push(i + 1);
		i += 1;
	}
	return { text: text.join(''), starts, ends };
}

/** `undefined` for no occurrence and for repeats that no hint separates. */
export function findBestTextMatch(
	source: string,
	needle: string,
	context?: AnchorContext
): { start: number; end: number } | undefined {
	const src = normalizeForMatch(source);
	const ndl = normalizeForMatch(needle).text;
	if (!ndl) return undefined;

	const candidates: number[] = [];
	for (let at = src.text.indexOf(ndl); at !== -1; at = src.text.indexOf(ndl, at + 1)) {
		candidates.push(at);
	}
	if (candidates.length === 0) return undefined;

	const toSource = (start: number) => ({
		start: src.starts[start]!,
		end: src.ends[start + ndl.length - 1]!
	});
	if (candidates.length === 1) return toSource(candidates[0]!);

	const prefix = normalizeForMatch(context?.prefix ?? '').text.slice(-40);
	const suffix = normalizeForMatch(context?.suffix ?? '').text.slice(0, 40);
	const expected = context?.offset;
	const ranked = candidates
		.map((start) => {
			const end = start + ndl.length;
			const before = src.text.slice(Math.max(0, start - prefix.length - 1), start).trimEnd();
			const after = src.text.slice(end, end + suffix.length + 1).trimStart();
			const contextHits =
				(prefix && before.endsWith(prefix) ? 1 : 0) + (suffix && after.startsWith(suffix) ? 1 : 0);
			const distance =
				expected === undefined ? Infinity : Math.abs((src.starts[start] ?? 0) - expected);
			return { start, contextHits, distance };
		})
		.sort((a, b) => b.contextHits - a.contextHits || a.distance - b.distance);
	const best = ranked[0]!;
	const second = ranked[1]!;
	if (best.contextHits > second.contextHits) return toSource(best.start);
	if (best.distance <= MAX_HINT_DRIFT && second.distance >= best.distance + MIN_SEPARATION) {
		return toSource(best.start);
	}
	return undefined;
}

export function resolveTextAnchor(sourceText: string, anchor: TextAnchor): AnchorResolution {
	const expected = normalizeForMatch(anchor.text).text;
	if (!expected) return { kind: 'missing' };

	const hint = anchor.hint;
	if (hint && hint.end > hint.start) {
		const found = normalizeForMatch(sourceText.slice(hint.start, hint.end)).text;
		if (found === expected)
			return { kind: 'placed', start: hint.start, end: hint.end, via: 'hint' };
	}

	const context = {
		...anchor.context,
		offset: anchor.context?.offset ?? hint?.start
	};
	const match = findBestTextMatch(sourceText, anchor.text, context);
	if (match) return { kind: 'placed', ...match, via: 'search' };
	return normalizeForMatch(sourceText).text.includes(expected)
		? { kind: 'ambiguous' }
		: { kind: 'missing' };
}

export function normalizeWhitespace(value: string): string {
	return value.replace(/\s+/g, ' ').trim();
}

function clamp(value: number, min: number, max: number): number {
	return Math.min(max, Math.max(min, value));
}
