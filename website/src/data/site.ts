import type { NavLink } from '../lib/types';

export const SITE = {
	name: 'Indelible',
	tagline: 'Read it later. Keep it forever.',
	description:
		'Open-source read-it-later and knowledge archiver. Everything you save is captured in full, so your library never rots.',
	repo: 'https://github.com/useindelible/indelible',
	licence: 'AGPL-3.0',
} as const;

/**
 * The landing page's section links.
 *
 * Root-relative ('/#x', not '#x') because the header is shared: a bare anchor
 * points at the CURRENT page, so from /privacy every one of these was a link
 * that did nothing. Every href must also match a section id rendered by
 * src/pages/index.astro — '#apps' matched none, which is how it shipped dead.
 */
export const NAV: readonly NavLink[] = [
	{ label: 'Product', href: '/#surfaces' },
	{ label: 'Mila', href: '/#mila' },
	{ label: 'Apps', href: '/#mobile' },
	{ label: 'Integrations', href: '/#connect' },
	{ label: 'Self-host', href: '/#host' },
];

/**
 * The docs entry point.
 *
 * Kept out of NAV on purpose: NAV is in-page anchors, this leaves the page, and
 * it sits beside the CTA so it survives the width where the anchor list is
 * hidden. Before it existed the only route to the docs was the footer.
 */
export const DOCS_LINK: NavLink = {
	label: 'Docs',
	href: '/docs/getting-started/introduction/',
};

/** The stable release files needed before following the self-hosting guide. */
export const INSTALL_COMMANDS: readonly string[] = [
	'mkdir indelible && cd indelible',
	'curl -fsSLO https://github.com/useindelible/indelible/releases/latest/download/docker-compose.yml',
	'curl -fsSL https://github.com/useindelible/indelible/releases/latest/download/example.env -o .env',
];

/** Every way in and out, as shown in the hero strip. */
export const CAPABILITIES: readonly string[] = [
	'Web',
	'Android',
	'iOS',
	'Chrome extension',
	'Obsidian',
	'Notion',
	'Email in',
	'RSS',
	'PDF',
	'EPUB',
	'YouTube transcripts',
];

export interface FooterColumn {
	heading: string;
	links: readonly NavLink[];
}

export const FOOTER: readonly FooterColumn[] = [
	{
		heading: 'Product',
		links: [
			{ label: 'Overview', href: '/#surfaces' },
			{ label: 'Mila', href: '/#mila' },
			{ label: 'Mobile apps', href: '/#mobile' },
			{ label: 'Integrations', href: '/#connect' },
		],
	},
	{
		heading: 'Docs',
		links: [
			{ label: 'Quick start', href: '/docs/getting-started/quick-start/' },
			{ label: 'Self-hosting', href: '/docs/self-hosting/install/' },
			{ label: 'Configuration', href: '/docs/reference/configuration/' },
			{ label: 'What to expect', href: '/docs/getting-started/limitations/' },
		],
	},
	{
		heading: 'Project',
		links: [
			{ label: 'GitHub', href: SITE.repo },
			{ label: 'Licence', href: SITE.repo + '/blob/main/LICENSE' },
			{ label: 'Security', href: SITE.repo + '/blob/main/SECURITY.md' },
			{ label: 'Privacy', href: '/privacy/' },
		],
	},
];
