<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		meta?: string;
		danger?: boolean;
		children: Snippet;
	}

	let { title, meta, danger = false, children }: Props = $props();
</script>

<section class="group">
	<div class="group-label" class:danger>
		<span>{title}</span>
		{#if meta}<span class="meta">{meta}</span>{/if}
	</div>
	{@render children()}
</section>

<style>
	.group {
		margin-bottom: 28px;
	}

	.group:last-of-type {
		margin-bottom: 0;
	}

	.group-label {
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		padding: 0 4px 8px;
		display: flex;
		align-items: baseline;
		gap: 8px;
	}

	.group-label.danger {
		color: var(--destructive);
	}

	.meta {
		font-weight: 500;
		text-transform: none;
		letter-spacing: -0.005em;
		color: var(--text-secondary);
		font-size: 12px;
	}
</style>
