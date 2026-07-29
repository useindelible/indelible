import { render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import BookNavBar from '$lib/components/reader/book/BookNavBar.svelte';
import type { TocEntry } from '$lib/components/reader/book/book-source';

describe('BookNavBar', () => {
	it('uses the navigable chapter number when the active spine starts with a section heading', () => {
		const toc: TocEntry[] = [
			{
				id: 'section-setup',
				title: 'Setup',
				depth: 1,
				index: 0
			},
			{
				id: 'chapter-answers',
				title: 'Answers',
				depth: 2,
				index: 0
			},
			{
				id: 'chapter-structure',
				title: 'Structure',
				depth: 2,
				index: 1
			}
		];

		render(BookNavBar, {
			props: {
				toc,
				currentIndex: 0,
				totalChapters: 2,
				onPrev: vi.fn(),
				onNext: vi.fn()
			}
		});

		expect(screen.getByText('1 of 2 chapters')).toBeTruthy();
		expect(screen.queryByText('0 of 2 chapters')).toBeNull();
	});
});
