<script lang="ts">
	import type { HomeItemResponse } from '$lib/api';
	import { coverColor, domainInitial, readingMeta } from '../dashboard-model';

	interface Props {
		item: HomeItemResponse;
		onOpen?: (id: string) => void;
	}

	let { item, onOpen }: Props = $props();
	const meta = $derived(readingMeta(item));
	const progress = $derived(Math.max(0, Math.min(item.progress_percent ?? 0, 100)));
</script>

<button
	type="button"
	class="content-card"
	aria-label={`Open ${item.title}`}
	onclick={() => onOpen?.(item.id)}
>
	<div class="card-thumb">
		{#if item.thumbnail_url}
			<img class="card-cover" src={item.thumbnail_url} alt="" loading="lazy" />
		{:else}
			<div class="card-cover-placeholder cover-{coverColor(item.domain)}">
				<span class="cover-initial">{domainInitial(item.domain)}</span>
			</div>
		{/if}
		{#if progress > 0}
			<div class="card-thumb-progress" aria-label={`${progress}% read`}>
				<div class="card-thumb-progress-fill" style={`width: ${progress}%`}></div>
			</div>
		{/if}
	</div>
	<div class="card-body">
		<div class="card-source-row">
			<span class="card-favicon">{domainInitial(item.domain)}</span>
			<span class="card-domain">{item.domain ?? 'Unknown'}</span>
		</div>
		<div class="card-title">{item.title}</div>
		{#if meta}
			<div class="card-meta">{meta}</div>
		{/if}
	</div>
</button>

<style>
	.content-card {
		flex: 0 0 252px;
		border-radius: 12px;
		background: var(--bg-primary);
		border: 0.5px solid var(--border-primary);
		box-shadow: var(--shadow-1);
		overflow: hidden;
		cursor: pointer;
		transition:
			box-shadow 200ms ease,
			transform 200ms ease;
		scroll-snap-align: start;
		display: flex;
		flex-direction: column;
		text-align: left;
		padding: 0;
		font-family: var(--font-sans);
	}

	.content-card:hover {
		box-shadow: var(--shadow-3);
		transform: translateY(-2px);
	}

	:global([data-theme='dark']) .content-card {
		background: var(--bg-tertiary);
	}

	.card-thumb {
		width: 100%;
		height: 140px;
		overflow: hidden;
		position: relative;
		flex-shrink: 0;
	}

	.card-thumb-progress {
		position: absolute;
		bottom: 0;
		left: 0;
		right: 0;
		height: 3px;
		background: var(--fill-hover);
	}

	.card-thumb-progress-fill {
		height: 100%;
		background: var(--accent);
		border-radius: 0 2px 2px 0;
	}

	.card-cover {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}

	.card-cover-placeholder {
		width: 100%;
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--fill-selected);
	}

	.cover-blue {
		background: var(--highlight-blue-bg);
	}
	.cover-green {
		background: var(--highlight-green-bg);
	}
	.cover-purple {
		background: var(--highlight-purple-bg);
	}
	.cover-orange,
	.cover-red {
		background: var(--fill-warning);
	}
	.cover-teal {
		background: var(--fill-success);
	}
	.cover-pink {
		background: var(--highlight-pink-bg);
	}

	.cover-initial {
		font-family: var(--font-sans);
		font-size: 36px;
		font-weight: 700;
		color: var(--text-quaternary);
		user-select: none;
	}

	.card-body {
		padding: 12px 14px;
		display: flex;
		flex-direction: column;
		gap: 6px;
		flex: 1;
	}

	.card-source-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.card-favicon {
		width: 14px;
		height: 14px;
		border-radius: 3px;
		background: var(--accent);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 8px;
		font-weight: 700;
		color: var(--text-on-color);
		flex-shrink: 0;
		font-family: var(--font-sans);
	}

	.card-domain {
		font-size: 11.5px;
		font-weight: 500;
		color: var(--text-secondary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.card-title {
		font-size: 14px;
		font-weight: 600;
		line-height: 1.35;
		color: var(--text-primary);
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}

	.card-meta {
		font-size: 12px;
		font-weight: 400;
		line-height: 1.4;
		color: var(--text-tertiary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		margin-top: auto;
	}

	@media (max-width: 599px) {
		.content-card {
			flex-basis: 232px;
		}
	}
</style>
