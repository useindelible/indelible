/**
 * Integrations → Notion and Obsidian, from the live settings pages.
 *
 * The workspace is named for the placeholder user in data/app.ts: it is the
 * same person's workspace, so the two screens have to agree.
 *
 * The identifiers below are the real ones the live workspace showed. They are
 * Notion object IDs, not secrets — a database ID is useless without the OAuth
 * grant — but they are also the shape a reader will recognise, which is the
 * point of showing them.
 */

export const NOTION = {
	workspace: 'Morgan Reid’s Space',
	database: 'Morgan Reid’s Space · Indelible',
	connectedOn: 'Connected on 20 August 2026',
	lastEdited: 'Last edited 22 minutes ago',
	pending: '0 pending',
	documents: '58',
	lastSync: '22 minutes ago',
	blurb:
		'Indelible exports every saved document, highlight, and note into a managed database in your Notion workspace. Append-only by default — your edits stay safe.',
	connectionNote:
		'The Notion workspace and database that Indelible writes to. Renaming properties in Notion is safe — Indelible matches by stored property ID.',
	syncNote:
		'Indelible appends new documents and highlights on a schedule. You can also push manually whenever you like.',
	exportNote:
		'Defaults match Readwise-style export behavior. Changes apply on the next sync.',
	ids: [
		{ label: 'Managed database', value: 'ac4f82f6-6094-4021-b67d-27f86b7be8bf' },
		{ label: 'Data source', value: 'cbaa2df2-2964-4bdc-b0a7-86e49a42bfa6' },
	],
	sync: [
		{ label: 'Status', value: 'Healthy', ok: true },
		{ label: 'Last sync', value: '20 Aug 2026, 18:23' },
		{ label: 'Pending jobs', value: '0' },
	],
	rows: [
		{ title: 'On the difficulty of writing', source: 'Substack', saved: 'May 3' },
		{ title: 'A short history of patience', source: 'Web', saved: 'May 1' },
		{ title: 'What it means to read again', source: 'Email', saved: 'Apr 29' },
	],
} as const;

export interface ObsidianRow {
	title: string;
	/** May contain <code> for path fragments; rendered as markup. */
	sub: string;
	right: 'in-obsidian' | 'generate' | 'switch-on' | 'switch-off';
}

export const OBSIDIAN = {
	blurb:
		'Server-rendered Markdown, append-only highlights, and granular formatting controls straight into your vault.',
	synced: '2h ago',
	syncNote: 'Vault writes happen from the Obsidian plugin',
	behaviourNote: 'How Indelible writes into your vault',
} as const;

export const OBSIDIAN_SYNC: readonly ObsidianRow[] = [
	{
		title: 'Manual vault sync',
		sub: 'Run this from the Obsidian plugin settings or command palette. The plugin downloads server-rendered artifacts and writes them into your open vault.',
		right: 'in-obsidian',
	},
	{
		title: 'Plugin settings',
		sub: 'Schedule, sync-on-open, deleted-file resync, and current-file reimport confirmation live in the Obsidian plugin’s settings.',
		right: 'in-obsidian',
	},
	{
		title: 'Plugin access token',
		sub: 'The Obsidian plugin authenticates with a personal access token scoped to exports. Generate one in Developer settings, then paste it into the plugin.',
		right: 'generate',
	},
];

export const OBSIDIAN_BEHAVIOUR: readonly ObsidianRow[] = [
	{
		title: 'Group in category folders',
		sub: 'PDFs use <code>books/</code>; videos and emails use <code>articles/</code>.',
		right: 'switch-on',
	},
	{
		title: 'Export full Reader documents',
		sub: 'Write a generated companion file when Indelible has a prepared readable asset for the document. Note templates control the linked export note, not this companion body.',
		right: 'switch-off',
	},
	{
		title: 'Append sync notifications',
		sub: 'Add a timestamped line to <code>Indelible/Indelible Syncs.md</code> after each run.',
		right: 'switch-off',
	},
];

/** The Obsidian mark: a faceted gem, drawn rather than shipped as a logo file. */
export const OBSIDIAN_GEM = `<svg viewBox="0 0 200 200" fill="none" aria-hidden="true">
<g stroke="rgba(190,155,255,.30)" stroke-width="1">
<path d="M100 8 190 100 100 192 10 100z"/><path d="M100 8v184M10 100h180"/>
<path d="M100 8 46 100l54 92 54-92z"/><path d="M46 100h108"/></g>
<path d="M100 26 168 100 100 174 32 100z" fill="rgba(150,105,240,.55)"/>
<path d="M100 26 100 174 32 100z" fill="rgba(120,80,215,.55)"/>
<path d="M100 26 168 100 100 174z" fill="rgba(178,140,255,.45)"/>
<path d="M100 26 168 100 100 174 32 100z" stroke="rgba(214,190,255,.7)" stroke-width="1.4"/>
</svg>`;
