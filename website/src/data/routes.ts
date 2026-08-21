/**
 * The ways a document gets in.
 *
 * The heading says six; these four cards enumerate six mechanisms between
 * them (extension, email, RSS, OPML, files, URLs). If a card is added or
 * split, check the heading still counts.
 */
export interface Route {
	/** Which status hue the dot uses. Decorative grouping, not a status. */
	tone: 'sky' | 'crimson' | 'amber' | 'moss';
	title: string;
	body: string;
}

export const ROUTES: readonly Route[] = [
	{
		tone: 'sky',
		title: 'Browser extension',
		body: 'Chrome, Edge and Firefox. Saves the page, projects your highlights back onto the original, and refuses duplicates.',
	},
	{
		tone: 'crimson',
		title: 'Save by email',
		body: 'Your own address. Forward a newsletter and it arrives cleaned, with the reply chain stripped.',
	},
	{
		tone: 'amber',
		title: 'RSS and OPML',
		body: 'Subscribe directly, import your whole reader in one file, or pick from suggested sources.',
	},
	{
		tone: 'moss',
		title: 'Files and URLs',
		body: 'Drop in PDFs and EPUBs, or paste a link from anywhere in the app.',
	},
];
