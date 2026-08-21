/**
 * The EPUB open in the book reader.
 *
 * Moby-Dick from Project Gutenberg, at page 1 of 866 — deliberately at 0%
 * progress, because that is what a book looks like the moment it lands and it
 * is the honest way to show the position memory has nothing to remember yet.
 */
import type { RecordField, RecordGroup } from '../components/screens/app/RecordPanel.astro';

export interface Chapter {
	/** Chapter number where the book gives one; sections have none. */
	no?: string;
	name: string;
	/** Page number where the book gives one. */
	page?: string;
	current?: boolean;
	/** A section divider rather than a chapter. */
	section?: boolean;
}

export const BOOK = {
	title: 'Moby Dick; Or, The Whale',
	author: 'Herman Melville',
	progress: 0,
	page: 'Page 1 of 866',
	chapterLabel:
		'Ch. 1: (Supplied by a Late Consumptive Usher to a Grammar School.)',
	footPrev: '‹',
	footMiddle: '1 of 11 chapters',
	footNext: 'Ch. 2: CHAPTER 9. The Sermon. ›',
	tabs: ['Contents', 'Bookmarks', 'Search'],
} as const;

export const BOOK_CHAPTERS: readonly Chapter[] = [
	{ name: 'MOBY-DICK; OR, THE WHALE.', current: true },
	{ name: 'ORIGINAL TRANSCRIBER’S NOTES:', section: true },
	{ name: 'ETYMOLOGY.', section: true },
	{ no: '1', name: '(Supplied by a Late Consumptive Usher to a Grammar School.)', page: '1' },
	{ name: 'EXTRACTS. (SUPPLIED BY A SUB-SUB-LIBRARIAN).', section: true },
	{ no: '2', name: 'EXTRACTS.', page: '1' },
	{ name: 'CHAPTER 1. LOOMINGS.' },
	{ name: 'CHAPTER 2. THE CARPET-BAG.' },
	{ name: 'CHAPTER 3. THE SPOUTER-INN.' },
	{ name: 'CHAPTER 4. THE COUNTERPANE.' },
	{ name: 'CHAPTER 5. BREAKFAST.' },
	{ name: 'CHAPTER 6. THE STREET.' },
	{ name: 'CHAPTER 7. THE CHAPEL.' },
	{ name: 'CHAPTER 8. THE PULPIT.' },
	{ name: 'CHAPTER 9. THE SERMON.' },
];

/** The whaling-scene cover, coded like every other image on the page. */
export const BOOK_COVER =
	'<span class="cvr">' +
	'<i style="inset:0;background:linear-gradient(170deg,#A2957A,#5C5340)"></i>' +
	'<i style="left:6%;top:46%;width:70%;height:26%;background:#3A3427;border-radius:52% 40% 30% 60%"></i>' +
	'<i style="left:52%;top:30%;width:30%;height:16%;background:#4A4331;border-radius:60% 40% 50% 50%"></i>' +
	'</span>';

export const BOOK_RECORD = {
	title: 'Moby Dick; Or, The Whale',
	subtitle: 'Herman Melville',
	summary:
		'Whaling and the sea frame Ishmael’s search for purpose, escape, and knowledge as he joins the Pequod and prepares to pursue the great white whale Moby Dick. His friendship with the tattooed harpooneer Queequeg challenges assumptions about civilization, religion, and difference, while Father Mapple’s sermon presents obedience, repentance, and truth as central moral concerns. The novel combines maritime adventure, whale lore, spiritual reflection, and Ahab’s growing pursuit into a meditation on human fellowship, fate, and the dangers of obsession.',
	fields: [
		{ label: 'Type', value: 'Book (EPUB)' },
		{ label: 'Length', value: '866 pages' },
		{ label: 'Saved', value: 'Today' },
		{ label: 'Language', value: 'en' },
	] satisfies RecordField[],
	groups: [
		{
			heading: 'Reading progress',
			fields: [
				{ label: 'Progress', value: '0% (0 of 866 pages)' },
				{ label: 'Remaining', value: '~14h estimated' },
				{ label: 'Last read', value: 'Just now' },
			],
		},
	] satisfies RecordGroup[],
};
