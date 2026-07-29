import { beforeEach, describe, expect, it, vi } from 'vitest';

const mockGetEpubToc = vi.fn();
const mockGetEpubChapter = vi.fn();

vi.mock('$lib/api/generated', async () => {
	const actual = await vi.importActual<typeof import('$lib/api/generated')>('$lib/api/generated');
	return {
		...actual,
		getEpubToc: (...args: unknown[]) => mockGetEpubToc(...args),
		getEpubChapter: (...args: unknown[]) => mockGetEpubChapter(...args)
	};
});

import { getEpubChapter, getEpubToc } from '$lib/api';

describe('EPUB toc/chapter adapters', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('forwards the toc request with json parse mode', async () => {
		mockGetEpubToc.mockResolvedValue({ data: { metadata: {}, toc: [] } });

		await getEpubToc({ path: { document_id: 'doc_book' }, parseAs: 'json' });

		expect(mockGetEpubToc).toHaveBeenCalledWith({
			path: { document_id: 'doc_book' },
			parseAs: 'json'
		});
	});

	it('forwards the chapter request with text parse mode and chapter index', async () => {
		mockGetEpubChapter.mockResolvedValue({ data: '<p>chapter</p>' });

		const { data } = await getEpubChapter({
			path: { document_id: 'doc_book', chapter_index: 3 },
			parseAs: 'text'
		});

		expect(mockGetEpubChapter).toHaveBeenCalledWith({
			path: { document_id: 'doc_book', chapter_index: 3 },
			parseAs: 'text'
		});
		expect(data).toBe('<p>chapter</p>');
	});
});
