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

export function findBestTextMatch(
	source: string,
	needle: string,
	locator?: Pick<SourceLocatorPayload, 'offset' | 'prefix' | 'suffix'>
): { start: number; end: number } | undefined {
	const normalizedNeedle = normalizeWhitespace(needle);
	if (!normalizedNeedle) return undefined;

	const starts = findCandidateMatches(source, normalizedNeedle);
	if (starts.length === 0) return undefined;

	const expected = locator?.offset;
	let best = starts[0];
	if (!best) return undefined;
	let bestScore = Number.POSITIVE_INFINITY;

	for (const match of starts) {
		let score = expected === undefined ? 0 : Math.abs(match.start - expected);
		if (contextMatches(source, match.start, locator?.prefix, true)) score -= 1000;
		if (contextMatches(source, match.end, locator?.suffix, false)) score -= 1000;
		if (score < bestScore) {
			best = match;
			bestScore = score;
		}
	}

	return best;
}

export function normalizeWhitespace(value: string): string {
	return value.replace(/\s+/g, ' ').trim();
}

function findCandidateMatches(
	source: string,
	normalizedNeedle: string
): Array<{
	start: number;
	end: number;
}> {
	const pattern = escapeRegExp(normalizedNeedle).replace(/\s+/g, '\\s+');
	const regex = new RegExp(pattern, 'g');
	const matches: Array<{ start: number; end: number }> = [];

	let match = regex.exec(source);
	while (match) {
		matches.push({ start: match.index, end: match.index + match[0].length });
		if (match[0].length === 0) regex.lastIndex += 1;
		match = regex.exec(source);
	}

	return matches;
}

function contextMatches(
	source: string,
	offset: number,
	context: string | undefined,
	before: boolean
): boolean {
	const normalized = normalizeWhitespace(context ?? '');
	if (!normalized) return false;

	const sample = before
		? normalizeWhitespace(source.slice(Math.max(0, offset - normalized.length - 40), offset))
		: normalizeWhitespace(source.slice(offset, Math.min(source.length, offset + normalized.length + 40)));

	return before ? sample.endsWith(normalized.slice(-40)) : sample.startsWith(normalized.slice(0, 40));
}

function escapeRegExp(value: string): string {
	return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function clamp(value: number, min: number, max: number): number {
	return Math.min(max, Math.max(min, value));
}
