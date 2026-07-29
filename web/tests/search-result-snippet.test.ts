import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';

import SearchResultRow from '$lib/components/search/SearchResultRow.svelte';
import type { SearchResultResponse } from '$lib/api/generated/types.gen';

function makeResult(snippet: string): SearchResultResponse {
	return {
		content_type: 'article',
		title: 'Test article',
		snippet,
		saved_at: '2026-01-01T00:00:00.000Z',
		url: 'https://example.com/a'
	} as unknown as SearchResultResponse;
}

const baseProps = {
	selected: false,
	onSelect: () => {},
	onOpen: () => {}
};

describe('SearchResultRow snippet sanitization', () => {
	it('keeps <mark> highlights but strips injected HTML', () => {
		const { container } = render(SearchResultRow, {
			props: {
				...baseProps,
				result: makeResult('<mark>safe</mark><img src=x onerror="alert(1)">')
			}
		});
		const excerpt = container.querySelector('.result-excerpt');
		expect(excerpt).toBeTruthy();
		expect(excerpt?.querySelector('mark')).toBeTruthy();
		expect(excerpt?.querySelector('img')).toBeNull();
		expect(excerpt?.innerHTML).not.toContain('onerror');
	});

	it('strips <script> from an unescaped ts_headline snippet', () => {
		const payload = 'before<scr' + 'ipt>alert(1)</scr' + 'ipt>after';
		const { container } = render(SearchResultRow, {
			props: { ...baseProps, result: makeResult(payload) }
		});
		const excerpt = container.querySelector('.result-excerpt');
		expect(excerpt?.querySelector('script')).toBeNull();
		expect(excerpt?.textContent).toContain('before');
	});
});
