import type { MilaSourceRef } from '$lib/api/generated/types.gen';
import { renderMarkdown } from '$lib/utils/markdown';

type ReaderHref = (documentId: string) => string;

const SOURCE_TOKEN_RE = /\[S\d+\]/g;

export function renderMilaMessageMarkdown(
	text: string,
	sourceRefs: MilaSourceRef[],
	readerHref: ReaderHref
): string {
	const refsByLabel = new Map(sourceRefs.map((ref) => [ref.source_label, ref]));
	const withCollapsedDuplicateSources = collapseRepeatedAdjacentLabels(text);
	const withInlineSources = withCollapsedDuplicateSources.replace(SOURCE_TOKEN_RE, (token) => {
		const ref = refsByLabel.get(token.slice(1, -1));
		if (!ref) return token;
		const href = readerHref(ref.document_id);
		return `<a class="chat-inline-source" href="${escapeAttribute(href)}" title="${escapeAttribute(ref.item_title)}">${escapeHtml(shortTitle(ref.item_title))}</a>`;
	});

	return renderMarkdown(withInlineSources);
}

function collapseRepeatedAdjacentLabels(text: string): string {
	return text.replace(/(\[S\d+\])(?:\1)+/g, '$1');
}

function shortTitle(title: string): string {
	const trimmed = title.trim();
	if (trimmed.length <= 28) return trimmed;
	return `${trimmed.slice(0, 27)}...`;
}

function escapeAttribute(value: string): string {
	return escapeHtml(value).replaceAll('"', '&quot;');
}

function escapeHtml(value: string): string {
	return value
		.replaceAll('&', '&amp;')
		.replaceAll('<', '&lt;')
		.replaceAll('>', '&gt;')
		.replaceAll('"', '&quot;')
		.replaceAll("'", '&#39;');
}
