<script lang="ts">
	import { t, type MessageKey } from '$lib/i18n';

	interface Props {
		value: string | null;
		onChange: (color: string | null) => void;
	}

	let { value, onChange }: Props = $props();

	const PALETTE = [
		{ labelKey: 'tag_color_yellow', value: '#FFD600' },
		{ labelKey: 'tag_color_blue', value: '#0A84FF' },
		{ labelKey: 'tag_color_green', value: '#34C759' },
		{ labelKey: 'tag_color_pink', value: '#FF2D55' },
		{ labelKey: 'tag_color_purple', value: '#AF52DE' }
	];
</script>

<div class="color-picker" role="radiogroup" aria-label={$t('tag_color')}>
	<button
		type="button"
		class="swatch"
		class:selected={!value}
		role="radio"
		aria-checked={!value}
		aria-label={$t('tag_color_none')}
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
			aria-label={$t(color.labelKey as MessageKey)}
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
