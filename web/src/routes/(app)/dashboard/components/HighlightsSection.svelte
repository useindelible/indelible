<script lang="ts">
	import { resolve } from '$app/paths';
	import type { HighlightWithNoteResponse } from '$lib/api';
	import { t } from '$lib/i18n';

	interface Props {
		highlights: HighlightWithNoteResponse[];
		loading?: boolean;
	}

	let { highlights, loading = false }: Props = $props();
</script>

<section class="section">
	<div class="section-header">
		<h2 class="section-title">{$t('dashboard_section_highlights')}</h2>
	</div>

	{#if loading}
		<div class="hl-list">
			{#each [0, 1, 2, 3] as skeleton (skeleton)}
				<div class="hl-item skeleton">
					<div class="hl-bar"></div>
					<div class="hl-content">
						<div class="skeleton-line"></div>
						<div class="skeleton-line narrow"></div>
					</div>
				</div>
			{/each}
		</div>
	{:else if highlights.length === 0}
		<div class="hl-empty">
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.3"
				stroke-linecap="round"
				stroke-linejoin="round"
				aria-hidden="true"
				width="32"
				height="32"
			>
				<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
				<path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
			</svg>
			<span>{$t('dashboard_highlights_empty')}</span>
		</div>
	{:else}
		<div class="hl-list">
			{#each highlights as highlight (highlight.id)}
				<div class="hl-item hl-color-{highlight.color}">
					<div class="hl-bar" aria-hidden="true"></div>
					<div class="hl-content">
						<p class="hl-text">{highlight.text_content}</p>
						{#if highlight.item_title && highlight.document_id}
							<a
								class="hl-source"
								href={resolve('/(app)/reader/[documentId]', {
									documentId: highlight.document_id
								})}>{highlight.item_title}</a
							>
						{/if}
					</div>
				</div>
			{/each}
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

	.hl-empty {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 20px 16px;
		border-radius: 10px;
		border: 0.5px solid var(--border-primary);
		color: var(--text-tertiary);
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
	}

	.hl-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.hl-item {
		display: flex;
		align-items: stretch;
		gap: 12px;
		border-radius: 8px;
		border: 0.5px solid var(--border-primary);
		overflow: hidden;
		background: var(--bg-secondary);
	}

	.hl-bar {
		width: 3px;
		flex-shrink: 0;
		border-radius: 0;
		background: var(--border-secondary);
	}

	.hl-color-yellow .hl-bar {
		background: var(--highlight-yellow-border);
	}
	.hl-color-blue .hl-bar {
		background: var(--highlight-blue-border);
	}
	.hl-color-green .hl-bar {
		background: var(--highlight-green-border);
	}
	.hl-color-pink .hl-bar {
		background: var(--highlight-pink-border);
	}
	.hl-color-purple .hl-bar {
		background: var(--highlight-purple-border);
	}

	.hl-content {
		padding: 10px 12px 10px 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
	}

	.hl-text {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		color: var(--text-primary);
		line-height: 1.5;
		margin: 0;
		overflow: hidden;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
	}

	.hl-source {
		font-family: var(--font-sans);
		font-size: 11px;
		color: var(--text-tertiary);
		text-decoration: none;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.hl-source:hover {
		color: var(--text-secondary);
		text-decoration: underline;
	}

	.hl-item.skeleton {
		border: 0.5px solid var(--border-primary);
	}

	.hl-item.skeleton .hl-bar,
	.skeleton-line {
		background: var(--border-primary);
		animation: shimmer 1.4s ease infinite;
	}

	.skeleton-line {
		height: 12px;
		border-radius: 6px;
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
</style>
