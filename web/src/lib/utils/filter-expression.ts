import type { FilterExpressionNode } from '$lib/api/generated/types.gen';

export type FilterCondition = {
	id: string;
	field: string;
	op: string;
	value: string | number | boolean | string[];
};

type FilterValue = FilterCondition['value'];
const BOOLEAN_FIELDS = new Set(['is_favorite', 'has_unsubscribe', 'sender_blocked']);

export type FilterExpression =
	| {
			type: 'and' | 'or';
			conditions: FilterExpression[];
	  }
	| {
			type: 'condition';
			field: string;
			op: string;
			value: string | number | boolean | string[];
	  };

export function buildFilterExpression(
	conditions: FilterCondition[],
	conjunction: 'and' | 'or'
): FilterExpression {
	return {
		type: conjunction,
		conditions: conditions.map((c) => {
			let value: FilterValue = c.value;

			if (BOOLEAN_FIELDS.has(c.field)) {
				value = c.value === true || c.value === 'true';
			} else if (c.op === 'in' && !Array.isArray(c.value)) {
				value = c.value ? [String(c.value)] : [];
			} else if (c.field === 'saved_at' || c.field === 'published_at') {
				value =
					typeof c.value === 'string' && c.value
						? new Date(c.value).toISOString()
						: String(c.value);
			}

			return {
				type: 'condition',
				field: c.field,
				op: BOOLEAN_FIELDS.has(c.field) ? 'eq' : c.op,
				value
			} satisfies FilterExpression;
		})
	};
}

export function toApiFilterExpression(
	expression: FilterExpression | null | undefined
): FilterExpressionNode | null | undefined {
	return expression as unknown as FilterExpressionNode | null | undefined;
}

export function fromApiFilterExpression(
	expression: FilterExpressionNode | null | undefined
): FilterExpression | null {
	return (expression ?? null) as unknown as FilterExpression | null;
}

export function filterExpressionHasField(
	expression: FilterExpression | null | undefined,
	field: string
): boolean {
	if (!expression) return false;
	if (expression.type === 'condition') return expression.field === field;
	return expression.conditions.some((condition) => filterExpressionHasField(condition, field));
}

export function parseFilterExpression(expr: unknown): {
	conditions: FilterCondition[];
	conjunction: 'and' | 'or';
} {
	if (!expr || typeof expr !== 'object') return { conditions: [], conjunction: 'and' };
	const obj = expr as Record<string, unknown>;
	const conj = (obj.type === 'or' ? 'or' : 'and') as 'and' | 'or';
	const rawConditions = Array.isArray(obj.conditions) ? obj.conditions : [];
	const parsed: FilterCondition[] = [];
	for (const raw of rawConditions) {
		if (raw && typeof raw === 'object' && (raw as Record<string, unknown>).type === 'condition') {
			const r = raw as Record<string, unknown>;
			parsed.push({
				id: crypto.randomUUID(),
				field: String(r.field ?? ''),
				op: String(r.op ?? 'eq'),
				value: (r.value as FilterCondition['value']) ?? ''
			});
		}
	}
	return { conditions: parsed, conjunction: conj };
}
