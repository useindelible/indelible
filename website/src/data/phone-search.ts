/**
 * Search on a phone.
 *
 * The same query and corpus as the desktop search screen. Three of the five
 * hits fit the frame; the filter pills carry the full counts, which is what
 * the app does rather than pretending the list is complete.
 */

export interface PhoneSearchHit {
	tint: string;
	emoji?: string;
	kind?: string;
	play?: boolean;
	duration?: string;
	source: string;
	length: string;
	title: string;
	/** Raw text; the term is highlighted at render time. */
	summary: string;
	stamp: string;
}

export const PHONE_SEARCH_QUERY = 'beatles';

export const PHONE_SEARCH_HEAD = { eyebrow: '5 results, 12ms', title: 'Search' } as const;

export const PHONE_SEARCH_FILTERS = [
	{ label: 'All 5', on: true },
	{ label: 'Articles 4', on: false },
	{ label: 'Videos 1', on: false },
];

export const PHONE_SEARCH_HITS: readonly PhoneSearchHit[] = [
	{
		tint: 't-indigo', emoji: '📄',
		source: 'en.wikipedia.org', length: '15 min',
		title: 'Beatles for Sale - Wikipedia',
		summary:
			"Beatles for Sale is the fourth studio album by the English rock band the Beatles . It was released ... from the upbeat tone that had characterised the Beatles' previous work",
		stamp: 'Saved 1d ago',
	},
	{
		tint: 't-plum', kind: 'Video', play: true, duration: '3:31',
		source: 'YouTube', length: '8m ago',
		title: "The Beatles - Don't Let Me Down (Live Performance) [Mono / 2009 Remaster]",
		summary:
			'TheBeatlesVEVO 575.8M views 3:31 The Beatles performing "Don\'t Let Me Down." Anthology Collection CD & vinyl ... Michael Lindsay-Hogg was once again directing a Beatles \' shoot',
		stamp: 'Saved 8m ago',
	},
	{
		tint: 't-slate', emoji: '📄',
		source: 'en.wikipedia.org', length: '12 min',
		title: 'Attention Is All You Need - Wikipedia',
		summary:
			'song " All You Need Is Love " by the Beatles . 11 The name "Transformer" was picked because Jakob Uszkoreit',
		stamp: 'Saved 3m ago',
	},
];
