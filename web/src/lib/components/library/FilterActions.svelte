<script lang="ts">
	interface Props {
		onAddCondition: () => void;
		onSaveClick: () => void;
	}

	let { onAddCondition, onSaveClick }: Props = $props();
</script>

<div class="filter-actions-row">
	<button type="button" class="filter-add-btn" onclick={onAddCondition}>
		<svg viewBox="0 0 24 24" aria-hidden="true">
			<line x1="12" y1="5" x2="12" y2="19" />
			<line x1="5" y1="12" x2="19" y2="12" />
		</svg>
		Add condition
	</button>
	<button type="button" class="filter-save-btn" onclick={onSaveClick}> Save as view </button>
</div>

<style>
	.filter-actions-row {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 6px 20px 10px;
	}

	.filter-add-btn {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 4px 10px;
		border-radius: 6px;
		font-size: 12px;
		font-weight: 500;
		letter-spacing: -0.01em;
		color: var(--text-tertiary);
		cursor: pointer;
		background: none;
		border: none;
		font-family: var(--font-sans);
	}

	.filter-add-btn:hover {
		color: var(--text-secondary);
		background: var(--fill-hover);
	}

	.filter-add-btn svg {
		width: 12px;
		height: 12px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.filter-save-btn {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 4px 10px;
		border-radius: 6px;
		font-size: 12px;
		font-weight: 500;
		letter-spacing: -0.01em;
		color: var(--accent);
		cursor: pointer;
		margin-left: auto;
		background: none;
		border: none;
		font-family: var(--font-sans);
	}

	.filter-save-btn:hover {
		background: var(--fill-hover);
	}
</style>
