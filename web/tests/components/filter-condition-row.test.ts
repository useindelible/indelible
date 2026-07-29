import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';

import FilterConditionRow from '$lib/components/library/FilterConditionRow.svelte';
import type { FilterCondition } from '$lib/utils/filter-expression';
import { getLibraryFilterFieldDef } from '$lib/utils/library-filter-fields';

describe('FilterConditionRow', () => {
	it('renders the condition controls and removes through a callback', async () => {
		const condition: FilterCondition = { id: 'cond_1', field: 'tag', op: 'contains', value: '' };
		const onRemove = vi.fn();

		render(FilterConditionRow, {
			props: {
				condition,
				index: 0,
				conjunction: 'and',
				fieldDef: getLibraryFilterFieldDef('tag'),
				contentFields: [getLibraryFilterFieldDef('tag')],
				attributeFields: [getLibraryFilterFieldDef('is_favorite')],
				dateFields: [getLibraryFilterFieldDef('saved_at')],
				collections: [],
				tagSuggestions: [],
				fieldPickerOpen: false,
				opPickerOpen: false,
				valuePickerOpen: false,
				valueLabel: 'Select...',
				valuePlaceholder: true,
				onToggleConjunction: vi.fn(),
				onToggleFieldPicker: vi.fn(),
				onToggleOpPicker: vi.fn(),
				onToggleValuePicker: vi.fn(),
				onFieldChange: vi.fn(),
				onOperatorChange: vi.fn(),
				onValueChange: vi.fn(),
				onToggleBoolean: vi.fn(),
				onToggleMultiValue: vi.fn(),
				onSearchTags: vi.fn(),
				onCloseValuePicker: vi.fn(),
				onRemove
			}
		});

		expect(screen.getByText('Where')).toBeTruthy();
		expect(screen.getByRole('button', { name: /Tag/ })).toBeTruthy();
		expect(screen.getByRole('button', { name: 'contains' })).toBeTruthy();
		expect(screen.getByRole('button', { name: /Select/ })).toBeTruthy();

		await fireEvent.click(screen.getByRole('button', { name: 'Remove condition' }));

		expect(onRemove).toHaveBeenCalledWith('cond_1');
	});
});
