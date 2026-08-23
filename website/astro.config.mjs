import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';
import starlight from '@astrojs/starlight';
import svelte from '@astrojs/svelte';

export default defineConfig({
	site: 'https://useindelible.com',
	redirects: {
		'/docs': '/docs/getting-started/introduction/',
	},
	integrations: [
		sitemap({
			filter: (page) => page !== 'https://useindelible.com/screens/',
		}),
		starlight({
			title: 'Indelible',
			description: 'Open-source, self-hosted read-it-later and knowledge archiver.',
			logo: { src: './src/assets/logo.svg', alt: 'Indelible' },
			favicon: '/favicon.svg',
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/useindelible/indelible' },
			],
			// Code blocks quote a real shell, so — like the landing page's Terminal
			// primitive — they stay dark in both site themes. One dark syntax theme
			// rather than a light/dark pair is what makes that true: with a single
			// theme Starlight's dark-mode switch has nothing to switch, and
			// useStarlightUiThemeColors defaults to false so its own chrome colours
			// do not fight the overrides below.
			expressiveCode: {
				themes: ['vitesse-dark'],
				// With one theme Starlight would still scope the variables to
				// [data-theme='vitesse-dark'], a selector nothing ever carries, and the
				// block would render unstyled. `false` emits them on the root instead.
				themeCssSelector: false,
				styleOverrides: {
					borderRadius: '4px',
					borderWidth: '1px',
					borderColor: 'var(--line-2)',
					codeFontFamily: 'var(--font-mono)',
					codeFontSize: '0.84rem',
					codeLineHeight: '1.75',
					codePaddingBlock: '1rem',
					codePaddingInline: '1.15rem',
					codeBackground: 'var(--term-bg-solid)',
					codeForeground: 'var(--term-ink)',
					scrollbarThumbColor: 'color-mix(in srgb, var(--term-ink) 16%, transparent)',
					scrollbarThumbHoverColor: 'color-mix(in srgb, var(--term-ink) 30%, transparent)',
					frames: {
						// The macOS traffic lights are somebody else's chrome. The bar
						// keeps its job — naming the block — and loses the costume.
						terminalTitlebarDotsOpacity: '0',
						terminalTitlebarBackground: 'var(--term-bg-solid)',
						terminalTitlebarForeground: 'color-mix(in srgb, var(--term-ink) 50%, transparent)',
						terminalTitlebarBorderBottomColor: 'color-mix(in srgb, var(--term-ink) 12%, transparent)',
						terminalBackground: 'var(--term-bg-solid)',
						editorBackground: 'var(--term-bg-solid)',
						editorTabBarBackground: 'var(--term-bg-solid)',
						editorTabBarBorderBottomColor: 'color-mix(in srgb, var(--term-ink) 12%, transparent)',
						editorTabBorderRadius: '0',
						editorActiveTabBackground: 'transparent',
						editorActiveTabForeground: 'var(--term-ink)',
						editorActiveTabBorderColor: 'transparent',
						editorActiveTabIndicatorTopColor: 'transparent',
						editorActiveTabIndicatorBottomColor: 'var(--accent)',
						editorActiveTabIndicatorHeight: '2px',
						editorTabsMarginInlineStart: '0',
						frameBoxShadowCssValue: 'none',
						inlineButtonBackground: 'var(--term-ink)',
						inlineButtonForeground: 'var(--term-ink)',
						inlineButtonBorder: 'color-mix(in srgb, var(--term-ink) 20%, transparent)',
						tooltipSuccessBackground: 'var(--accent)',
						tooltipSuccessForeground: 'var(--on-accent)',
					},
				},
			},
			// The docs read the same palette as the landing page: themes.css holds
			// the colour literals, tokens.css forwards them to semantic names, and
			// starlight-theme.css maps those onto Starlight's own variables.
			customCss: [
				'./src/styles/themes.css',
				'./src/styles/tokens.css',
				'./src/styles/starlight-theme.css',
			],
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
