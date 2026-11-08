export interface TextPosition {
	node: Text;
	offset: number;
}

/** Whitespace-only text has no box, so an offset inside it moves to the next visible text. */
export function textPositionAt(body: HTMLElement, charOffset: number): TextPosition | null {
	const walker = document.createTreeWalker(body, NodeFilter.SHOW_TEXT);
	let charCount = 0;
	let reached = false;
	let node = walker.nextNode() as Text | null;
	while (node) {
		const text = node.textContent ?? '';
		if (reached && text.trim()) return { node, offset: 0 };
		if (!reached && charCount + text.length >= charOffset) {
			if (text.trim()) return { node, offset: Math.min(charOffset - charCount, text.length) };
			reached = true;
		}
		charCount += text.length;
		node = walker.nextNode() as Text | null;
	}
	return null;
}
