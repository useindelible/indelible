import { describe, it, expect } from 'vitest';
import {
	buildFilterExpression,
	isFlatFilterExpression,
	parseFilterExpression,
	type FilterCondition
} from '../../src/lib/utils/filter-expression';
import {
	coerceFilterFieldChange,
	coerceFilterOperatorChange,
	createDefaultFilterCondition,
	getFilterValueLabel,
	isFilterValuePlaceholder
} from '../../src/lib/components/library/filter-bar-model';
import {
	getLibraryFilterFieldDef,
	type LibraryFilterFieldDef
} from '../../src/lib/utils/library-filter-fields';

describe('buildFilterExpression', () => {
	it('produces correct structure with AND conjunction', () => {
		const conditions: FilterCondition[] = [
			{ id: '1', field: 'item_type', op: 'eq', value: 'article' }
		];
		const result = buildFilterExpression(conditions, 'and');
		expect(result).toEqual({
			type: 'and',
			conditions: [{ type: 'condition', field: 'item_type', op: 'eq', value: 'article' }]
		});
	});

	it('produces correct structure with OR conjunction', () => {
		const conditions: FilterCondition[] = [
			{ id: '1', field: 'item_type', op: 'eq', value: 'article' },
			{ id: '2', field: 'domain', op: 'eq', value: 'example.com' }
		];
		const result = buildFilterExpression(conditions, 'or');
		expect(result.type).toBe('or');
		expect(result.conditions).toHaveLength(2);
	});

	it('passes through sender condition unchanged', () => {
		const conditions: FilterCondition[] = [
			{ id: '1', field: 'sender', op: 'contains', value: 'newsletter' }
		];
		const result = buildFilterExpression(conditions, 'and');
		const cond = (result.conditions as Array<Record<string, unknown>>)[0]!;
		expect(cond.field).toBe('sender');
		expect(cond.op).toBe('contains');
		expect(cond.value).toBe('newsletter');
	});

	it('coerces has_unsubscribe to boolean with forced eq op', () => {
		const conditions: FilterCondition[] = [
			{ id: '1', field: 'has_unsubscribe', op: 'eq', value: true }
		];
		const result = buildFilterExpression(conditions, 'and');
		const cond = (result.conditions as Array<Record<string, unknown>>)[0]!;
		expect(cond.value).toBe(true);
		expect(cond.op).toBe('eq');
	});

	it('coerces sender_blocked string "true" to boolean', () => {
		const conditions: FilterCondition[] = [
			{ id: '1', field: 'sender_blocked', op: 'eq', value: 'true' }
		];
		const result = buildFilterExpression(conditions, 'and');
		const cond = (result.conditions as Array<Record<string, unknown>>)[0]!;
		expect(cond.value).toBe(true);
	});

	it('coerces is_favorite to boolean true', () => {
		const conditions: FilterCondition[] = [
			{ id: '1', field: 'is_favorite', op: 'eq', value: true }
		];
		const result = buildFilterExpression(conditions, 'and');
		const cond = (result.conditions as Array<Record<string, unknown>>)[0]!;
		expect(cond.value).toBe(true);
		expect(cond.op).toBe('eq');
	});

	it('coerces is_favorite to boolean false', () => {
		const conditions: FilterCondition[] = [
			{ id: '1', field: 'is_favorite', op: 'eq', value: false }
		];
		const result = buildFilterExpression(conditions, 'and');
		const cond = (result.conditions as Array<Record<string, unknown>>)[0]!;
		expect(cond.value).toBe(false);
	});

	it('wraps single value in array for "in" op', () => {
		const conditions: FilterCondition[] = [
			{ id: '1', field: 'item_type', op: 'in', value: 'article' }
		];
		const result = buildFilterExpression(conditions, 'and');
		const cond = (result.conditions as Array<Record<string, unknown>>)[0]!;
		expect(cond.value).toEqual(['article']);
	});

	it('preserves array for "in" op', () => {
		const conditions: FilterCondition[] = [
			{ id: '1', field: 'item_type', op: 'in', value: ['article', 'book'] }
		];
		const result = buildFilterExpression(conditions, 'and');
		const cond = (result.conditions as Array<Record<string, unknown>>)[0]!;
		expect(cond.value).toEqual(['article', 'book']);
	});

	it('produces empty array for "in" op with empty value', () => {
		const conditions: FilterCondition[] = [{ id: '1', field: 'item_type', op: 'in', value: '' }];
		const result = buildFilterExpression(conditions, 'and');
		const cond = (result.conditions as Array<Record<string, unknown>>)[0]!;
		expect(cond.value).toEqual([]);
	});

	it('converts date fields to ISO 8601', () => {
		const conditions: FilterCondition[] = [
			{ id: '1', field: 'saved_at', op: 'gt', value: '2025-01-15' }
		];
		const result = buildFilterExpression(conditions, 'and');
		const cond = (result.conditions as Array<Record<string, unknown>>)[0]!;
		expect(typeof cond.value).toBe('string');
		expect((cond.value as string).includes('T')).toBe(true);
	});

	it('handles published_at date conversion', () => {
		const conditions: FilterCondition[] = [
			{ id: '1', field: 'published_at', op: 'lt', value: '2025-06-01' }
		];
		const result = buildFilterExpression(conditions, 'and');
		const cond = (result.conditions as Array<Record<string, unknown>>)[0]!;
		expect((cond.value as string).includes('T')).toBe(true);
	});

	it('each condition has type "condition"', () => {
		const conditions: FilterCondition[] = [
			{ id: '1', field: 'tag', op: 'contains', value: 'rust' },
			{ id: '2', field: 'domain', op: 'eq', value: 'example.com' }
		];
		const result = buildFilterExpression(conditions, 'and');
		for (const cond of result.conditions as Array<Record<string, unknown>>) {
			expect(cond.type).toBe('condition');
		}
	});

	it('handles empty conditions list', () => {
		const result = buildFilterExpression([], 'and');
		expect(result).toEqual({ type: 'and', conditions: [] });
	});

	it('forces is_favorite op to eq regardless of input', () => {
		const conditions: FilterCondition[] = [
			{ id: '1', field: 'is_favorite', op: 'neq', value: true }
		];
		const result = buildFilterExpression(conditions, 'and');
		const cond = (result.conditions as Array<Record<string, unknown>>)[0]!;
		expect(cond.op).toBe('eq');
	});
});

describe('parseFilterExpression', () => {
	it('parses AND expression', () => {
		const expr = {
			type: 'and',
			conditions: [{ type: 'condition', field: 'item_type', op: 'eq', value: 'article' }]
		};
		const { conditions, conjunction } = parseFilterExpression(expr);
		expect(conjunction).toBe('and');
		expect(conditions).toHaveLength(1);
		expect(conditions[0]!.field).toBe('item_type');
		expect(conditions[0]!.op).toBe('eq');
		expect(conditions[0]!.value).toBe('article');
	});

	it('parses OR expression', () => {
		const expr = {
			type: 'or',
			conditions: [
				{ type: 'condition', field: 'tag', op: 'contains', value: 'rust' },
				{ type: 'condition', field: 'domain', op: 'eq', value: 'example.com' }
			]
		};
		const { conditions, conjunction } = parseFilterExpression(expr);
		expect(conjunction).toBe('or');
		expect(conditions).toHaveLength(2);
	});

	it('handles null input', () => {
		const { conditions, conjunction } = parseFilterExpression(null);
		expect(conditions).toHaveLength(0);
		expect(conjunction).toBe('and');
	});

	it('handles undefined input', () => {
		const { conditions, conjunction } = parseFilterExpression(undefined);
		expect(conditions).toHaveLength(0);
		expect(conjunction).toBe('and');
	});

	it('handles non-object input', () => {
		const { conditions } = parseFilterExpression('not an object');
		expect(conditions).toHaveLength(0);
	});

	it('skips non-condition nodes', () => {
		const expr = {
			type: 'and',
			conditions: [
				{ type: 'condition', field: 'tag', op: 'eq', value: 'rust' },
				{ type: 'group', conditions: [] },
				'invalid'
			]
		};
		const { conditions } = parseFilterExpression(expr);
		expect(conditions).toHaveLength(1);
		expect(conditions[0]!.field).toBe('tag');
	});

	it('handles missing conditions array', () => {
		const expr = { type: 'and' };
		const { conditions } = parseFilterExpression(expr);
		expect(conditions).toHaveLength(0);
	});

	it('preserves boolean values', () => {
		const expr = {
			type: 'and',
			conditions: [{ type: 'condition', field: 'is_favorite', op: 'eq', value: true }]
		};
		const { conditions } = parseFilterExpression(expr);
		expect(conditions[0]!.value).toBe(true);
	});

	it('preserves array values', () => {
		const expr = {
			type: 'and',
			conditions: [{ type: 'condition', field: 'item_type', op: 'in', value: ['article', 'pdf'] }]
		};
		const { conditions } = parseFilterExpression(expr);
		expect(conditions[0]!.value).toEqual(['article', 'pdf']);
	});

	it('preserves numeric values', () => {
		const expr = {
			type: 'and',
			conditions: [{ type: 'condition', field: 'word_count', op: 'gt', value: 5000 }]
		};
		const { conditions } = parseFilterExpression(expr);
		expect(conditions[0]!.value).toBe(5000);
	});

	it('defaults conjunction to and for unknown type', () => {
		const expr = {
			type: 'xor',
			conditions: [{ type: 'condition', field: 'tag', op: 'eq', value: 'x' }]
		};
		const { conjunction } = parseFilterExpression(expr);
		expect(conjunction).toBe('and');
	});
});

describe('roundtrip: build then parse', () => {
	it('preserves simple string conditions', () => {
		const original: FilterCondition[] = [
			{ id: '1', field: 'item_type', op: 'eq', value: 'article' },
			{ id: '2', field: 'tag', op: 'contains', value: 'rust' }
		];
		const expr = buildFilterExpression(original, 'and');
		const { conditions, conjunction } = parseFilterExpression(expr);
		expect(conjunction).toBe('and');
		expect(conditions).toHaveLength(2);
		expect(conditions[0]!.field).toBe('item_type');
		expect(conditions[0]!.value).toBe('article');
		expect(conditions[1]!.field).toBe('tag');
		expect(conditions[1]!.value).toBe('rust');
	});

	it('preserves numeric conditions', () => {
		const original: FilterCondition[] = [{ id: '1', field: 'word_count', op: 'gt', value: 5000 }];
		const expr = buildFilterExpression(original, 'and');
		const { conditions } = parseFilterExpression(expr);
		expect(conditions[0]!.value).toBe(5000);
	});

	it('preserves boolean conditions', () => {
		const original: FilterCondition[] = [{ id: '1', field: 'is_favorite', op: 'eq', value: true }];
		const expr = buildFilterExpression(original, 'and');
		const { conditions } = parseFilterExpression(expr);
		expect(conditions[0]!.value).toBe(true);
	});

	it('preserves array conditions', () => {
		const original: FilterCondition[] = [
			{ id: '1', field: 'item_type', op: 'in', value: ['article', 'pdf', 'book'] }
		];
		const expr = buildFilterExpression(original, 'or');
		const { conditions, conjunction } = parseFilterExpression(expr);
		expect(conjunction).toBe('or');
		expect(conditions[0]!.value).toEqual(['article', 'pdf', 'book']);
	});
});

describe('filter bar model helpers', () => {
	it('creates a default condition from the first visible field', () => {
		const condition = createDefaultFilterCondition([
			getLibraryFilterFieldDef('is_favorite'),
			getLibraryFilterFieldDef('tag')
		]);

		expect(condition.field).toBe('is_favorite');
		expect(condition.op).toBe('eq');
		expect(condition.value).toBe(true);
	});

	it('coerces field changes to the new field value type', () => {
		const wordCountField: LibraryFilterFieldDef = {
			key: 'word_count',
			label: 'Word count',
			section: 'attributes',
			ops: [{ value: 'gt', label: 'greater than' }],
			valueType: 'number'
		};

		expect(coerceFilterFieldChange(wordCountField)).toEqual({
			field: 'word_count',
			op: 'gt',
			value: 0
		});
	});

	it('coerces in-operator values to arrays and back', () => {
		const condition: FilterCondition = { id: '1', field: 'item_type', op: 'eq', value: 'article' };
		expect(
			coerceFilterOperatorChange(condition, getLibraryFilterFieldDef('item_type'), 'in')
		).toEqual({
			op: 'in',
			value: ['article']
		});
		expect(
			coerceFilterOperatorChange(
				{ ...condition, op: 'in', value: ['book'] },
				getLibraryFilterFieldDef('item_type'),
				'eq'
			)
		).toEqual({ op: 'eq', value: 'book' });
	});

	it('labels collection values from loaded collections', () => {
		const label = getFilterValueLabel(
			{ id: '1', field: 'collection', op: 'eq', value: 'col_1' },
			getLibraryFilterFieldDef('collection'),
			[{ id: 'col_1', name: 'Reading List' }]
		);

		expect(label).toBe('Reading List');
	});

	it('detects placeholder values for selectable filters', () => {
		expect(
			isFilterValuePlaceholder(
				{ id: '1', field: 'tag', op: 'contains', value: '' },
				getLibraryFilterFieldDef('tag')
			)
		).toBe(true);
		expect(
			isFilterValuePlaceholder(
				{ id: '1', field: 'is_favorite', op: 'eq', value: false },
				getLibraryFilterFieldDef('is_favorite')
			)
		).toBe(false);
	});
});

describe('isFlatFilterExpression', () => {
	it('accepts the shapes the filter bar can render', () => {
		expect(
			isFlatFilterExpression({ type: 'condition', field: 'tag', op: 'contains', value: 'a' })
		).toBe(true);
		expect(
			isFlatFilterExpression({
				type: 'and',
				conditions: [{ type: 'condition', field: 'tag', op: 'contains', value: 'a' }]
			})
		).toBe(true);
	});

	it('rejects shapes that would flatten into a different query', () => {
		expect(
			isFlatFilterExpression({
				type: 'and',
				conditions: [
					{ type: 'condition', field: 'tag', op: 'contains', value: 'a' },
					{ type: 'or', conditions: [] }
				]
			})
		).toBe(false);
		expect(
			isFlatFilterExpression({
				type: 'not',
				condition: { type: 'condition', field: 'tag', op: 'contains', value: 'a' }
			})
		).toBe(false);
	});
});

describe('parseFilterExpression root condition', () => {
	it('represents a bare condition as a single editable row', () => {
		const parsed = parseFilterExpression({
			type: 'condition',
			field: 'tag',
			op: 'contains',
			value: 'alpha'
		});
		expect(parsed.conjunction).toBe('and');
		expect(parsed.conditions).toHaveLength(1);
		expect(parsed.conditions[0]).toMatchObject({ field: 'tag', op: 'contains', value: 'alpha' });
	});
});
