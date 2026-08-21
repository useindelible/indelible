/** The record shown beside the search results: the selected hit's document. */
import type { RecordField, RecordEntity } from '../components/screens/app/RecordPanel.astro';
import { MILA_SUMMARY_CREDIT } from './app';

export const SEARCH_RECORD = {
	title: 'Beatles for Sale - Wikipedia',
	domain: 'en.wikipedia.org',
	author: 'Wikipedia',
	summary:
		'Beatles for Sale is the Beatles fourth studio album, released in December 1964 amid relentless touring and exhaustion. Its darker, more introspective songs draw on country music, rockabilly, and Bob Dylan, while several cover versions compensate for the limited supply of original material. Studio experimentation with fade in, guitar feedback, expanded percussion, and layered recording gives the album a transitional character as the band moves from upbeat pop toward more mature songwriting and sophisticated production.',
	summaryBy: MILA_SUMMARY_CREDIT,
	fields: [
		{ label: 'Type', value: 'Article' },
		{ label: 'Domain', value: 'en.wikipedia.org' },
		{ label: 'Published', value: 'Aug 22, 2002' },
		{ label: 'Length', value: '34 min read' },
		{ label: 'Words', value: '7,899 words' },
		{ label: 'Saved', value: '1 day ago' },
		{ label: 'Progress', value: '', progress: 72 },
		{ label: 'Last read', value: 'just now' },
		{ label: 'Language', value: 'English' },
	] satisfies RecordField[],
	entityHeading: 'People',
	entities: [
		{ name: 'John Lennon', count: '4 other docs' },
		{ name: 'Paul McCartney', count: '4 other docs' },
		{ name: 'George Harrison', count: '3 other docs' },
		{ name: 'Bob Dylan' },
		{ name: 'George Martin', count: '2 other docs' },
	] satisfies RecordEntity[],
} as const;
