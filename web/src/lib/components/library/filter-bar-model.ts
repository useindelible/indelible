import type { CollectionResponse } from '$lib/api/generated/types.gen';
import { t } from '$lib/i18n';
import type { FilterCondition } from '$lib/utils/filter-expression';
import {
	getLibraryFilterFieldDef,
	type LibraryFilterFieldDef
} from '$lib/utils/library-filter-fields';
import { get } from 'svelte/store';

export type FilterCollection = Pick<CollectionResponse, 'id' | 'name'>;

function createFilterId(): string {
	return globalThis.crypto?.randomUUID() ?? `filter_${Date.now().toString(36)}`;
}

function getDefaultFilterValue(def: LibraryFilterFieldDef): FilterCondition['value'] {
	if (def.valueType === 'boolean') return true;
	if (def.valueType === 'select' && def.options?.length) return def.options[0]!.value;
	if (def.valueType === 'number') return 0;
	return '';
}

export function coerceFilterFieldChange(
	def: LibraryFilterFieldDef
): Pick<FilterCondition, 'field' | 'op' | 'value'> {
	return {
		field: def.key,
		op: def.valueType === 'boolean' ? 'eq' : (def.ops[0]?.value ?? 'eq'),
		value: getDefaultFilterValue(def)
	};
}

export function createDefaultFilterCondition(
	visibleFields: LibraryFilterFieldDef[]
): FilterCondition {
	const def = visibleFields[0] ?? getLibraryFilterFieldDef('tag');
	return {
		id: createFilterId(),
		...coerceFilterFieldChange(def)
	};
}

export function coerceFilterOperatorChange(
	condition: FilterCondition,
	def: LibraryFilterFieldDef,
	op: string
): Pick<FilterCondition, 'op' | 'value'> {
	let value = condition.value;

	if (op === 'in' && !Array.isArray(value)) {
		value = typeof value === 'string' && value ? [value] : [];
	} else if (op !== 'in' && Array.isArray(value)) {
		value = value[0] ?? def.options?.[0]?.value ?? '';
	}

	return { op, value };
}

export function getFilterValueLabel(
	condition: FilterCondition,
	def: LibraryFilterFieldDef,
	collections: FilterCollection[]
): string {
	if (def.valueType === 'boolean') {
		return condition.value === true
			? get(t)(def.booleanLabelKeys?.true ?? 'library_filter_boolean_true')
			: get(t)(def.booleanLabelKeys?.false ?? 'library_filter_boolean_false');
	}

	if (Array.isArray(condition.value)) {
		if (condition.value.length === 0) return get(t)('library_filter_select');
		return condition.value
			.map((value) => {
				const option = def.options?.find((candidate) => candidate.value === value);
				return option ? get(t)(option.labelKey) : value;
			})
			.join(', ');
	}

	if (condition.field === 'collection' && typeof condition.value === 'string' && condition.value) {
		const collection = collections.find((candidate) => candidate.id === condition.value);
		if (collection) return collection.name;
	}

	if (def.options) {
		const option = def.options.find((candidate) => candidate.value === String(condition.value));
		if (option) return get(t)(option.labelKey);
	}

	if (condition.value === '' || condition.value === 0) return get(t)('library_filter_select');
	return String(condition.value);
}

export function isFilterValuePlaceholder(
	condition: FilterCondition,
	def: LibraryFilterFieldDef
): boolean {
	if (def.valueType === 'boolean') return false;
	if (Array.isArray(condition.value)) return condition.value.length === 0;
	return condition.value === '' || (def.valueType === 'number' && condition.value === 0);
}

export function filterCollections(
	collections: FilterCollection[],
	searchValue: FilterCondition['value']
): FilterCollection[] {
	const search = typeof searchValue === 'string' ? searchValue : '';
	const normalizedSearch = search.toLowerCase();
	return collections.filter(
		(collection) =>
			!normalizedSearch ||
			collection.name.toLowerCase().includes(normalizedSearch) ||
			collection.id === search
	);
}
