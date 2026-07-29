<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		message: string;
		icon?: Snippet;
		action?: Snippet;
	}

	let { message, icon, action }: Props = $props();
</script>

<div class="empty-state">
	{#if icon}
		<div class="empty-icon">
			{@render icon()}
		</div>
	{/if}
	<p class="empty-message">{message}</p>
	{#if action}
		<div class="empty-action">
			{@render action()}
		</div>
	{/if}
</div>

<style>
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		text-align: center;
		padding: 48px 24px;
		gap: 8px;
	}

	.empty-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-secondary);
		margin-bottom: 4px;
	}

	.empty-message {
		font-family: var(--font-sans);
		font-size: 14px;
		font-weight: 500;
		letter-spacing: -0.01em;
		color: var(--text-secondary);
		margin: 0;
	}

	.empty-action {
		margin-top: 8px;
	}
</style>
