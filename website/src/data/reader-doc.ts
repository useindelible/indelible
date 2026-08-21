/**
 * The short document open in the desktop reader.
 *
 * Read off the live app rather than written for the page, which is why the
 * prose has the shape it does. Do not tidy it: the point of this screen is
 * that extraction keeps a real article intact, headings, rules and all.
 *
 * This is one of several documents the reader can be handed — see
 * data/longread.ts for the other. The screen holds no document of its own.
 */
import type { ReaderDocument } from '../lib/types';

export const READER_DOC: ReaderDocument = {
	title: 'I Tested DeepSeek vs Qwen vs Kimi vs GLM — Here’s the Winner - DEV Community',
	author: 'fiercedash',
	domain: 'dev.to',
	published: 'August 16, 2026',
	length: '9 min read',
	tick: 3,
	ticks: 24,

	blocks: [
		{
			kind: 'p',
			text: 'So here’s what happened: i Tested DeepSeek vs Qwen vs Kimi vs GLM — Here’s the Winner',
		},
		{
			kind: 'p',
			text: 'Okay, so I’ve been on this absolute rabbit hole for the past few weeks, and I have to share what I’ve found. You know how everyone’s been talking about GPT-4o and Claude, but there’s this whole other universe of Chinese AI models that are honestly punching way above their weight? Yeah, I went deep into it. Let me walk you through what I learned.',
		},
		{
			kind: 'p',
			text: 'If you’ve ever stared at a pricing page wondering which model to actually use for your side project, your startup’s chatbot, or that one client who’s been asking about cheaper alternatives — this is for you. I spent hours testing DeepSeek, Qwen, Kimi, and GLM through Global API’s unified endpoint, and I’m going to break it all down for you. No fluff, no marketing speak, just what actually works.',
		},
		{ kind: 'hr' },
		{ kind: 'h2', text: 'Why I Even Started Looking at Chinese Models' },
		{
			kind: 'p',
			text: 'Let me be honest with you — I was skeptical at first. My mental model was “Western models = good, Chinese models = questionable.” Then a friend who runs a SaaS startup told me he cut his API bill by 80% by switching to DeepSeek for non-critical workloads. Eighty percent! I had to see for myself.',
		},
		{
			kind: 'p',
			text: 'The thing is, China’s AI scene has exploded in the last couple of years. You’ve got four major players — DeepSeek from High-Flyer (幻方), Qwen from Alibaba (阿里), Kimi from Moonshot AI (月之暗面), and GLM from Zhipu AI (智谱) — and each one has its own personality, if you will.',
		},
	],

	record: {
		author: { name: 'fiercedash' },
		summary:
			'Chinese AI models DeepSeek, Qwen, Kimi, and GLM offer competitive alternatives across price, coding, reasoning, language, and multimodal capabilities. DeepSeek V4 Flash provides the strongest overall price to performance, Qwen offers the broadest range of model sizes and features, Kimi specializes in premium reasoning, and GLM excels at Chinese language tasks. The comparison uses a unified OpenAI compatible API and recommends selecting models according to workload, with independent testing for specific needs.',
		fields: [
			{ label: 'Type', value: 'Article' },
			{ label: 'Domain', value: 'dev.to' },
			{ label: 'Published', value: 'Aug 16, 2026' },
			{ label: 'Length', value: '9 min read' },
			{ label: 'Words', value: '1,990 words' },
			{ label: 'Saved', value: '56 minutes ago' },
			{ label: 'Progress', value: '', progress: 100 },
			{ label: 'Last read', value: '2 minutes ago' },
		],
	},
};

/** The Listen transport, when a document is being read aloud. */
export const READER_TTS = {
	elapsed: '02:14',
	total: '14:32',
	/** Scrubber position as a percentage of the total. */
	position: 15,
	speed: '1.25×',
	voice: 'Mila',
	persona: 'Calm',
} as const;
