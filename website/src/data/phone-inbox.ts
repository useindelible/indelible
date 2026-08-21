/**
 * The inbox on a phone: the library as one column.
 *
 * A row shows either reading progress or a saved stamp, never both — an item
 * you have opened is described by where you are in it, one you have not by
 * when it arrived.
 */

export interface PhoneInboxItem {
	tint: string;
	emoji: string;
	unread?: boolean;
	source: string;
	length: string;
	title: string;
	summary: string;
	progress?: number;
	progressLabel?: string;
	stamp?: string;
}

export const PHONE_INBOX_HEAD = {
	eyebrow: 'Your library',
	title: 'Inbox',
	count: '42',
	meter: { unread: 42, reading: 5, done: 19, third: '19 done' },
} as const;

export const PHONE_INBOX_FILTERS = [
	{ label: 'All', count: '66', on: true },
	{ label: 'Articles', count: '38', on: false },
	{ label: 'Video', count: '11', on: false },
	{ label: 'PDF', count: '9', on: false },
	{ label: 'Email', count: '8', on: false },
];

export const PHONE_INBOX_ITEMS: readonly PhoneInboxItem[] = [
	{
		tint: 't-indigo', emoji: '📐',
		source: 'blog.yoshuawuyts.com', length: '8 min',
		title: 'Building Resilient Distributed Systems with Rust',
		summary:
			'Error propagation with typed results, exponential backoff with jitter, and circuit breakers in production services.',
		progress: 62, progressLabel: '62% / 3 min left',
	},
	{
		tint: 't-plum', emoji: '🧠', unread: true,
		source: 'arxiv.org', length: '12 min',
		title: 'The Bitter Lesson of AI Scaling Laws',
		summary:
			'Why compute-driven approaches consistently beat hand-engineered solutions, and what it means for research.',
		stamp: 'Saved 1d ago',
	},
	{
		tint: 't-clay', emoji: '⚡', unread: true,
		source: 'antonz.org', length: '6 min',
		title: 'SQLite Is Not a Toy Database',
		summary:
			'When and why SQLite outperforms Postgres for read-heavy workloads, embedded systems and edge compute.',
		progress: 23, progressLabel: '23% / 5 min left',
	},
	{
		tint: 't-slate', emoji: '🌊',
		source: 'stratechery.com', length: '15 min',
		title: 'The End of the Beginning',
		summary:
			'A paradox at the heart of the technology industry: the more transformative a platform becomes, the less visible it feels.',
		progress: 88, progressLabel: '88% / 2 min left',
	},
	{
		tint: 't-moss', emoji: '✍️',
		source: 'paulgraham.com', length: '24 min',
		title: 'How to Do Great Work',
		summary:
			'Curiosity, and the discipline of following it past the point where it stops being comfortable.',
		progress: 41, progressLabel: '41% / 14 min left',
	},
	{
		tint: 't-plum', emoji: '🔒', unread: true,
		source: 'blog.chain.link', length: '10 min',
		title: 'Zero-Knowledge Proofs Explained Simply',
		summary:
			'From the cave analogy to modern cryptographic protocols, without the mathematics getting in the way.',
		stamp: 'Saved 3w ago',
	},
];
