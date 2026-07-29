<script lang="ts">
	import * as api from '$lib/api';
	import type { TagResponse } from '$lib/api/generated/types.gen';
	import { getSidebar } from '$lib/stores/sidebar.svelte';
	import type { FilterCondition } from '$lib/utils/filter-expression';
	import {
		getLibraryFilterFieldDef,
		getVisibleLibraryFilterFields
	} from '$lib/utils/library-filter-fields';
	import FilterActions from './FilterActions.svelte';
	import FilterConditionRow from './FilterConditionRow.svelte';
	import {
		coerceFilterFieldChange,
		coerceFilterOperatorChange,
		createDefaultFilterCondition,
		getFilterValueLabel,
		isFilterValuePlaceholder
	} from './filter-bar-model';

	interface Props {
		conditions: FilterCondition[];
		conjunction: 'and' | 'or';
		activeType?: string;
		onConditionsChange: (conditions: FilterCondition[]) => void;
		onConjunctionChange: (conjunction: 'and' | 'or') => void;
		onSaveClick: () => void;
	}

	let {
		conditions,
		conjunction,
		activeType,
		onConditionsChange,
		onConjunctionChange,
		onSaveClick
	}: Props = $props();

	let openFieldPicker = $state<string | null>(null);
	let openOpPicker = $state<string | null>(null);
	let openValuePicker = $state<string | null>(null);
	let tagSuggestions = $state<TagResponse[]>([]);

	const sidebar = getSidebar();

	const visibleFields = $derived(getVisibleLibraryFilterFields(activeType));
	const contentFields = $derived(visibleFields.filter((field) => field.section === 'content'));
	const attributeFields = $derived(visibleFields.filter((field) => field.section === 'attributes'));
	const dateFields = $derived(visibleFields.filter((field) => field.section === 'dates'));

	function addCondition() {
		onConditionsChange([...conditions, createDefaultFilterCondition(visibleFields)]);
	}

	function removeCondition(id: string) {
		onConditionsChange(conditions.filter((condition) => condition.id !== id));
	}

	function updateCondition(id: string, patch: Partial<FilterCondition>) {
		onConditionsChange(
			conditions.map((condition) => (condition.id === id ? { ...condition, ...patch } : condition))
		);
	}

	function setField(conditionId: string, fieldKey: string) {
		updateCondition(conditionId, coerceFilterFieldChange(getLibraryFilterFieldDef(fieldKey)));
		openFieldPicker = null;
	}

	function setOp(conditionId: string, op: string) {
		const condition = conditions.find((candidate) => candidate.id === conditionId);
		if (!condition) return;

		updateCondition(
			conditionId,
			coerceFilterOperatorChange(condition, getLibraryFilterFieldDef(condition.field), op)
		);
		openOpPicker = null;
	}

	function setValue(conditionId: string, value: FilterCondition['value']) {
		updateCondition(conditionId, { value });
	}

	function toggleBoolean(conditionId: string) {
		const condition = conditions.find((candidate) => candidate.id === conditionId);
		if (!condition) return;
		updateCondition(conditionId, { value: !condition.value });
	}

	function toggleMultiValue(conditionId: string, value: string) {
		const condition = conditions.find((candidate) => candidate.id === conditionId);
		if (!condition || !Array.isArray(condition.value)) return;
		const next = condition.value.includes(value)
			? condition.value.filter((candidate) => candidate !== value)
			: [...condition.value, value];
		updateCondition(conditionId, { value: next });
	}

	async function searchTags(query: string) {
		try {
			const resp = await api.listTags({ query: { limit: 50 } });
			if (resp.data) {
				const tags = resp.data.data as TagResponse[];
				tagSuggestions = query
					? tags.filter((tag) => tag.name.toLowerCase().includes(query.toLowerCase()))
					: tags;
			}
		} catch {
			tagSuggestions = [];
		}
	}

	function closeValuePicker() {
		openValuePicker = null;
	}

	function handleClickOutsideDropdowns(event: MouseEvent) {
		const target = event.target as HTMLElement;
		if (!target.closest('.field-picker') && !target.closest('.filter-field-btn')) {
			openFieldPicker = null;
		}
		if (!target.closest('.op-picker') && !target.closest('.filter-op-btn')) {
			openOpPicker = null;
		}
		if (!target.closest('.value-picker') && !target.closest('.filter-value-btn')) {
			openValuePicker = null;
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			openFieldPicker = null;
			openOpPicker = null;
			openValuePicker = null;
		}
	}

	$effect(() => {
		if (openFieldPicker || openOpPicker || openValuePicker) {
			document.addEventListener('mousedown', handleClickOutsideDropdowns);
			document.addEventListener('keydown', handleKeydown);
			return () => {
				document.removeEventListener('mousedown', handleClickOutsideDropdowns);
				document.removeEventListener('keydown', handleKeydown);
			};
		}
	});

	$effect(() => {
		if (openValuePicker) {
			const condition = conditions.find((candidate) => candidate.id === openValuePicker);
			if (condition && condition.field === 'tag') {
				searchTags(typeof condition.value === 'string' ? condition.value : '');
			}
		}
	});
</script>

<div class="filter-bar">
	{#each conditions as condition, index (condition.id)}
		{@const fieldDef = getLibraryFilterFieldDef(condition.field)}
		<FilterConditionRow
			{condition}
			{index}
			{conjunction}
			{fieldDef}
			{contentFields}
			{attributeFields}
			{dateFields}
			collections={sidebar.allCollections}
			{tagSuggestions}
			fieldPickerOpen={openFieldPicker === condition.id}
			opPickerOpen={openOpPicker === condition.id}
			valuePickerOpen={openValuePicker === condition.id}
			valueLabel={getFilterValueLabel(condition, fieldDef, sidebar.allCollections)}
			valuePlaceholder={isFilterValuePlaceholder(condition, fieldDef)}
			onToggleConjunction={() => onConjunctionChange(conjunction === 'and' ? 'or' : 'and')}
			onToggleFieldPicker={(id) => (openFieldPicker = openFieldPicker === id ? null : id)}
			onToggleOpPicker={(id) => (openOpPicker = openOpPicker === id ? null : id)}
			onToggleValuePicker={(id) => (openValuePicker = openValuePicker === id ? null : id)}
			onFieldChange={setField}
			onOperatorChange={setOp}
			onValueChange={setValue}
			onToggleBoolean={toggleBoolean}
			onToggleMultiValue={toggleMultiValue}
			onSearchTags={searchTags}
			onCloseValuePicker={closeValuePicker}
			onRemove={removeCondition}
		/>
	{/each}

	<FilterActions onAddCondition={addCondition} {onSaveClick} />
</div>

<style>
	.filter-bar {
		display: flex;
		flex-direction: column;
		gap: 0;
		border-bottom: 0.5px solid var(--border-primary);
		flex-shrink: 0;
	}

	.filter-bar :global(.filter-row + .filter-row) {
		border-top: 0.5px solid var(--border-primary);
	}
</style>
