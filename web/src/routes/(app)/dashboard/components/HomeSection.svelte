<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HomeItemResponse } from '$lib/api';
	import HomeItemCard from './HomeItemCard.svelte';

	interface Props {
		title: string;
		items?: HomeItemResponse[];
		loading?: boolean;
		seeAllHref?: string;
		onOpen?: (id: string) => void;
		children?: Snippet;
		actions?: Snippet;
	}

	let {
		title,
		items = [],
		loading = false,
		seeAllHref,
		onOpen,
		children,
		actions
	}: Props = $props();
</script>

<section class="section">
	<div class="section-header">
		<h2 class="section-title">{title}</h2>
		{#if actions}
			{@render actions()}
		{:else if seeAllHref}
			<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- callers pass resolved app hrefs into this route-local section component. -->
			<a href={seeAllHref} class="see-all">See all <span aria-hidden="true">-&gt;</span></a>
		{/if}
	</div>

	{#if children}
		{@render children()}
	{:else}
		<div class="card-carousel">
			{#if loading}
				{#each [0, 1, 2, 3] as skeleton (skeleton)}
					<div class="content-card skeleton">
						<div class="skeleton-thumb"></div>
						<div class="card-body">
							<div class="skeleton-line short"></div>
							<div class="skeleton-line"></div>
							<div class="skeleton-line narrow"></div>
						</div>
					</div>
				{/each}
			{:else}
				{#each items.slice(0, 6) as item (item.id)}
					<HomeItemCard {item} {onOpen} />
				{/each}
			{/if}
		</div>
	{/if}
</section>

<style>
	.section {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.section-header {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
	}

	.section-title {
		font-family: var(--font-sans);
		font-size: 20px;
		font-weight: 600;
		line-height: 1.25;
		color: var(--text-primary);
		margin: 0;
	}

	.see-all {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--accent);
		text-decoration: none;
		transition: color 150ms ease;
	}

	.see-all:hover {
		color: var(--accent-hover, var(--accent));
	}

	.card-carousel {
		display: flex;
		gap: 16px;
		overflow-x: auto;
		scroll-snap-type: x mandatory;
		-webkit-overflow-scrolling: touch;
		padding-bottom: 4px;
	}

	.card-carousel::-webkit-scrollbar {
		display: none;
	}

	.content-card {
		flex: 0 0 252px;
		border-radius: 12px;
		background: var(--bg-primary);
		border: 0.5px solid var(--border-primary);
		box-shadow: var(--shadow-1);
		overflow: hidden;
		scroll-snap-align: start;
	}

	.skeleton .skeleton-thumb {
		width: 100%;
		height: 140px;
		background: var(--border-primary);
		animation: shimmer 1.4s ease infinite;
	}

	.card-body {
		padding: 12px 14px;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.skeleton-line {
		height: 12px;
		border-radius: 6px;
		background: var(--border-primary);
		animation: shimmer 1.4s ease infinite;
	}

	.skeleton-line.short {
		width: 50%;
		height: 10px;
	}

	.skeleton-line.narrow {
		width: 70%;
	}

	@keyframes shimmer {
		0%,
		100% {
			opacity: 0.5;
		}
		50% {
			opacity: 1;
		}
	}

	@media (max-width: 599px) {
		.content-card {
			flex-basis: 232px;
		}
	}
</style>
