import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$lib/auth-tokens', () => ({
	getAccessToken: () => 'test-token'
}));

import { uploadOpml } from '$lib/api/feeds';

describe('uploadOpml', () => {
	let originalFetch: typeof fetch;

	beforeEach(() => {
		originalFetch = globalThis.fetch;
	});

	afterEach(() => {
		globalThis.fetch = originalFetch;
	});

	it('identifies a malformed file and surfaces the first OPML parser error', async () => {
		globalThis.fetch = vi.fn().mockResolvedValue(
			new Response(
				JSON.stringify({
					detail: 'validation error',
					errors: [
						{ field: 'opml', message: 'invalid OPML XML: mismatched closing tag' },
						{ field: 'opml', message: 'second error must not replace the first' }
					]
				}),
				{ status: 422, headers: { 'content-type': 'application/problem+json' } }
			)
		) as typeof fetch;

		const result = await uploadOpml(new File(['<opml>'], 'broken-export.opml'));

		expect(result).toEqual({
			ok: false,
			error:
				'broken-export.opml: invalid OPML XML: mismatched closing tag — Choose a valid OPML file and try again.'
		});
	});

	it('preserves the server detail for non-validation failures', async () => {
		globalThis.fetch = vi.fn().mockResolvedValue(
			new Response(JSON.stringify({ detail: 'Service temporarily unavailable' }), {
				status: 503,
				headers: { 'content-type': 'application/problem+json' }
			})
		) as typeof fetch;

		const result = await uploadOpml(new File(['<opml/>'], 'feeds.opml'));

		expect(result).toEqual({ ok: false, error: 'Service temporarily unavailable' });
	});
});
