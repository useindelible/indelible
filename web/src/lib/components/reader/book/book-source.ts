import * as apiSdk from '$lib/api';
import type { EpubTocResponse } from '$lib/api';
import type { PDFPageProxy, TextItem } from 'pdfjs-dist/types/src/display/api';

export type { PDFPageProxy, TextItem };

export interface TocEntry {
	id: string;
	title: string;
	depth: number;
	index: number;
	/** Stable spine/chapter ID — same for all subsections within one file. */
	chapterId?: string;
	/** Fragment anchor within the chapter HTML (e.g. "deserved-respect"). */
	fragment?: string;
	wordCount?: number;
	startPage?: number;
	/** Basename of the spine item's manifest href (e.g. "notes.xhtml"). Used to resolve
	 * intra-EPUB cross-chapter link targets by matching against link filenames. */
	spineHref?: string;
}

export function epubChapterSequence(toc: TocEntry[]): TocEntry[] {
	const representatives = new Map<number, TocEntry>();
	for (const entry of toc) {
		const current = representatives.get(entry.index);
		if (!current || entry.depth > current.depth) {
			representatives.set(entry.index, entry);
		}
	}
	return [...representatives.values()].sort((a, b) => a.index - b.index);
}

export interface BookMetadata {
	title?: string;
	author?: string;
	publisher?: string;
	isbn?: string;
	language?: string;
	totalChapters: number;
	totalWords?: number;
	estimatedPages?: number;
}

export type BookPageContent =
	| { type: 'html'; html: string; id: string; title: string; wordCount: number }
	| {
			type: 'pdf';
			page: PDFPageProxy;
			width: number;
			height: number;
	  };

export interface BookSource {
	metadata: BookMetadata;
	toc: TocEntry[];
	loadPage(index: number): Promise<BookPageContent>;
	destroy(): void;
}

export function estimatePageNumber(
	entry: TocEntry,
	charOffset: number,
	chapterTotalChars: number
): number {
	const startPage = entry.startPage ?? 1;
	const wordCount = entry.wordCount ?? 0;
	const chapterPages = Math.max(1, Math.ceil(wordCount / 250));
	if (chapterTotalChars <= 0) return startPage;
	const fractionThrough = Math.min(1, Math.max(0, charOffset / chapterTotalChars));
	return startPage + Math.floor(fractionThrough * chapterPages);
}

export async function createEpubSource(itemId: string): Promise<BookSource> {
	const { data } = await apiSdk.getEpubToc({
		path: { document_id: itemId },
		parseAs: 'json'
	});

	const tocResponse: EpubTocResponse | undefined = data;
	if (!tocResponse?.toc) {
		throw new Error('Failed to load EPUB table of contents');
	}

	const meta = tocResponse.metadata;
	const metadata: BookMetadata = {
		title: meta.title ?? undefined,
		author: meta.author ?? undefined,
		publisher: meta.publisher ?? undefined,
		isbn: meta.isbn ?? undefined,
		language: meta.language ?? undefined,
		totalChapters: meta.total_chapters,
		totalWords: meta.total_words,
		estimatedPages: meta.estimated_pages
	};

	const toc: TocEntry[] = tocResponse.toc.map((entry) => ({
		id: entry.id,
		title: entry.title,
		depth: entry.depth,
		index: entry.spine_index,
		chapterId: entry.chapter_id ?? entry.id,
		fragment: entry.fragment ?? undefined,
		wordCount: entry.word_count,
		startPage: entry.start_page,
		spineHref: entry.spine_href || undefined
	}));

	const chapterCache = new Map<number, BookPageContent>();

	return {
		metadata,
		toc,
		async loadPage(index: number): Promise<BookPageContent> {
			const cached = chapterCache.get(index);
			if (cached) return cached;

			const { data: chapterHtml } = await apiSdk.getEpubChapter({
				path: { document_id: itemId, chapter_index: index },
				parseAs: 'text'
			});

			const tocEntry = toc.find((e) => e.index === index);
			const content: BookPageContent = {
				type: 'html',
				html: (chapterHtml as unknown as string) ?? '',
				id: tocEntry?.id ?? `ch-${index}`,
				title: tocEntry?.title ?? `Chapter ${index + 1}`,
				wordCount: tocEntry?.wordCount ?? 0
			};

			chapterCache.set(index, content);
			return content;
		},
		destroy() {
			chapterCache.clear();
		}
	};
}

export async function createPdfSource(
	downloadUrl: string,
	itemMeta?: { title?: string; author?: string | null }
): Promise<BookSource> {
	const pdfjsLib = await import('pdfjs-dist');
	pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
		'pdfjs-dist/build/pdf.worker.min.mjs',
		import.meta.url
	).toString();

	const pdf = await pdfjsLib.getDocument(downloadUrl).promise;
	const numPages = pdf.numPages;

	let outline: TocEntry[] = [];
	try {
		const pdfOutline = await pdf.getOutline();
		if (pdfOutline && pdfOutline.length > 0) {
			type OutlineItem = (typeof pdfOutline)[number];
			const flatItems: { item: OutlineItem; depth: number }[] = [];

			const flatten = (items: OutlineItem[], depth: number) => {
				for (const item of items) {
					flatItems.push({ item, depth });
					if (item.items?.length) flatten(item.items, depth + 1);
				}
			};
			flatten(pdfOutline, 1);

			const resolved = await Promise.all(
				flatItems.map(async ({ item, depth }, idx) => {
					let pageIndex = idx;
					try {
						if (item.dest) {
							const dest =
								typeof item.dest === 'string' ? await pdf.getDestination(item.dest) : item.dest;
							if (dest) {
								const ref = dest[0];
								pageIndex = await pdf.getPageIndex(ref);
							}
						}
					} catch {
						// Fall back to sequential index
					}
					return {
						id: `outline-${idx}`,
						title: item.title,
						depth,
						index: pageIndex,
						startPage: pageIndex + 1
					} satisfies TocEntry;
				})
			);

			outline = resolved;
		}
	} catch {
		// PDF has no outline
	}

	const toc: TocEntry[] =
		outline.length > 0
			? outline
			: Array.from({ length: numPages }, (_, i) => ({
					id: `page-${i}`,
					title: `Page ${i + 1}`,
					depth: 1,
					index: i,
					startPage: i + 1
				}));

	let pdfTitle: string | undefined;
	let pdfAuthor: string | undefined;
	try {
		const pdfMeta = await pdf.getMetadata();
		const info = pdfMeta?.info as Record<string, unknown> | undefined;
		if (info) {
			if (typeof info.Title === 'string' && info.Title.trim()) pdfTitle = info.Title.trim();
			if (typeof info.Author === 'string' && info.Author.trim()) pdfAuthor = info.Author.trim();
		}
	} catch {
		// PDF has no metadata
	}

	const metadata: BookMetadata = {
		title: pdfTitle ?? itemMeta?.title ?? undefined,
		author: pdfAuthor ?? itemMeta?.author ?? undefined,
		totalChapters: numPages,
		estimatedPages: numPages
	};

	return {
		metadata,
		toc,
		async loadPage(index: number): Promise<BookPageContent> {
			const page = await pdf.getPage(index + 1);
			const viewport = page.getViewport({ scale: 1 });

			return {
				type: 'pdf',
				page,
				width: viewport.width,
				height: viewport.height
			};
		},
		destroy() {
			pdf.destroy();
		}
	};
}
