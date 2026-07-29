import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import svelte from '@astrojs/svelte';

export default defineConfig({
	site: 'https://useindelible.com',
	redirects: {
		'/docs': '/docs/getting-started/introduction/',
	},
	integrations: [
		starlight({
			title: 'Indelible',
			description: 'Open-source, self-hosted read-it-later and knowledge archiver.',
			logo: { src: './src/assets/logo.svg', alt: 'Indelible' },
			favicon: '/favicon.svg',
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/useindelible/indelible' },
			],
			customCss: ['./src/styles/site-tokens.css', './src/styles/starlight-theme.css'],
			sidebar: [
				{ label: 'Getting Started', items: [{ autogenerate: { directory: 'docs/getting-started' } }] },
				{ label: 'Self-Hosting', items: [{ autogenerate: { directory: 'docs/self-hosting' } }] },
				{ label: 'How-To Guides', items: [{ autogenerate: { directory: 'docs/how-to' } }] },
				{ label: 'Reference', items: [{ autogenerate: { directory: 'docs/reference' } }] },
			],
		}),
		svelte(),
	],
});
