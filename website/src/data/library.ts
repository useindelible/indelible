/**
 * The document corpus shown across every product screen.
 *
 * One source of truth on purpose: a desktop screen and its phone counterpart
 * must show the same documents, and the fastest way to break that is to let
 * each screen carry its own list.
 *
 * Ported verbatim from the design source. Do not paraphrase entries — the
 * summaries and covers are what make two screens visibly agree.
 */

export interface Doc {
	title: string;
	summary: string;
	source: string;
	author: string;
	/** "8 min", or a type word such as "Book" / "PDF". */
	length: string;
	age: string;
	/** Reading progress, 0-100. */
	progress: number;
	/** Letter-mark tint for the source badge. */
	accent: string;
	/** Coded cover: an emoji on a tinted ground. */
	emoji: string;
	tint: string;
}

export const DOCS: readonly Doc[] = [
	{
		title: "Building Resilient Distributed Systems with Rust",
		summary:
			"Error propagation with typed results, exponential backoff with jitter, and circuit breakers in production services.",
		source: "blog.yoshuawuyts.com",
		author: "Yoshua Wuyts",
		length: "8 min",
		age: "2h ago",
		progress: 62,
		accent: "#0071E3",
		emoji: "📐",
		tint: "linear-gradient(150deg,#DCE9FF,#C4DBFF)",
	},
	{
		title: "The Bitter Lesson of AI Scaling Laws",
		summary:
			"Why compute-driven approaches consistently beat hand-engineered solutions, and what it means for research.",
		source: "arxiv.org",
		author: "Rich Sutton",
		length: "12 min",
		age: "1d ago",
		progress: 0,
		accent: "#34C759",
		emoji: "🧠",
		tint: "linear-gradient(150deg,#EFDCFF,#E2C6FB)",
	},
	{
		title: "SQLite Is Not a Toy Database",
		summary:
			"When and why SQLite outperforms Postgres for read-heavy workloads, embedded systems and edge compute.",
		source: "antonz.org",
		author: "Anton Zhiyanov",
		length: "6 min",
		age: "3d ago",
		progress: 23,
		accent: "#FF9500",
		emoji: "⚡",
		tint: "linear-gradient(150deg,#FFE9C7,#FFD79B)",
	},
	{
		title: "The End of the Beginning",
		summary:
			"A paradox at the heart of the technology industry: the more transformative a platform becomes, the less visible it feels.",
		source: "stratechery.com",
		author: "Ben Thompson",
		length: "15 min",
		age: "6d ago",
		progress: 88,
		accent: "#0071E3",
		emoji: "🌊",
		tint: "linear-gradient(150deg,#D7F0FF,#B9E0FA)",
	},
	{
		title: "How to Do Great Work",
		summary:
			"Curiosity, and the discipline of following it past the point where it stops being comfortable.",
		source: "paulgraham.com",
		author: "Paul Graham",
		length: "24 min",
		age: "2w ago",
		progress: 41,
		accent: "#34C759",
		emoji: "✍️",
		tint: "linear-gradient(150deg,#D8F3DE,#BCE7C8)",
	},
	{
		title: "Zero-Knowledge Proofs Explained Simply",
		summary:
			"From the cave analogy to modern cryptographic protocols, without the mathematics getting in the way.",
		source: "blog.chain.link",
		author: "Chainlink Labs",
		length: "10 min",
		age: "3w ago",
		progress: 0,
		accent: "#FF3B30",
		emoji: "🔒",
		tint: "linear-gradient(150deg,#FFDCE4,#FBC3D2)",
	},
	{
		title: "Designing Data-Intensive Applications",
		summary:
			"Replication, partitioning, transactions and the trade-offs behind reliable distributed architectures.",
		source: "dataintensive.net",
		author: "Martin Kleppmann",
		length: "Book",
		age: "1mo ago",
		progress: 68,
		accent: "#0071E3",
		emoji: "📖",
		tint: "linear-gradient(150deg,#E3E8FF,#CBD5FB)",
	},
	{
		title: "Time, Clocks and the Ordering of Events",
		summary:
			"The foundational paper on logical clocks and causal ordering that underpins modern consensus work.",
		source: "lamport.azurewebsites.net",
		author: "Leslie Lamport",
		length: "PDF",
		age: "1mo ago",
		progress: 100,
		accent: "#0071E3",
		emoji: "🕰️",
		tint: "linear-gradient(150deg,#DDF2F7,#BFE4EE)",
	},
	{
		title: "The Fallacies of Distributed Computing",
		summary:
			"Eight assumptions every engineer building networked systems eventually has to unlearn.",
		source: "architecturenotes.co",
		author: "Mahdi Yusuf",
		length: "9 min",
		age: "6w ago",
		progress: 0,
		accent: "#FF9500",
		emoji: "📡",
		tint: "linear-gradient(150deg,#FFE3D4,#FFCAB0)",
	},
	{
		title: "What the Forest Knows",
		summary:
			"Fungal networks, nutrient trading and the slow intelligence of an old wood.",
		source: "emergencemagazine.org",
		author: "Robert Macfarlane",
		length: "14 min",
		age: "2mo ago",
		progress: 12,
		accent: "#34C759",
		emoji: "🌿",
		tint: "linear-gradient(150deg,#DFF3E4,#C2E7CD)",
	},
	{
		title: "The Weekly Craft newsletter",
		summary:
			"Issue 147 on reading fewer things but taking better notes on the ones that matter.",
		source: "thesundaydispatch.com",
		author: "Newsletter",
		length: "7 min",
		age: "2mo ago",
		progress: 0,
		accent: "#B0252B",
		emoji: "📮",
		tint: "linear-gradient(150deg,#FFEFD6,#FFDDAE)",
	},
	{
		title: "Aggregation Theory in the Age of Assistants",
		summary:
			"When the interface becomes a model, who actually owns the demand it aggregates.",
		source: "stratechery.com",
		author: "Ben Thompson",
		length: "11 min",
		age: "3mo ago",
		progress: 0,
		accent: "#8442D9",
		emoji: "🧭",
		tint: "linear-gradient(150deg,#E7E1FF,#D2C6FA)",
	},
];

export interface Entity {
	name: string;
	count: string;
}

/**
 * Entities Mila extracted, keyed by index into DOCS.
 *
 * A document with no entry falls back to its own author and source. That is
 * the least wrong thing to show: an earlier version hard-coded one document's
 * people onto every screen, so a book and a talk both listed a Rust essay's
 * entities.
 */
const DOC_ENTITIES: Record<number, readonly Entity[]> = {
	0: [
		{ name: 'Yoshua Wuyts', count: '7 docs' },
		{ name: 'Rust Foundation', count: '19 docs' },
		{ name: 'Tokio', count: '12 docs' },
		{ name: 'Async Rust', count: '9 docs' },
	],
	4: [
		{ name: 'Paul Graham', count: '11 docs' },
		{ name: 'Y Combinator', count: '14 docs' },
		{ name: 'Lisp', count: '4 docs' },
		{ name: 'Cambridge', count: '6 docs' },
	],
	5: [
		{ name: 'Chainlink Labs', count: '5 docs' },
		{ name: 'zk-SNARK', count: '8 docs' },
		{ name: 'Ethereum', count: '17 docs' },
		{ name: 'Vitalik Buterin', count: '6 docs' },
	],
};

export function entitiesFor(doc: Doc, index?: number): readonly Entity[] {
	if (index !== undefined && DOC_ENTITIES[index]) return DOC_ENTITIES[index];

	const site = doc.source.split('.')[0];
	return [
		{ name: doc.author, count: '7 docs' },
		{ name: site.charAt(0).toUpperCase() + site.slice(1), count: '12 docs' },
	];
}

export interface Collection {
	emoji: string;
	tint: string;
	name: string;
	description: string;
	items: string;
	/** Empty when the collection has no children. */
	sub?: string;
}

export const COLLECTIONS: readonly Collection[] = [
	{ emoji: '🔬', tint: 'linear-gradient(150deg,#DCE9FF,#C4DBFF)', name: 'Research', description: 'Papers, articles and deep dives on distributed systems.', items: '23 items', sub: '3 sub-collections' },
	{ emoji: '✍️', tint: 'linear-gradient(150deg,#EFDCFF,#E2C6FB)', name: 'Writing craft', description: 'Essays on craft, process, editing and publishing.', items: '15 items' },
	{ emoji: '🌍', tint: 'linear-gradient(150deg,#D8F3DE,#BCE7C8)', name: 'Climate and energy', description: 'Energy transition, climate policy and carbon removal.', items: '31 items', sub: '2 sub-collections' },
	{ emoji: '🚀', tint: 'linear-gradient(150deg,#FFE9C7,#FFD79B)', name: 'Startups', description: 'Fundraising, go to market and team building.', items: '19 items' },
	{ emoji: '💰', tint: 'linear-gradient(150deg,#D7F0FF,#B9E0FA)', name: 'Personal finance', description: 'Index funds, tax strategy and compound interest.', items: '12 items' },
	{ emoji: '🎨', tint: 'linear-gradient(150deg,#FFDCE4,#FBC3D2)', name: 'Design inspiration', description: 'Interfaces, typography and spatial design.', items: '8 items' },
	{ emoji: '🧭', tint: 'linear-gradient(150deg,#E7E1FF,#D2C6FA)', name: 'AI research', description: 'Papers and essays on models, retrieval and evaluation.', items: '27 items', sub: '4 sub-collections' },
	{ emoji: '📮', tint: 'linear-gradient(150deg,#FFEFD6,#FFDDAE)', name: 'Newsletters', description: 'Everything that arrives by email, kept and searchable.', items: '44 items' },
	{ emoji: '🗂️', tint: 'linear-gradient(150deg,#DDF2F7,#BFE4EE)', name: 'Reference shelf', description: 'Specs, papers and manuals worth keeping permanently.', items: '18 items', sub: '2 sub-collections' },
];

export interface FilterCondition {
	field: string;
	operator: string;
	value: string;
}

/**
 * A saved view's rule. The first row reads "Where" and the rest read "And";
 * the conjunction is a real toggle in the app that can become "Or".
 */
export const SAVED_VIEW_RULE: readonly FilterCondition[] = [
	{ field: 'Content type', operator: 'is', value: 'Article' },
	{ field: 'Domain', operator: 'is', value: 'stratechery.com' },
	{ field: 'Tag', operator: 'is not', value: 'long reads' },
];
