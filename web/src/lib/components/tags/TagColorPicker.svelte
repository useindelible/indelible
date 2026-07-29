<script lang="ts">
	interface Props {
		value: string | null;
		onChange: (color: string | null) => void;
	}

	let { value, onChange }: Props = $props();

	const PALETTE = [
		{ label: 'Yellow', value: '#FFD600' },
		{ label: 'Blue', value: '#0A84FF' },
		{ label: 'Green', value: '#34C759' },
		{ label: 'Pink', value: '#FF2D55' },
		{ label: 'Purple', value: '#AF52DE' }
	];
</script>

<div class="color-picker" role="radiogroup" aria-label="Tag color">
	<button
		type="button"
		class="swatch"
		class:selected={!value}
		role="radio"
		aria-checked={!value}
		aria-label="No color"
		onclick={() => onChange(null)}
	>
		<span class="swatch-inner swatch-none">-</span>
	</button>
	{#each PALETTE as color (color.value)}
		<button
			type="button"
			class="swatch"
			class:selected={value === color.value}
			role="radio"
			aria-checked={value === color.value}
			aria-label={color.label}
			onclick={() => onChange(color.value)}
		>
			<span class="swatch-inner" style="background: {color.value}"></span>
		</button>
	{/each}
</div>

<style>
	.color-picker {
		display: flex;
		gap: 8px;
	}

	.swatch {
		width: 32px;
		height: 32px;
		border-radius: 50%;
		border: 2px solid transparent;
		background: transparent;
		cursor: pointer;
		padding: 3px;
		display: flex;
		align-items: center;
		justify-content: center;
		transition:
			border-color 0.12s ease,
			transform 0.12s ease;
	}

	.swatch:hover {
		transform: scale(1.1);
	}

	.swatch.selected {
		border-color: var(--accent);
	}

	.swatch-inner {
		width: 100%;
		height: 100%;
		border-radius: 50%;
	}

	.swatch-none {
		background: var(--fill-secondary);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 14px;
		color: var(--text-tertiary);
	}
</style>
