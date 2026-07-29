import { beforeEach, describe, expect, it, vi } from 'vitest';

const mockQueryLibraryEntries = vi.fn();
const mockGetSmartList = vi.fn();

vi.mock('$lib/api', () => ({
	queryLibraryEntries: (...args: unknown[]) => mockQueryLibraryEntries(...args),
	getSmartList: (...args: unknown[]) => mockGetSmartList(...args)
}));
vi.mock('$lib/api/settings', () => ({
	loadPreferencesSettings: vi.fn().mockResolvedValue({}),
	savePreferencesSettings: vi.fn().mockResolvedValue(undefined)
}));

type Expr = {
	condition?: Expr;
	type?: string;
	field?: string;
	op?: string;
	value?: unknown;
	conditions?: Expr[];
};

const SAVED_EXPRESSION: Expr = {
	type: 'and',
	conditions: [{ type: 'condition', field: 'tag', op: 'contains', value: 'alpha' }]
};

function lastFilterExpression(): Expr | null {
	const calls = mockQueryLibraryEntries.mock.calls;
	if (calls.length === 0) return null;
	const last = calls[calls.length - 1][0] as { body: { filter_expression: Expr | null } };
	return last.body.filter_expression;
}

// The store is a module singleton, so each case needs its own module instance.
async function freshLibrary() {
	vi.resetModules();
	/* eslint-disable-next-line svelte-runes/no-external-svelte-imports -- test drives the store directly to assert the query it builds */
	const mod = await import('../../src/lib/stores/library.svelte');
	return mod.getLibrary();
}

describe('Saved smart view draft rules', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockQueryLibraryEntries.mockResolvedValue({
			data: { data: [], page: { has_more: false, next_cursor: null } }
		});
		mockGetSmartList.mockResolvedValue({
			data: { id: 'sl1', name: 'Alpha', filter_expression: SAVED_EXPRESSION }
		});
	});

	it('queries the saved expression while the draft is untouched', async () => {
		const lib = await freshLibrary();
		await lib.setSmartList('sl1');

		expect(lastFilterExpression()).toEqual(SAVED_EXPRESSION);
		expect(lib.smartListModified).toBe(false);
	});

	it('queries the edited rules, not the saved ones', async () => {
		const lib = await freshLibrary();
		await lib.setSmartList('sl1');

		lib.setDraftConditions([{ id: 'c1', field: 'tag', op: 'contains', value: 'beta' }]);
		await lib.resetAndFetch();

		const expr = lastFilterExpression();
		expect(expr?.conditions?.[0]?.value).toBe('beta');
		expect(lib.smartListModified).toBe(true);
	});

	it('queries no filter once every rule is removed', async () => {
		const lib = await freshLibrary();
		await lib.setSmartList('sl1');

		lib.setDraftConditions([]);
		await lib.resetAndFetch();

		expect(lastFilterExpression()).toBeNull();
		expect(lib.smartListModified).toBe(true);
	});

	it('preserves a root-condition view the flat filter bar cannot represent', async () => {
		const rootCondition: Expr = {
			type: 'condition',
			field: 'tag',
			op: 'contains',
			value: 'alpha'
		};
		mockGetSmartList.mockResolvedValue({
			data: { id: 'sl2', name: 'Root', filter_expression: rootCondition }
		});

		const lib = await freshLibrary();
		await lib.setSmartList('sl2');

		expect(lastFilterExpression()).toEqual(rootCondition);
		expect(lib.smartListModified).toBe(false);
	});

	it('preserves a nested view rather than flattening it away', async () => {
		const nested: Expr = {
			type: 'and',
			conditions: [
				{ type: 'condition', field: 'tag', op: 'contains', value: 'alpha' },
				{
					type: 'or',
					conditions: [
						{ type: 'condition', field: 'domain', op: 'eq', value: 'a.example' },
						{ type: 'condition', field: 'domain', op: 'eq', value: 'b.example' }
					]
				}
			]
		};
		mockGetSmartList.mockResolvedValue({
			data: { id: 'sl3', name: 'Nested', filter_expression: nested }
		});

		const lib = await freshLibrary();
		await lib.setSmartList('sl3');

		expect(lastFilterExpression()).toEqual(nested);
		expect(lib.smartListModified).toBe(false);
		expect(lib.smartListAdvanced).toBe(true);
	});

	it('preserves a negated view', async () => {
		const negated: Expr = {
			type: 'not',
			condition: { type: 'condition', field: 'tag', op: 'contains', value: 'alpha' }
		};
		mockGetSmartList.mockResolvedValue({
			data: { id: 'sl4', name: 'Negated', filter_expression: negated }
		});

		const lib = await freshLibrary();
		await lib.setSmartList('sl4');

		expect(lastFilterExpression()).toEqual(negated);
		expect(lib.smartListAdvanced).toBe(true);
	});

	it('closes an open filter bar when switching to an advanced view', async () => {
		const nested: Expr = {
			type: 'and',
			conditions: [
				{ type: 'condition', field: 'tag', op: 'contains', value: 'alpha' },
				{
					type: 'or',
					conditions: [{ type: 'condition', field: 'domain', op: 'eq', value: 'a.example' }]
				}
			]
		};

		const lib = await freshLibrary();
		await lib.setSmartList('sl1');
		lib.toggleFilterBar();
		expect(lib.filterBarOpen).toBe(true);

		mockGetSmartList.mockResolvedValue({
			data: { id: 'sl5', name: 'Nested', filter_expression: nested }
		});
		await lib.setSmartList('sl5');

		// Leaving the bar open would render the lossy parsed rules the guard hides.
		expect(lib.filterBarOpen).toBe(false);
		expect(lastFilterExpression()).toEqual(nested);
	});

	it('treats a flat view as editable', async () => {
		const lib = await freshLibrary();
		await lib.setSmartList('sl1');

		expect(lib.smartListAdvanced).toBe(false);
	});

	it('keeps a smart list expression free of the page type scope', async () => {
		const lib = await freshLibrary();
		lib.setActiveType('articles');
		await lib.setSmartList('sl1');

		// An email-only list opened under /library/articles must not be ANDed with
		// item_type=article, which would empty the view.
		expect(JSON.stringify(lastFilterExpression())).not.toContain('item_type');
	});
});
