<script lang="ts">
	interface Props {
		count?: number;
	}

	let { count = 6 }: Props = $props();
</script>

{#each Array.from({ length: count }, (_, i) => i) as i (i)}
	<div class="skeleton-row" aria-hidden="true">
		<div class="skeleton-thumb shimmer"></div>
		<div class="skeleton-content">
			<div class="skeleton-title shimmer"></div>
			<div class="skeleton-excerpt shimmer"></div>
			<div class="skeleton-source shimmer"></div>
		</div>
		<div class="skeleton-timestamp shimmer"></div>
	</div>
{/each}

<style>
	@keyframes shimmer {
		0% {
			opacity: 1;
		}
		50% {
			opacity: 0.45;
		}
		100% {
			opacity: 1;
		}
	}

	.shimmer {
		animation: shimmer 1.6s ease-in-out infinite;
		background: var(--bg-tertiary);
		border-radius: var(--radius-sm);
	}

	.skeleton-row {
		display: flex;
		align-items: flex-start;
		gap: 16px;
		padding: 14px 20px;
		border-bottom: 0.5px solid var(--border-primary);
	}

	.skeleton-thumb {
		width: 56px;
		height: 56px;
		border-radius: var(--radius-lg);
		flex-shrink: 0;
	}

	.skeleton-content {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding-top: 2px;
	}

	.skeleton-title {
		height: 14px;
		width: 75%;
	}

	.skeleton-excerpt {
		height: 13px;
		width: 90%;
	}

	.skeleton-source {
		height: 11px;
		width: 40%;
		margin-top: 2px;
	}

	.skeleton-timestamp {
		width: 40px;
		height: 11px;
		margin-top: 4px;
		flex-shrink: 0;
	}
</style>
