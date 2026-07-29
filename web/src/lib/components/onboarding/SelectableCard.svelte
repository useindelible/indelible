<script lang="ts">
	let {
		label,
		description = '',
		selected = false,
		onclick
	}: {
		label: string;
		description?: string;
		selected?: boolean;
		onclick?: () => void;
	} = $props();
</script>

<button class="selectable-card" class:selected {onclick} type="button">
	<span class="card-label">{label}</span>
	{#if description}
		<span class="card-description">{description}</span>
	{/if}
</button>

<style>
	.selectable-card {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 16px;
		background: var(--bg-secondary);
		border: 2px solid var(--border-primary);
		border-radius: var(--radius-lg);
		cursor: pointer;
		transition:
			border-color 0.15s ease,
			background 0.15s ease;
		text-align: left;
		width: 100%;
	}

	.selectable-card:hover {
		background: var(--fill-hover);
		border-color: var(--border-secondary);
	}

	.selectable-card.selected {
		border-color: var(--accent);
		background: var(--fill-selected);
	}

	.card-label {
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.card-description {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-secondary);
		line-height: 1.4;
	}
</style>
