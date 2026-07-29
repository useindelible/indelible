<script lang="ts">
	import type { Snippet } from 'svelte';

	type Variant =
		| 'account'
		| 'reading'
		| 'integrations'
		| 'notion'
		| 'obsidian'
		| 'feed'
		| 'archival'
		| 'mila'
		| 'developer';

	interface Props {
		variant: Variant;
		children: Snippet;
	}

	let { variant, children }: Props = $props();
</script>

<section class="hero" data-variant={variant} aria-label="Page header">
	<div class="hero-inner">
		{@render children()}
	</div>
</section>

<style>
	.hero {
		position: relative;
		overflow: hidden;
		padding: 48px 56px 32px;
		border-bottom: 0.5px solid var(--border-hairline);
		/* The hero's available width depends on two collapsible sidebars, so
		   per-variant layout must query the hero itself, not the viewport. */
		container-type: inline-size;
		container-name: hero;
	}

	.hero::before {
		content: '';
		position: absolute;
		inset: 0;
		background:
			radial-gradient(540px 380px at 88% 20%, var(--hero-blob-a), transparent 60%),
			radial-gradient(620px 480px at 0% 110%, var(--hero-blob-b), transparent 60%),
			radial-gradient(420px 280px at 60% 100%, var(--hero-blob-c), transparent 60%),
			linear-gradient(135deg, var(--hero-from) 0%, var(--hero-to) 100%);
		z-index: 0;
	}

	.hero::after {
		content: '';
		position: absolute;
		left: 0;
		right: 0;
		bottom: 0;
		height: 1px;
		background: linear-gradient(90deg, transparent 0%, var(--hero-edge) 50%, transparent 100%);
		opacity: 0.6;
		z-index: 0;
	}

	.hero-inner {
		position: relative;
		z-index: 1;
		display: flex;
		align-items: center;
		gap: 28px;
		max-width: 920px;
	}

	.hero[data-variant='account'] {
		--hero-from: var(--hero-account-from);
		--hero-to: var(--hero-account-to);
		--hero-blob-a: var(--hero-account-blob-a);
		--hero-blob-b: var(--hero-account-blob-b);
		--hero-blob-c: transparent;
		--hero-edge: var(--hero-account-edge);
	}

	.hero[data-variant='reading'] {
		--hero-from: var(--hero-reading-from);
		--hero-to: var(--hero-reading-to);
		--hero-blob-a: var(--hero-reading-blob-a);
		--hero-blob-b: var(--hero-reading-blob-b);
		--hero-blob-c: transparent;
		--hero-edge: var(--hero-reading-edge);
	}

	.hero[data-variant='integrations'] {
		--hero-from: var(--hero-integrations-from);
		--hero-to: var(--hero-integrations-to);
		--hero-blob-a: var(--hero-integrations-blob-a);
		--hero-blob-b: var(--hero-integrations-blob-b);
		--hero-blob-c: var(--hero-integrations-blob-c);
		--hero-edge: var(--hero-integrations-edge);
	}

	.hero[data-variant='notion'] {
		--hero-from: var(--hero-notion-from);
		--hero-to: var(--hero-notion-to);
		--hero-blob-a: var(--hero-notion-blob-a);
		--hero-blob-b: var(--hero-notion-blob-b);
		--hero-blob-c: transparent;
		--hero-edge: var(--hero-notion-edge);
	}

	.hero[data-variant='obsidian'] {
		--hero-from: var(--hero-obsidian-from);
		--hero-to: var(--hero-obsidian-to);
		--hero-blob-a: var(--hero-obsidian-blob-a);
		--hero-blob-b: var(--hero-obsidian-blob-b);
		--hero-blob-c: var(--hero-obsidian-blob-c);
		--hero-edge: var(--hero-obsidian-edge);
	}

	.hero[data-variant='feed'] {
		--hero-from: var(--hero-feed-from);
		--hero-to: var(--hero-feed-to);
		--hero-blob-a: var(--hero-feed-blob-a);
		--hero-blob-b: var(--hero-feed-blob-b);
		--hero-blob-c: var(--hero-feed-blob-c);
		--hero-edge: var(--hero-feed-edge);
	}

	.hero[data-variant='archival'] {
		--hero-from: var(--hero-archival-from);
		--hero-to: var(--hero-archival-to);
		--hero-blob-a: var(--hero-archival-blob-a);
		--hero-blob-b: var(--hero-archival-blob-b);
		--hero-blob-c: transparent;
		--hero-edge: var(--hero-archival-edge);
	}

	.hero[data-variant='mila'] {
		--hero-from: var(--hero-mila-from);
		--hero-to: var(--hero-mila-to);
		--hero-blob-a: var(--hero-mila-blob-a);
		--hero-blob-b: var(--hero-mila-blob-b);
		--hero-blob-c: var(--hero-mila-blob-c);
		--hero-edge: var(--hero-mila-edge);
	}

	.hero[data-variant='developer'] {
		--hero-from: var(--hero-developer-from);
		--hero-to: var(--hero-developer-to);
		--hero-blob-a: var(--hero-developer-blob-a);
		--hero-blob-b: var(--hero-developer-blob-b);
		--hero-blob-c: transparent;
		--hero-edge: var(--hero-developer-edge);
	}

	@media (max-width: 599px) {
		.hero {
			padding: 24px 16px 20px;
		}

		.hero-inner {
			gap: 18px;
		}
	}
</style>
