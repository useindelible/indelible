/**
 * The feed on a phone.
 *
 * The same entries as the desktop Feed view, folded into one column, so the
 * two frames on the page read as one product rather than two demos.
 */

export interface PhoneFeedEntry {
	tint: string;
	emoji: string;
	unread?: boolean;
	source: string;
	length: string;
	title: string;
	summary: string;
}

export const PHONE_FEED_FILTERS = [
	{ label: 'All 24', on: true },
	{ label: 'Unseen 12', on: false },
	{ label: 'Sources 7', on: false },
];

export const PHONE_FEED_HEAD = {
	eyebrow: '7 sources, refreshed 4m ago',
	title: 'Unseen',
	count: '12',
	meter: { unread: 12, reading: 3, done: 9, third: '24 read this week' },
} as const;

export const PHONE_FEED_ENTRIES: readonly PhoneFeedEntry[] = [
	{
		tint: 't-slate', emoji: '🛰', unread: true,
		source: 'nasa.gov', length: '16m ago',
		title: 'The View from Above: The Gemini Visual Acuity Experiments',
		summary:
			'NASA astronaut L. Gordon Cooper, Jr. took 29 color photographs of the Earth with a 70mm camera as he orbited our planet during the Mercury-Atlas 9 mission.',
	},
	{
		tint: 't-moss', emoji: '💻', unread: true,
		source: 'github.blog', length: '0m ago',
		title: 'Separate GitHub Actions path for GitHub Code Quality',
		summary:
			'A dedicated workflow path for code quality CodeQL actions workflows is now generally available.',
	},
	{
		tint: 't-indigo', emoji: '🌌',
		source: 'science.nasa.gov', length: '16m ago',
		title: 'NASA Data Feeds River Forecasts as Snow Drought Effects Linger',
		summary:
			'NASA Earth science data is supporting machine-learning forecasts that inform decisions about water, power, and public safety in Washington state.',
	},
	{
		tint: 't-clay', emoji: '☢️', unread: true,
		source: 'techcrunch.com', length: '1h ago',
		title: 'Apollo Atomics wants to make nuclear power cheaper by shrinking an overlooked part',
		summary:
			'Y Combinator alumnus Apollo Atomics is shrinking a key nuclear reactor part, which promises to slash the cost of electricity below natural gas.',
	},
];
