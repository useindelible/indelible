/**
 * Preferences → Mila & AI.
 *
 * The default state: Mila runs on the included provider, with bring-your-own
 * switched off. That is the honest starting picture — the screen's argument is
 * that you CAN point it elsewhere, not that you already have.
 */
import type { AppIconName } from '../lib/icons';

export interface PrefsNavItem {
	icon: AppIconName;
	label: string;
}

export const PREFS_NAV: readonly PrefsNavItem[] = [
	{ icon: 'person', label: 'Account' },
	{ icon: 'books', label: 'Reading & Appear…' },
	{ icon: 'feed', label: 'Integrations' },
	{ icon: 'feed', label: 'Feed Management' },
	{ icon: 'emails', label: 'Email' },
	{ icon: 'archive', label: 'Archival' },
	{ icon: 'chat', label: 'Mila & AI' },
	{ icon: 'articles', label: 'Developer' },
];

export const MILA = {
	eyebrow: 'Your reading assistant',
	greeting: 'Hi, I’m Mila.',
	blurb:
		'I summarise long reads, pull out tags and entities, and help you find anything in your library — using your own model and your own key.',
	toggleTitle: 'Enable Mila',
	toggleSub: 'AI summaries, auto-tags, and reading assistant — on',
	providerNote: 'Powered by Indelible — included in your plan',
	byoTitle: 'Use my own AI provider',
	byoSub: 'Connect any OpenAI-compatible endpoint, including OpenRouter for Claude',
	indexTitle: 'Your library is ready',
	indexSub: 'Every eligible item is indexed and searchable.',
	indexed: 56,
	indexTotal: 56,
	indexChip: 'Platform default',
	/** One bar per indexed chunk; the count is decorative, the state is not. */
	bars: 48,
	presetNote: 'One default per action · Built-ins are read-only',
} as const;

export interface PromptPreset {
	colour: string;
	name: string;
	sub: string;
	builtIn: string;
	prompt: string;
}

export const PROMPT_PRESETS: readonly PromptPreset[] = [
	{
		colour: '#3B82F6',
		name: 'Summary',
		sub: 'Generated when an item is captured or you tap Summarise.',
		builtIn: 'Built-in Summary',
		prompt:
			'You write summaries for a personal reading library. Write one paragraph of 2 to 4 sentence…',
	},
	{
		colour: '#34C759',
		name: 'Tags',
		sub: 'Suggested topical tags for newly saved items.',
		builtIn: 'Built-in Tags',
		prompt:
			'You suggest retrieval-friendly tags for a personal reading library. Suggest 3 to 8 short lowerc…',
	},
];
