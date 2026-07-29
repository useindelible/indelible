<script lang="ts">
	import type { Snippet } from 'svelte';
	import Button from '$lib/components/ui/Button.svelte';

	interface Props {
		loading: boolean;
		loadError: string | null;
		onRetry: () => void;
		children?: Snippet;
	}

	let { loading, loadError, onRetry, children }: Props = $props();
</script>

{#if loading}
	<div class="body-area">
		<p class="meta">Loading Notion connection…</p>
	</div>
{:else if loadError}
	<div class="body-area">
		<div class="error-block" role="alert">
			<p>{loadError}</p>
			<Button variant="secondary" size="sm" onclick={onRetry}>Retry</Button>
		</div>
	</div>
{:else}
	<div class="body-area">
		{@render children?.()}
	</div>
{/if}

<style>
	.body-area {
		padding: 36px 56px 16px;
		display: flex;
		flex-direction: column;
		max-width: 920px;
		width: 100%;
		align-self: center;
		margin: 0 auto;
	}

	.meta {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-tertiary);
		margin: 0;
	}

	.error-block {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 14px 16px;
		border-radius: 10px;
		background: var(--fill-danger);
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-primary);
	}

	@media (max-width: 720px) {
		.body-area {
			padding: 24px 20px 16px;
		}
	}
</style>
