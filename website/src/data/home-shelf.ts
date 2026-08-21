/**
 * The Home view's shelves.
 *
 * The desktop Home screen and the phone Home screen render from this same
 * data, folded differently: the top Continue Reading item becomes the phone's
 * hero card, the rest become its rail, and Quick Reads becomes the row list.
 * Change an item here and both surfaces move together — which is the point.
 */
import type { CoverArtName } from './cover-art';

export interface ShelfItem {
	art: CoverArtName;
	source: string;
	title: string;
	byline: string;
	/** Reading progress, 0-100. */
	progress: number;
}

/** The item the reader is furthest into. Rendered as the hero. */
export const CONTINUE_HERO = {
	art: 'desk',
	source: 'towardsdatascience.com',
	title: 'Mechanistic View of Transformers',
	byline: 'Kunj Mehta, 12 min left',
	progress: 55,
} as const satisfies ShelfItem;

export const CONTINUE_RAIL: readonly ShelfItem[] = [
	{
		art: 'xai',
		source: 'www.youtube.com',
		title: 'xAI just caught up (Grok 4.6 is here)',
		byline: 'Theo - t3.gg',
		progress: 22,
	},
	{
		art: 'band',
		source: 'en.wikipedia.org',
		title: 'Beatles for Sale - Wikipedia',
		byline: 'Wikipedia',
		progress: 100,
	},
	{
		art: 'transformer',
		source: 'en.wikipedia.org',
		title: 'Attention Is All You Need - Wikipedia',
		byline: 'Wikipedia',
		progress: 28,
	},
];

export interface QuickRead {
	art: CoverArtName;
	source: string;
	length: string;
	title: string;
	summary: string;
	stamp: string;
}

export const QUICK_READS: readonly QuickRead[] = [
	{
		art: 'band',
		source: 'pitchfork.com',
		length: '9 min',
		title: 'The Beatles: Beatles For Sale Album Review | Pitchfork',
		summary:
			'Beatles for Sale caught the band mid flight, exhausted by touring and short of original material, and it is the better record for admitting it.',
		stamp: 'Tom Ewing',
	},
	{
		art: 'spaced',
		source: 'en.wikipedia.org',
		length: '6 min',
		title: 'Spaced repetition',
		summary:
			'A schedule that widens the gap after every correct answer and collapses it after a wrong one, so review time lands where recall is weakest.',
		stamp: 'Wikipedia',
	},
];

/** The reading meter across the top of Home. */
export const READING_METER = {
	unread: 42,
	reading: 5,
	done: 19,
	remaining: '2h 50m to clear',
} as const;

export const WEEK_STATS = [
	{ icon: 'clock', value: '4h 12m', label: 'Read, 7d' },
	{ icon: 'bookmark', value: '23', label: 'Saved, 7d' },
	{ icon: 'check', value: '9', label: 'Finished, 7d' },
] as const;

/** Source letter-marks. Real favicons are not fetched; these are stated
    stand-ins, consistent with the row favicons in the library list. */
export const SOURCE_MARKS = {
	tds: { letter: 'T', colour: '#1E88C7' },
	youtube: { letter: 'W', colour: '#CC0000' },
	wikipedia: { letter: 'E', colour: '#3E6DA8' },
	pitchfork: { letter: 'P', colour: '#C0392B' },
	ririro: { letter: 'R', colour: '#7A5CC4' },
	cnbc: { letter: 'W', colour: '#CC0000' },
} as const;

export type SourceKey = keyof typeof SOURCE_MARKS;

export interface HomeCard {
	art: CoverArtName;
	mark: SourceKey;
	source: string;
	title: string;
	author: string;
	/** Only Continue Reading cards carry progress. */
	progress?: number;
}

export const HOME_CONTINUE: readonly HomeCard[] = [
	{ art: 'desk', mark: 'tds', source: 'towardsdatascience.com', title: 'Mechanistic View of Transformers: Patterns, Messages, Residual Stream', author: 'Kunj Mehta', progress: 55 },
	{ art: 'xai', mark: 'youtube', source: 'www.youtube.com', title: 'xAI just caught up (Grok 4.6 is here)', author: 'Theo - t3.gg', progress: 22 },
	{ art: 'band', mark: 'wikipedia', source: 'en.wikipedia.org', title: 'Beatles for Sale - Wikipedia', author: 'Wikipedia', progress: 100 },
	{ art: 'transformer', mark: 'wikipedia', source: 'en.wikipedia.org', title: 'Attention Is All You Need - Wikipedia', author: 'Wikipedia', progress: 28 },
	{ art: 'appstore', mark: 'cnbc', source: 'www.cnbc.com', title: 'Apple overhauls EU app store fees to resolve payments clash', author: 'Kif Leswing', progress: 8 },
	{ art: 'plain', mark: 'ririro', source: 'ririro.com', title: 'Martin Dickens and the Christmas Goose', author: 'pim', progress: 3 },
];

export const HOME_QUICK: readonly HomeCard[] = [
	{ art: 'band', mark: 'pitchfork', source: 'pitchfork.com', title: 'The Beatles: Beatles For Sale Album Review | Pitchfork', author: 'Tom Ewing' },
	{ art: 'ped', mark: 'wikipedia', source: 'en.wikipedia.org', title: 'Design thinking', author: 'Wikipedia' },
	{ art: 'spaced', mark: 'wikipedia', source: 'en.wikipedia.org', title: 'Spaced repetition', author: 'Wikipedia' },
	{ art: 'ped', mark: 'wikipedia', source: 'en.wikipedia.org', title: 'Personal knowledge management', author: 'Wikipedia' },
	{ art: 'digilib', mark: 'wikipedia', source: 'en.wikipedia.org', title: 'Digital library', author: 'Wikipedia' },
	{ art: 'shelf', mark: 'wikipedia', source: 'en.wikipedia.org', title: 'Zettelkasten', author: 'Wikipedia' },
];

export const HOME_RECENT: readonly HomeCard[] = [
	{ art: 'desk', mark: 'tds', source: 'towardsdatascience.com', title: 'A field guide to retrieval evaluation', author: 'Kunj Mehta' },
	{ art: 'digilib', mark: 'wikipedia', source: 'en.wikipedia.org', title: 'Open access', author: 'Wikipedia' },
	{ art: 'xai', mark: 'youtube', source: 'www.youtube.com', title: 'The state of local models, end of year', author: 'Theo - t3.gg' },
	{ art: 'appstore', mark: 'cnbc', source: 'www.cnbc.com', title: 'Regulators circle the default browser rules', author: 'Kif Leswing' },
	{ art: 'transformer', mark: 'wikipedia', source: 'en.wikipedia.org', title: 'Residual neural network', author: 'Wikipedia' },
	{ art: 'shelf', mark: 'wikipedia', source: 'en.wikipedia.org', title: 'Commonplace book', author: 'Wikipedia' },
];
