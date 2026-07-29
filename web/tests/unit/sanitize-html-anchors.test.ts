import { describe, expect, it } from 'vitest';

import { sanitizeReaderHtml } from '../../src/lib/utils/sanitize-html';

// The ToC contract depends on the client sanitizer preserving the anchor
// vocabulary the backend injects (prefixed ids and local fragment hrefs)
// while still stripping active content.
describe('sanitizeReaderHtml anchor preservation', () => {
	it('keeps heading ids and local fragment hrefs', () => {
		const out = sanitizeReaderHtml(
			'<h2 id="ind-toc-history">History</h2><a href="#ind-fn:1" id="ind-fnref:1">[1]</a>'
		);
		expect(out).toContain('id="ind-toc-history"');
		expect(out).toContain('href="#ind-fn:1"');
		expect(out).toContain('id="ind-fnref:1"');
	});

	it('still strips scripts and event handlers', () => {
		const out = sanitizeReaderHtml(
			'<h2 id="ind-toc-a" onclick="x()">A</h2><script>evil()</script>'
		);
		expect(out).not.toContain('onclick');
		expect(out).not.toContain('script');
		expect(out).toContain('id="ind-toc-a"');
	});
});
