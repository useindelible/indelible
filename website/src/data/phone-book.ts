/**
 * The book on a phone: Moby-Dick, chapter one, on sepia paper.
 *
 * The same book as the desktop EPUB reader, so the two frames on the page read
 * as one library rather than two demos.
 */

export const PHONE_BOOK = {
	mark: '🐋',
	crumb: 'Moby-Dick · Ch. 1',
	title: 'MOBY-DICK; or, THE WHALE.',
	author: 'Herman Melville',
	page: 'Page 1 of 866',
	remaining: '~14h left',
} as const;

export interface BookBlock {
	kind: 'p' | 'pull';
	text: string;
	highlight?: { phrase: string; colour: string };
}

export const PHONE_BOOK_BODY: readonly BookBlock[] = [
	{ kind: 'p', text: 'The Project Gutenberg eBook of Moby Dick; Or, The Whale' },
	{
		kind: 'p',
		text:
			'Call me Ishmael. Some years ago, never mind how long precisely, having little or no money in my purse, and nothing particular to interest me on shore, I thought I would sail about a little and see the watery part of the world.',
		highlight: { phrase: 'Call me Ishmael.', colour: 'y' },
	},
	{
		kind: 'pull',
		text: 'It is a way I have of driving off the spleen and regulating the circulation.',
	},
	{
		kind: 'p',
		text:
			'Whenever I find myself growing grim about the mouth; whenever it is a damp, drizzly November in my soul, then I account it high time to get to sea as soon as I can.',
	},
];
