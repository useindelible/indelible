/**
 * The article shown in every reader surface.
 *
 * The record — title, source, author, length, date — is the real saved
 * document, so desktop and phone readers agree with the rest of the corpus.
 *
 * The BODY is original prose on the same subject. The live page's wording
 * belongs to its author, and a marketing site is not the place to reproduce
 * someone else's article in full. Keep it that way.
 */

export interface ReaderParagraph {
	text: string;
	/** Optional inline highlight, matched as a substring of `text`. */
	highlight?: { phrase: string; colour: 'y' | 'b' };
}

export const ARTICLE = {
	title: 'I Tested DeepSeek vs Qwen vs Kimi vs GLM — Here’s the Winner',
	source: 'dev.to',
	author: 'fiercedash',
	length: '9 min',
	date: '16 Aug',
} as const;

export const ARTICLE_BODY: readonly ReaderParagraph[] = [
	{
		text: 'Four labs, one week, and a spreadsheet that got out of hand. I ran the same eleven prompts through DeepSeek, Qwen, Kimi and GLM and scored them on the things I actually ship: instruction following, long-context recall, and how often I had to run it twice.',
	},
	{
		text: 'The headline is boring and worth saying anyway: on most of my everyday work the gap between them is smaller than the gap between any of them and a bad prompt. Where they separate is cost and context, and that is where the interesting trade lives.',
		highlight: {
			phrase: 'the gap between them is smaller than the gap between any of them and a bad prompt',
			colour: 'y',
		},
	},
	{
		text: 'One number kept coming back. Moving a non-critical summarisation job off a frontier model cut that job’s bill by about eighty percent with no measurable quality loss, which is the kind of result that makes you re-read your own logs twice.',
		highlight: { phrase: 'about eighty percent', colour: 'b' },
	},
];

export const ARTICLE_HEADING = 'Where the four actually differ';
