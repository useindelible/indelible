/**
 * The novel the "Built for the long ones" section is about.
 *
 * 336,952 words, saved from the browser extension. The numbers in the section
 * copy are this document's, so the two must not drift: if the record below
 * changes, the lede in components/home/LongReads.astro changes with it.
 *
 * Progress is 0% and Last read is an em dash on purpose. The claim is that the
 * novel was READABLE in under a second, not that anyone has read it — filling
 * those in would be inventing evidence for a claim the page does not make.
 */
import type { ReaderDocument } from '../lib/types';

export const LONGREAD: ReaderDocument = {
	title: 'Martin Chuzzlewit by Charles Dickens – Read Online Free - Ririro',
	author: 'pim@ririro.com',
	domain: 'ririro.com',
	published: 'March 28, 2026',
	length: '1416 min read',
	/* Near the top of a very long document: the position is genuinely 0%. */
	tick: 1,
	ticks: 24,

	blocks: [
		{ kind: 'h2', text: 'Preface' },
		{
			kind: 'p',
			text: 'What is exaggeration to one class of minds and perceptions, is plain truth to another. That which is commonly called a long-sight, perceives in a prospect innumerable features and bearings non-existent to a short-sighted person. I sometimes ask myself whether there may occasionally be a difference of this kind between some writers and some readers; whether it is always the writer who colours highly, or whether it is now and then the reader whose eye for colour is a little dull?',
			emphasis: 'always',
		},
		{
			kind: 'p',
			text: 'On this head of exaggeration I have a positive experience, more curious than the speculation I have just set down. It is this: I have never touched a character precisely from the life, but some counterpart of that character has incredulously asked me: “Now really, did I ever really, see one like it?”',
		},
		{
			kind: 'p',
			text: 'All the Pecksniff family upon earth are quite agreed, I believe, that Mr Pecksniff is an exaggeration, and that no such character ever existed. I will not offer any plea on his behalf to so powerful and genteel a body, but will make a remark on the character of Jonas Chuzzlewit.',
		},
		{
			kind: 'p',
			text: 'I conceive that the sordid coarseness and brutality of Jonas would be unnatural, if there had been nothing in his early education, and in the precept and example always before him, to engender and develop the vices that make him odious. But, so born and so bred, admired for that which made him hateful, and justified from his cradle in cunning, treachery, and avarice; I claim him as the legitimate issue of the father upon whom those vices are seen to recoil. And I submit that their recoil upon that old man, in his unhonoured age, is not a mere piece of poetical justice, but is the extreme exposition of a direct truth.',
		},
	],

	record: {
		author: { name: 'pim', handle: 'ririro.com' },
		summary:
			'Martin Chuzzlewit examines greed, hypocrisy, family conflict, and the moral consequences of selfishness. Martin Chuzzlewit distrusts relatives who seek his wealth, while Mr Pecksniff disguises manipulation and avarice beneath moral language and benevolent gestures. Tom Pinch remains sincere and trusting despite Pecksniff’s exploitation, and the Chuzzlewit family gathers in competing factions to pursue Martin’s property. The opening establishes Dickens’s comic exposure of false virtue and argues that vice grows from education, example, and social neglect.',
		fields: [
			{ label: 'Type', value: 'Article' },
			{ label: 'Domain', value: 'ririro.com' },
			{ label: 'Published', value: 'Mar 28, 2026' },
			{ label: 'Length', value: '23h 36m read' },
			{ label: 'Words', value: '336,952 words' },
			{ label: 'Saved', value: '1 day ago' },
			{ label: 'Progress', value: '', progress: 0 },
			{ label: 'Last read', value: '—' },
		],
	},
};
