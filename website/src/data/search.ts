/**
 * The search view: one query and its results.
 *
 * SearchPhone runs the same query over the same corpus, so the desktop screen
 * and the phone inset read as one search rather than two unrelated ones.
 */

export const SEARCH_QUERY = 'beatles';

/** The filter grammar, shown as mono chips under the field. */
export const SEARCH_SYNTAX: readonly string[] = [
	'tag:', 'collection:', 'type:', 'author:', 'sender:', 'sender_domain:',
	'list:', 'subject:', 'before:', 'after:', 'is:', 'has:', 'url:', 'pinned:', '!tag:',
];

/** The concept the query matched, shown above the results. */
export const SEARCH_ENTITY = {
	name: 'Beatles for Sale',
	badge: 'Work',
	mentions: '5 mentions in your library',
	seen: 'First seen Aug 2026 · Most recent Aug 2026',
} as const;

export interface SearchHit {
	emoji: string;
	tint: string;
	favLetter: string;
	favColour: string;
	title: string;
	kind: string;
	age: string;
	domain: string;
	excerpt: string;
	/** e1 is the primary entity (orange); e2 is a related work (blue). */
	entities: readonly { name: string; tone: 'e1' | 'e2' }[];
	selected?: boolean;
}

export const SEARCH_HITS: readonly SearchHit[] = [
	{
		emoji: '📄', tint: 'linear-gradient(150deg,#EFDCFF,#E2C6FB)',
		favLetter: 'W', favColour: '#1D1D1F',
		title: 'Beatles for Sale - Wikipedia', kind: 'Article', age: '1d',
		domain: 'en.wikipedia.org',
		excerpt: "Beatles for Sale is the fourth studio album by the English rock band the Beatles . It was released ... from the upbeat tone that had characterised the Beatles' previous work, partly due to the band's exhaustion",
		entities: [
			{ name: 'The Beatles', tone: 'e1' },
			{ name: "Beatles '65", tone: 'e2' },
			{ name: 'Beatles VI', tone: 'e2' },
			{ name: "Another Beatles' Christmas Show", tone: 'e2' },
		],
		selected: true,
	},
	{
		emoji: '📄', tint: 'linear-gradient(150deg,#EFDCFF,#E2C6FB)',
		favLetter: 'W', favColour: '#1D1D1F',
		title: 'Here Comes the Sun - Wikipedia', kind: 'Article', age: '1d',
		domain: 'en.wikipedia.org',
		excerpt: "song by the English rock band the Beatles from their eleventh studio album Abbey Road ... avoid attending a meeting at the Beatles ' Apple Corps organisation. The lyrics reflect his relief at the arrival",
		entities: [{ name: 'The Beatles', tone: 'e1' }],
	},
	{
		emoji: '📄', tint: 'linear-gradient(150deg,#FFE9C7,#FFD79B)',
		favLetter: 'P', favColour: '#B24926',
		title: 'The Beatles: Beatles For Sale Album Review | Pitchfork', kind: 'Article', age: '6m',
		domain: 'pitchfork.com',
		excerpt: 'Beatles faced the same pressures every teen sensation has since-- fatigue, frustration, being bounced into recording substandard material ... Christmas mixture," claimed Derek Taylor on the Beatles For Sale sleeve. This pre-emptive strike looks more than',
		entities: [
			{ name: 'Beatles for Sale', tone: 'e2' },
			{ name: 'The Beatles', tone: 'e1' },
		],
	},
	{
		emoji: '🎬', tint: 'linear-gradient(150deg,#FFDCE4,#FBC3D2)',
		favLetter: 'Y', favColour: '#FF0000',
		title: "The Beatles - The Beatles - Don't Let Me Down (Live Performance) [Mono / 2009 Remaster]",
		kind: 'Video', age: '8m', domain: 'youtube.com',
		excerpt: 'TheBeatlesVEVO 575.8M views 3:31 The Beatles performing "Don\'t Let Me Down." Anthology Collection CD & vinyl ... Michael Lindsay-Hogg was once again directing a Beatles \' shoot. He and Paul met regularly at the tail',
		entities: [{ name: 'The Beatles', tone: 'e1' }],
	},
	{
		emoji: '📄', tint: 'linear-gradient(150deg,#EFDCFF,#E2C6FB)',
		favLetter: 'W', favColour: '#1D1D1F',
		title: 'Attention Is All You Need - Wikipedia', kind: 'Article', age: '3m',
		domain: 'en.wikipedia.org',
		excerpt: 'song " All You Need Is Love " by the Beatles . 11 The name "Transformer" was picked because Jakob Uszkoreit',
		entities: [{ name: 'The Beatles', tone: 'e1' }],
	},
];
