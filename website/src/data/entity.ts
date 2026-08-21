/**
 * The entity view's subject and its references.
 *
 * The live page reads "8 documents" while listing 7, because one row is a
 * "Post Not Found" record that is deliberately not shown here. Rather than
 * reproduce an off-by-one, the counts below are derived from the list, so the
 * frame is internally consistent.
 */

export const ENTITY = {
	name: 'OpenAI',
	kind: 'Organization',
	about: 'Developer of GPT and ChatGPT models',
	references: 74,
	firstSeen: 'Aug 18, 2026',
	lastSeen: 'Aug 20, 2026',
} as const;

export interface Mention {
	name: string;
	count: number;
}

export const CO_MENTIONS: readonly Mention[] = [
	{ name: 'GPT-3', count: 3 },
	{ name: 'Anthropic', count: 3 },
	{ name: 'Attention Is All You Need', count: 3 },
	{ name: 'BERT', count: 2 },
	{ name: 'ImageNet', count: 2 },
	{ name: 'Google', count: 2 },
	{ name: 'TriviaQA', count: 2 },
	{ name: 'DeepSeek', count: 2 },
	{ name: 'Kimi K3', count: 2 },
	{ name: 'The Beatles', count: 1 },
];

export interface EntityDoc {
	letter: string;
	/** Per-source hue, c1–c7. The live tiles are not one accent. */
	hue: string;
	title: string;
	domain?: string;
	date: string;
	excerpt?: string;
}

export const ENTITY_DOCS: readonly EntityDoc[] = [
	{
		letter: 'W',
		hue: 'c1',
		title: 'Alibaba’s lightweight Qwen model takes on larger AI systems from OpenAI, DeepSeek, Zhipu | South China Morning Post',
		domain: 'www.scmp.com',
		date: 'Aug 20, 2026',
		excerpt: 'Chinese tech giant’s latest offering performed on par with OpenAI’s GPT-5.6 Luna and nearly matched DeepSeek, Zhipu’s open-weight models.',
	},
	{
		letter: 'M',
		hue: 'c2',
		title: 'OpenAI Astra: The mysterious new quantum math-solving model | Mashable',
		domain: 'mashable.com',
		date: 'Aug 20, 2026',
		excerpt: 'Astra is an unreleased new OpenAI model, which has already solved 10 major math problems that stood open for decades.',
	},
	{ letter: 'E', hue: 'c3', title: 'Attention Is All You Need - Wikipedia', domain: 'en.wikipedia.org', date: 'Aug 20, 2026' },
	{ letter: 'L', hue: 'c4', title: 'Learning Transferable Visual Models From Natural Language Supervision', date: 'Aug 19, 2026' },
	{ letter: 'B', hue: 'c5', title: 'bert', date: 'Aug 19, 2026' },
	{ letter: 'L', hue: 'c6', title: 'language-models-few-shot', date: 'Aug 19, 2026' },
	{ letter: 'E', hue: 'c7', title: 'Large language model', domain: 'en.wikipedia.org', date: 'Aug 19, 2026' },
];
