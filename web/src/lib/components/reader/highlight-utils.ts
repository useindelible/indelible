import { boundaryAt, buildTextIndex, type TextRun } from '../../../../../shared/highlight-source';

export interface SelectionOffsets {
	text: string;
	startOffset: number;
	endOffset: number;
}

/**
 * Calculates character offsets of the current selection relative to the
 * text content of a container element. Walks the DOM tree to find the
 * character position within the container's flattened text.
 */
export function getSelectionOffsets(containerEl: HTMLElement): SelectionOffsets | null {
	const selection = window.getSelection();
	if (!selection || selection.isCollapsed || selection.rangeCount === 0) return null;

	const range = selection.getRangeAt(0);
	if (!containerEl.contains(range.startContainer) || !containerEl.contains(range.endContainer)) {
		return null;
	}

	const text = selection.toString().trim();
	if (!text) return null;

	const startOffset = getCharOffset(containerEl, range.startContainer, range.startOffset);
	const endOffset = getCharOffset(containerEl, range.endContainer, range.endOffset);

	if (startOffset === -1 || endOffset === -1) return null;

	return { text, startOffset, endOffset };
}

function shouldIndexReaderTextNode(node: Text): boolean {
	return node.parentElement?.closest('.hl-body-tag') === null;
}

function getCharOffset(container: HTMLElement, node: Node, offset: number): number {
	const index = buildTextIndex(container, shouldIndexReaderTextNode);
	const run = index.runs.find((candidate) => candidate.node === node);
	if (run) return run.start + Math.min(offset, run.node.length);

	return -1;
}

export interface HighlightRange {
	id: string;
	color: string;
	startOffset: number;
	endOffset: number;
}

/**
 * Applies highlight marks to text nodes within a container based on
 * character offsets. Clears existing highlights first, then wraps
 * matching ranges in <mark> elements.
 */
export function applyHighlights(containerEl: HTMLElement, highlights: HighlightRange[]): void {
	clearHighlights(containerEl);

	if (highlights.length === 0) return;

	const sorted = [...highlights].sort((a, b) => a.startOffset - b.startOffset);

	for (const hl of sorted) {
		wrapRange(containerEl, hl);
	}
}

function clearHighlights(containerEl: HTMLElement): void {
	const marks = containerEl.querySelectorAll('mark[data-highlight-id]');
	marks.forEach((mark) => {
		const parent = mark.parentNode;
		if (!parent) return;
		while (mark.firstChild) {
			parent.insertBefore(mark.firstChild, mark);
		}
		parent.removeChild(mark);
		parent.normalize();
	});
}

function wrapRange(containerEl: HTMLElement, hl: HighlightRange): void {
	const index = buildTextIndex(containerEl, shouldIndexReaderTextNode);
	const nodesToWrap = runsInRange(index.runs, hl.startOffset, hl.endOffset);

	for (const { node, start, end } of nodesToWrap) {
		const mark = document.createElement('mark');
		mark.className = `highlight-${hl.color}`;
		mark.dataset.highlightId = hl.id;
		mark.style.cursor = 'pointer';

		if (start === 0 && end === node.length) {
			node.parentNode?.insertBefore(mark, node);
			mark.appendChild(node);
		} else {
			const wrappedText = node.splitText(start);
			const afterText = wrappedText.splitText(end - start);
			wrappedText.parentNode?.insertBefore(mark, wrappedText);
			mark.appendChild(wrappedText);
			void afterText;
		}
	}
}

function runsInRange(
	runs: TextRun[],
	startOffset: number,
	endOffset: number
): { node: Text; start: number; end: number }[] {
	const start = boundaryAt(runs, startOffset, false);
	const end = boundaryAt(runs, endOffset, true);
	if (!start || !end || startOffset >= endOffset) return [];

	return runs
		.filter((run) => run.end > startOffset && run.start < endOffset)
		.map((run) => ({
			node: run.node,
			start: Math.max(0, startOffset - run.start),
			end: Math.min(run.node.length, endOffset - run.start)
		}));
}

/**
 * Returns the total character length of text content within a container,
 * used for calculating percentage positions of highlights.
 */
export function getTotalTextLength(containerEl: HTMLElement): number {
	return buildTextIndex(containerEl, shouldIndexReaderTextNode).text.length;
}
