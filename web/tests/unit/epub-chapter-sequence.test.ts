import { describe, expect, it } from 'vitest';

import { epubChapterSequence, type TocEntry } from '$lib/components/reader/book/book-source';

function entry(id: string, depth: number, index: number, fragment?: string): TocEntry {
	return { id, title: id, depth, index, fragment };
}

describe('epubChapterSequence', () => {
	it('keeps distinct spines when one entry is nested', () => {
		const toc = [entry('opening', 1, 0), entry('signal', 2, 1, 'signal'), entry('closing', 1, 2)];

		expect(epubChapterSequence(toc)).toEqual(toc);
	});

	it('uses the deepest entry once for a shared spine', () => {
		const shallow = entry('part', 1, 0);
		const deep = entry('chapter', 2, 0, 'chapter');

		expect(epubChapterSequence([shallow, deep])).toEqual([deep]);
	});

	it('preserves a legacy flat sequence', () => {
		const toc = [entry('one', 1, 0), entry('two', 1, 1), entry('three', 1, 2)];

		expect(epubChapterSequence(toc)).toEqual(toc);
	});

	it('keeps the first equal-depth entry for a shared spine', () => {
		const first = entry('first', 2, 0, 'first');
		const second = entry('second', 2, 0, 'second');

		expect(epubChapterSequence([first, second])).toEqual([first]);
	});

	it('sorts representatives by spine index', () => {
		const third = entry('third', 1, 2);
		const first = entry('first', 1, 0);
		const second = entry('second', 1, 1);

		expect(epubChapterSequence([third, first, second])).toEqual([first, second, third]);
	});
});
