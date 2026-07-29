<script lang="ts">
	import type { TagResponse } from '$lib/api/generated/types.gen';
	import type { FilterCondition } from '$lib/utils/filter-expression';
	import type { LibraryFilterFieldDef } from '$lib/utils/library-filter-fields';
	import FilterFieldPicker from './FilterFieldPicker.svelte';
	import FilterOperatorPicker from './FilterOperatorPicker.svelte';
	import FilterValuePicker from './FilterValuePicker.svelte';
	import type { FilterCollection } from './filter-bar-model';

	interface Props {
		condition: FilterCondition;
		index: number;
		conjunction: 'and' | 'or';
		fieldDef: LibraryFilterFieldDef;
		contentFields: LibraryFilterFieldDef[];
		attributeFields: LibraryFilterFieldDef[];
		dateFields: LibraryFilterFieldDef[];
		collections: FilterCollection[];
		tagSuggestions: TagResponse[];
		fieldPickerOpen: boolean;
		opPickerOpen: boolean;
		valuePickerOpen: boolean;
		valueLabel: string;
		valuePlaceholder: boolean;
		onToggleConjunction: () => void;
		onToggleFieldPicker: (id: string) => void;
		onToggleOpPicker: (id: string) => void;
		onToggleValuePicker: (id: string) => void;
		onFieldChange: (id: string, field: string) => void;
		onOperatorChange: (id: string, op: string) => void;
		onValueChange: (id: string, value: FilterCondition['value']) => void;
		onToggleBoolean: (id: string) => void;
		onToggleMultiValue: (id: string, value: string) => void;
		onSearchTags: (query: string) => void;
		onCloseValuePicker: () => void;
		onRemove: (id: string) => void;
	}

	let {
		condition,
		index,
		conjunction,
		fieldDef,
		contentFields,
		attributeFields,
		dateFields,
		collections,
		tagSuggestions,
		fieldPickerOpen,
		opPickerOpen,
		valuePickerOpen,
		valueLabel,
		valuePlaceholder,
		onToggleConjunction,
		onToggleFieldPicker,
		onToggleOpPicker,
		onToggleValuePicker,
		onFieldChange,
		onOperatorChange,
		onValueChange,
		onToggleBoolean,
		onToggleMultiValue,
		onSearchTags,
		onCloseValuePicker,
		onRemove
	}: Props = $props();
</script>

<div class="filter-row">
	{#if index === 0}
		<span class="filter-conjunction">Where</span>
	{:else}
		<button type="button" class="filter-conjunction-toggle" onclick={onToggleConjunction}>
			{conjunction === 'and' ? 'And' : 'Or'}
		</button>
	{/if}

	<div class="field-picker-anchor">
		<button
			type="button"
			class="filter-field-btn"
			onclick={() => onToggleFieldPicker(condition.id)}
		>
			{fieldDef.label}
			<svg viewBox="0 0 24 24" aria-hidden="true"><polyline points="6 9 12 15 18 9" /></svg>
		</button>

		{#if fieldPickerOpen}
			<FilterFieldPicker
				currentField={condition.field}
				{contentFields}
				{attributeFields}
				{dateFields}
				onFieldChange={(field) => onFieldChange(condition.id, field)}
			/>
		{/if}
	</div>

	<FilterOperatorPicker
		{condition}
		{fieldDef}
		pickerOpen={opPickerOpen}
		onTogglePicker={() => onToggleOpPicker(condition.id)}
		onOperatorChange={(op) => onOperatorChange(condition.id, op)}
	/>

	<FilterValuePicker
		{condition}
		{fieldDef}
		{collections}
		{tagSuggestions}
		pickerOpen={valuePickerOpen}
		{valueLabel}
		{valuePlaceholder}
		onTogglePicker={() => onToggleValuePicker(condition.id)}
		onToggleBoolean={() => onToggleBoolean(condition.id)}
		onValueChange={(value) => onValueChange(condition.id, value)}
		onToggleMultiValue={(value) => onToggleMultiValue(condition.id, value)}
		{onSearchTags}
		onClose={onCloseValuePicker}
	/>

	<button
		type="button"
		class="filter-remove-btn"
		onclick={() => onRemove(condition.id)}
		aria-label="Remove condition"
	>
		<svg viewBox="0 0 24 24" aria-hidden="true">
			<line x1="18" y1="6" x2="6" y2="18" />
			<line x1="6" y1="6" x2="18" y2="18" />
		</svg>
	</button>
</div>

<style>
	.filter-row {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 20px;
		min-height: 40px;
	}

	@media (max-width: 599px) {
		.filter-row {
			flex-wrap: wrap;
			padding: 8px 16px;
			row-gap: 6px;
		}
	}

	.filter-conjunction {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		color: var(--text-quaternary);
		width: 38px;
		flex-shrink: 0;
	}

	.filter-conjunction-toggle {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		color: var(--accent);
		width: 38px;
		flex-shrink: 0;
		cursor: pointer;
		background: none;
		border: none;
		padding: 0;
		font-family: var(--font-sans);
		text-align: left;
	}

	.filter-conjunction-toggle:hover {
		text-decoration: underline;
	}

	.field-picker-anchor {
		position: relative;
	}

	.filter-field-btn {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 4px 10px;
		border-radius: 6px;
		font-size: 12.5px;
		font-weight: 500;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		background: var(--fill-secondary);
		cursor: pointer;
		white-space: nowrap;
		border: none;
		font-family: var(--font-sans);
	}

	.filter-field-btn:hover {
		background: var(--fill-hover);
	}

	.filter-field-btn svg {
		width: 10px;
		height: 10px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.filter-remove-btn {
		width: 20px;
		height: 20px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 4px;
		color: var(--text-quaternary);
		cursor: pointer;
		flex-shrink: 0;
		margin-left: auto;
		background: none;
		border: none;
		padding: 0;
	}

	.filter-remove-btn:hover {
		background: var(--fill-hover);
		color: var(--text-tertiary);
	}

	.filter-remove-btn svg {
		width: 12px;
		height: 12px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
</style>
