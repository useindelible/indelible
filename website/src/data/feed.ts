/**
 * Feed entries.
 *
 * These are entries from a source, NOT saved documents: they carry a FEED
 * badge, an age, and an empty progress track rather than reading state. Most
 * of their metadata is genuinely unknown until the entry is fetched, which is
 * why the record beside them is full of em dashes.
 */
import type { FeedArtName } from './feed-art';
import type { RecordField } from '../components/screens/app/RecordPanel.astro';

export interface FeedEntry {
	art: FeedArtName;
	favLetter: string;
	favColour: string;
	title: string;
	excerpt: string;
	domain: string;
	author?: string;
	age: string;
}

export const FEED_ENTRIES: readonly FeedEntry[] = [
	{
		art: 'photo', favLetter: 'N', favColour: '#0B3D91',
		title: 'The View from Above: The Gemini Visual Acuity Experiments',
		excerpt: 'NASA astronaut L. Gordon Cooper, Jr. took 29 color photographs of the Earth with a 70mm camera as he orbited our planet during the Mercury-Atlas 9 mission in May 1…',
		domain: 'nasa.gov', author: 'Michele Ostovar', age: '16m',
	},
	{
		art: 'ph-green', favLetter: 'G', favColour: '#24292F',
		title: 'Separate GitHub Actions path for GitHub Code Quality',
		excerpt: 'A dedicated workflow path for code quality CodeQL actions workflows is now generally available. Your workflow run history and your Actions usage reports now tell GitH…',
		domain: 'github.blog', author: 'Allison', age: '0m',
	},
	{
		art: 'ph-green', favLetter: 'G', favColour: '#24292F',
		title: 'Track GitHub Code Quality enablement changes in the audit log',
		excerpt: 'GitHub Code Quality now writes an audit log event whenever someone enables, disables, or changes its settings on a repository. Three new events give you that history: …',
		domain: 'github.blog', author: 'Allison', age: '0m',
	},
	{
		art: 'sat', favLetter: 'N', favColour: '#0B3D91',
		title: 'NASA Data Feeds River Forecasts as Snow Drought Effects Linger',
		excerpt: 'NASA Earth science data is supporting machine-learning forecasts that inform decisions about water, power, and public safety in Washington state.',
		domain: 'science.nasa.gov', age: '16m',
	},
	{
		art: 'ph-blue', favLetter: 'TC', favColour: '#0A9E5C',
		title: 'Apollo Atomics wants to make nuclear power cheaper by shrinking an overlooked part',
		excerpt: 'Y Combinator alumnus Apollo Atomics is shrinking a key nuclear reactor part, which promises to slash the cost of electricity below natural gas.',
		domain: 'techcrunch.com', author: 'Tim De Chant', age: '1h',
	},
	{
		art: 'ph-blue', favLetter: 'TC', favColour: '#0A9E5C',
		title: 'AI data giant Alation confirms cyberattack',
		excerpt: 'The data search and AI giant confirmed unauthorized access to its systems during an incident on Tuesday, and said it was investigating the breach.',
		domain: 'techcrunch.com', author: 'Zack Whittaker', age: '2h',
	},
	{
		art: 'ph-blue', favLetter: 'TC', favColour: '#0A9E5C',
		title: 'For a16z, AI gives foreign founders an advantage',
		excerpt: 'a16z’s Borderless Founder network initiative supports immigrant and international founders. ‘Having one foot in your home country, and one foot in Silicon Valley,’ is an ad…',
		domain: 'techcrunch.com', author: 'Anna Heim', age: '2h',
	},
	{
		art: 'ph-blue', favLetter: 'TC', favColour: '#0A9E5C',
		title: 'US says hackers are targeting vulnerable water systems with the help of AI',
		excerpt: 'Hackers are targeting internet-connected Siemens controllers used in water facilities around the United States.',
		domain: 'techcrunch.com', author: 'Zack Whittaker', age: '2h',
	},
];

/**
 * The record beside the selected entry. An em dash is the honest value for a
 * field that has not been fetched — do not fill these in with plausible data.
 */
export const FEED_RECORD: {
	title: string;
	domain: string;
	author: string;
	excerpt: string;
	fields: readonly RecordField[];
} = {
	title: 'The View from Above: The Gemini Visual Acuity Experiments',
	domain: 'nasa.gov',
	author: 'Michele …',
	excerpt:
		'NASA astronaut L. Gordon Cooper, Jr. took 29 color photographs of the Earth with a 70mm camera as he orbited our planet during the Mercury-Atlas 9 mission in May 1963. Cooper’s view from the window of his Faith 7 spacecraft was spectacular, and he reported that he could see vehicles motoring on dirt roads, smoke-belching trains, […]',
	fields: [
		{ label: 'Type', value: 'Article' },
		{ label: 'Domain', value: 'nasa.gov' },
		{ label: 'Published', value: 'Aug 20, 2026' },
		{ label: 'Length', value: '—' },
		{ label: 'Words', value: '—' },
		{ label: 'Saved', value: '16 minutes ago' },
		{ label: 'Progress', value: '', progress: 0 },
		{ label: 'Last read', value: '—' },
		{ label: 'Language', value: '—' },
	],
};
