<script lang="ts">
	import type { LibraryFilterFieldDef } from '$lib/utils/library-filter-fields';

	interface Props {
		currentField: string;
		contentFields: LibraryFilterFieldDef[];
		attributeFields: LibraryFilterFieldDef[];
		dateFields: LibraryFilterFieldDef[];
		onFieldChange: (field: string) => void;
	}

	let { currentField, contentFields, attributeFields, dateFields, onFieldChange }: Props = $props();
</script>

<div class="field-picker">
	<div class="field-picker-section">Content</div>
	{#each contentFields as field (field.key)}
		<button
			type="button"
			class="field-picker-item"
			class:highlighted={currentField === field.key}
			onclick={() => onFieldChange(field.key)}
		>
			{field.label}
		</button>
	{/each}
	<div class="field-picker-divider"></div>
	<div class="field-picker-section">Attributes</div>
	{#each attributeFields as field (field.key)}
		<button
			type="button"
			class="field-picker-item"
			class:highlighted={currentField === field.key}
			onclick={() => onFieldChange(field.key)}
		>
			{field.label}
		</button>
	{/each}
	<div class="field-picker-divider"></div>
	<div class="field-picker-section">Dates</div>
	{#each dateFields as field (field.key)}
		<button
			type="button"
			class="field-picker-item"
			class:highlighted={currentField === field.key}
			onclick={() => onFieldChange(field.key)}
		>
			{field.label}
		</button>
	{/each}
</div>

<style>
	.field-picker {
		position: absolute;
		top: calc(100% + 4px);
		left: 0;
		width: 220px;
		background: var(--bg-elevated);
		border: 0.5px solid var(--border-secondary);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-3);
		z-index: 100;
		overflow: hidden;
		padding: 4px 0;
	}

	.field-picker-section {
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-quaternary);
		padding: 8px 12px 4px;
	}

	.field-picker-divider {
		height: 0.5px;
		background: var(--border-primary);
		margin: 4px 0;
	}

	.field-picker-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 7px 12px;
		font-size: 13px;
		font-weight: 400;
		color: var(--text-primary);
		cursor: pointer;
		letter-spacing: -0.01em;
		width: 100%;
		background: none;
		border: none;
		font-family: var(--font-sans);
		text-align: left;
	}

	.field-picker-item:hover {
		background: var(--fill-hover);
	}

	.field-picker-item.highlighted {
		background: var(--fill-selected);
		color: var(--accent);
	}
</style>
