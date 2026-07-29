import DOMPurify from 'dompurify';

const YOUTUBE_EMBED = /^https:\/\/(www\.)?youtube(-nocookie)?\.com\/embed\//i;

// Reader content is server-sanitized (ammonia) before storage; this is the client-side
// defense-in-depth layer so a gap in the ingestion pipeline can never reach {@html}.
// DOMPurify strips scripts, event-handler attributes, and javascript:/data: URLs. The only
// iframe the reader legitimately renders is the YouTube transcript embed, so iframes are
// permitted but every non-YouTube-embed one is dropped.
const READER_CONFIG = {
	ADD_TAGS: ['iframe'],
	ADD_ATTR: ['allow', 'allowfullscreen', 'frameborder', 'scrolling']
};

function dropForeignIframes(node: Node): void {
	if (node.nodeName !== 'IFRAME') return;
	const src = (node as Element).getAttribute('src') ?? '';
	if (!YOUTUBE_EMBED.test(src)) node.parentNode?.removeChild(node);
}

/**
 * Sanitize archived third-party article/EPUB HTML for inline rendering in the reader.
 *
 * The hook is registered and removed around the synchronous `sanitize` call so it never
 * affects the other DOMPurify consumers (markdown, search snippets), which run their own
 * stricter configs.
 */
export function sanitizeReaderHtml(html: string): string {
	DOMPurify.addHook('uponSanitizeElement', dropForeignIframes);
	try {
		return DOMPurify.sanitize(html, READER_CONFIG);
	} finally {
		DOMPurify.removeHook('uponSanitizeElement');
	}
}
