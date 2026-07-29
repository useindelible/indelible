import { render } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import BookSidebar from '$lib/components/reader/book/BookSidebar.svelte';
import type { BookSource } from '$lib/components/reader/book/book-source';

function source(): BookSource {
	return {
		metadata: {
			title: 'Async Rust',
			author: 'Maxwell Flitton',
			totalChapters: 1
		},
		toc: [
			{
				id: 'chapter-1',
				title: 'Chapter 1',
				depth: 1,
				index: 0
			}
		],
		async loadPage() {
			return { type: 'html', html: '', id: 'chapter-1', title: 'Chapter 1', wordCount: 0 };
		},
		destroy() {}
	};
}

describe('BookSidebar', () => {
	it('renders the thumbnail in the existing cover slot when one is available', () => {
		const { container } = render(BookSidebar, {
			props: {
				source: source(),
				currentIndex: 0,
				activeEntryId: null,
				progress: 12,
				highlights: [],
				activeTab: 'contents',
				onTabChange: vi.fn(),
				onNavigate: vi.fn(),
				onBookmarkNavigate: vi.fn(),
				thumbnailUrl: '/api/v1/assets/thumb.png'
			}
		});

		const cover = container.querySelector('.book-cover');
		const thumbnail = cover?.querySelector('img.book-cover-image');

		expect(thumbnail).toBeInstanceOf(HTMLImageElement);
		expect(thumbnail?.getAttribute('src')).toBe('/api/v1/assets/thumb.png');
		expect(cover?.querySelector('svg')).toBeNull();
	});
});
