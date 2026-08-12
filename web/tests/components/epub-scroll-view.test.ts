import { fireEvent, render } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import EpubScrollView from '$lib/components/reader/book/EpubScrollView.svelte';
import type { BookSource } from '$lib/components/reader/book/book-source';

class StubIntersectionObserver implements IntersectionObserver {
	readonly root = null;
	readonly rootMargin = '';
	readonly thresholds = [];
	disconnect = vi.fn();
	observe = vi.fn();
	takeRecords = vi.fn(() => []);
	unobserve = vi.fn();
}

function source(): BookSource {
	return {
		metadata: { title: 'Short book', totalChapters: 2 },
		toc: [
			{ id: 'chapter-0', title: 'Chapter 1', depth: 1, index: 0, chapterId: 'chapter-0' },
			{ id: 'chapter-1', title: 'Chapter 2', depth: 1, index: 1, chapterId: 'spine-1' },
			{
				id: 'chapter-1-section',
				title: 'Chapter 2 section',
				depth: 2,
				index: 1,
				chapterId: 'spine-1',
				fragment: 'section'
			}
		],
		loadPage: vi.fn(async (index) => ({
			type: 'html' as const,
			html: index === 1 ? '<h2 id="section">Section</h2>' : '<p>Opening</p>',
			id: `chapter-${index}`,
			title: `Chapter ${index + 1}`,
			wordCount: 1
		})),
		destroy: vi.fn()
	};
}

function setScrollMetrics(element: HTMLElement, scrollHeight: number, clientHeight: number) {
	Object.defineProperties(element, {
		scrollHeight: { configurable: true, value: scrollHeight },
		clientHeight: { configurable: true, value: clientHeight }
	});
}

describe('EpubScrollView.scrollToChapter', () => {
	beforeEach(() => {
		vi.stubGlobal('IntersectionObserver', StubIntersectionObserver);
		vi.stubGlobal('CSS', { escape: (value: string) => value });
		Element.prototype.scrollIntoView = vi.fn();
	});

	it('makes an explicit fragment target authoritative when the book has no overflow', () => {
		const onChapterChange = vi.fn();
		const onActiveEntryChange = vi.fn();
		const onProgress = vi.fn();
		const rendered = render(EpubScrollView, {
			props: {
				source: source(),
				highlights: [],
				onChapterChange,
				onActiveEntryChange,
				onProgress
			}
		});
		const scrollContainer =
			rendered.container.querySelector<HTMLElement>('.epub-scroll-container')!;
		setScrollMetrics(scrollContainer, 400, 800);
		const wrappers = rendered.container.querySelectorAll<HTMLElement>('[data-chapter-index]');
		vi.spyOn(scrollContainer, 'getBoundingClientRect').mockReturnValue({ top: 0 } as DOMRect);
		vi.spyOn(wrappers[0], 'getBoundingClientRect').mockReturnValue({ top: 0 } as DOMRect);
		vi.spyOn(wrappers[1], 'getBoundingClientRect').mockReturnValue({ top: 200 } as DOMRect);
		Element.prototype.scrollIntoView = vi.fn(() => {
			scrollContainer.dispatchEvent(new Event('scroll'));
		});

		rendered.component.scrollToChapter(1, 0, 'section');

		expect(onChapterChange).toHaveBeenCalledOnce();
		expect(onChapterChange).toHaveBeenCalledWith(1, 'spine-1');
		expect(onActiveEntryChange).toHaveBeenCalledOnce();
		expect(onActiveEntryChange).toHaveBeenCalledWith('chapter-1-section');
		expect(onProgress).toHaveBeenCalledOnce();
		expect(onProgress).toHaveBeenCalledWith(100, 1, 0);
	});

	it('preserves the requested character offset when the book has no overflow', () => {
		const onActiveEntryChange = vi.fn();
		const onProgress = vi.fn();
		const rendered = render(EpubScrollView, {
			props: {
				source: source(),
				highlights: [],
				onActiveEntryChange,
				onProgress
			}
		});
		const scrollContainer =
			rendered.container.querySelector<HTMLElement>('.epub-scroll-container')!;
		setScrollMetrics(scrollContainer, 800, 800);

		rendered.component.scrollToChapter(1, 37);

		expect(onActiveEntryChange).toHaveBeenCalledWith('chapter-1');
		expect(onProgress).toHaveBeenCalledWith(100, 1, 37);
	});

	it('emits the stable chapter id from scroll geometry when the book overflows', async () => {
		const onChapterChange = vi.fn();
		const onActiveEntryChange = vi.fn();
		const onProgress = vi.fn();
		const rendered = render(EpubScrollView, {
			props: {
				source: source(),
				highlights: [],
				onChapterChange,
				onActiveEntryChange,
				onProgress
			}
		});
		const scrollContainer =
			rendered.container.querySelector<HTMLElement>('.epub-scroll-container')!;
		setScrollMetrics(scrollContainer, 1200, 800);
		const wrappers = rendered.container.querySelectorAll<HTMLElement>('[data-chapter-index]');
		vi.spyOn(scrollContainer, 'getBoundingClientRect').mockReturnValue({
			top: 0
		} as DOMRect);
		vi.spyOn(wrappers[0], 'getBoundingClientRect').mockReturnValue({ top: -200 } as DOMRect);
		vi.spyOn(wrappers[1], 'getBoundingClientRect').mockReturnValue({ top: 0 } as DOMRect);

		rendered.component.scrollToChapter(1, 0, 'section');

		expect(onChapterChange).not.toHaveBeenCalled();
		expect(onActiveEntryChange).not.toHaveBeenCalled();
		expect(onProgress).not.toHaveBeenCalled();

		await fireEvent.scroll(scrollContainer);

		expect(onChapterChange).toHaveBeenCalledWith(1, 'spine-1');
		expect(onProgress).toHaveBeenCalledWith(0, 1, 0);
	});
});
