/**
 * The extension frame's content.
 *
 * The saved document is the one the rest of the page already carries: the
 * towardsdatascience.com piece that is Home's top Continue Reading card.
 * Same title, author and cover, so the surfaces agree.
 */

export const SAVED_PAGE = {
	title:
		'Mechanistic View of Transformers: Patterns, Messages, Residual Stream and LSTMs',
	tabTitle: 'Mechanistic View of Transformers: Pa…',
	host: 'towardsdatascience.com',
	slug: '/mechanistic-view-of-transformers-patterns-messages-residual-stream-and-lstms/',
	author: 'Kunj Mehta',
	readingTime: '12 min read',
	date: '16 Aug 2026',
	kicker: 'Artificial intelligence',
	savedAgo: 'saved 9h ago',
} as const;

/**
 * Deliberately generic. A real bookmark bar is personal data and has no
 * business on a marketing page.
 */
export const BOOKMARKS = [
	{ colour: '#4285F4', letter: 'G', name: 'Google' },
	{ colour: '#E8710A', letter: 'M', name: 'Morning reads' },
	{ colour: '#1E88C7', letter: 'T', name: 'Towards Data Science' },
	{ colour: '#0A0A0A', letter: 'H', name: 'Hacker News' },
	{ colour: '#2EA44F', letter: 'G', name: 'github.com' },
	{ colour: '#CC0000', letter: 'Y', name: 'Watch later' },
] as const;

export const TABS = [
	{ colour: '#1E88C7', letter: 'T', title: SAVED_PAGE.tabTitle, active: true },
	{ colour: '#0A0A0A', letter: 'H', title: 'Hacker News', active: false },
	{ colour: '#2EA44F', letter: 'G', title: 'useindelible/indelible', active: false },
] as const;

/**
 * Original prose on the article's own subject. The live page's wording belongs
 * to its author and is not reproduced here; the title, domain, author and
 * cover are the real record.
 *
 * The highlight sits in the FIRST paragraph on purpose: the publisher masthead
 * plus the injected bar push everything down, and a highlight further in gets
 * wiped below the bottom edge of the frame.
 */
export interface ExtensionParagraph {
	text: string;
	/** Rendered as a live highlight at the end of the paragraph. */
	highlight?: string;
}

export const ARTICLE_PARAGRAPHS: readonly ExtensionParagraph[] = [
	{
		text: 'Attention is usually taught as three matrices and a concatenation. Build Q, K and V for every head, run the heads independently, then project the stack once through O. ',
		highlight:
			'Read it mechanistically and that last projection stops being a single step at the end.',
	},
	{
		text: 'Each head owns its own slice of O, so it writes into the residual stream directly rather than waiting to be stacked beside its siblings. That is what makes the stream legible: every head becomes a small message passed forward on its own.',
	},
	{
		text: 'The sum at the far end reads as a channel rather than a bottleneck — which is the shape an LSTM cell state had all along.',
	},
];
