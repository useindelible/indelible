import { describe, expect, it } from 'vitest';

/* eslint-disable svelte-runes/no-external-svelte-imports -- test file importing pure util from svelte store */
import {
	buildLibraryItemsQueryBody,
	buildSmartListItemsQueryBody,
	triageOptionsForMode
} from '../../src/lib/stores/library.svelte';
import type { FilterExpression } from '../../src/lib/utils/filter-expression';
import type { FilterCondition } from '../../src/lib/utils/filter-expression';

describe('buildLibraryItemsQueryBody', () => {
	it('maps Simple triage mode to saved and archived tabs', () => {
		expect(triageOptionsForMode('manual')).toEqual([
			{ value: 'inbox', labelKey: 'library_triage_saved' },
			{ value: 'archive', labelKey: 'library_triage_archived' }
		]);
	});

	it('maps Triage mode to inbox, later, and archive tabs', () => {
		expect(triageOptionsForMode('focus')).toEqual([
			{ value: 'inbox', labelKey: 'library_triage_inbox' },
			{ value: 'later', labelKey: 'library_triage_later' },
			{ value: 'archive', labelKey: 'library_triage_archive' }
		]);
	});

	it('adds ambient item type and triage state filters when not explicitly set', () => {
		const body = buildLibraryItemsQueryBody({
			draftConditions: [],
			draftConjunction: 'and',
			activeType: 'articles',
			groupBy: 'triage',
			triageTab: 'later',
			cursor: undefined,
			limit: 50
		});

		expect(body.filter_expression).toEqual({
			type: 'and',
			conditions: [
				{ type: 'condition', field: 'item_type', op: 'eq', value: 'article' },
				{ type: 'condition', field: 'triage_state', op: 'eq', value: 'later' }
			]
		});
	});

	it('does not duplicate explicit item type or triage state conditions', () => {
		const draftConditions: FilterCondition[] = [
			{ id: '1', field: 'item_type', op: 'eq', value: 'video' },
			{ id: '2', field: 'triage_state', op: 'eq', value: 'archive' }
		];

		const body = buildLibraryItemsQueryBody({
			draftConditions,
			draftConjunction: 'and',
			activeType: 'articles',
			groupBy: 'triage',
			triageTab: 'later',
			cursor: undefined,
			limit: 50
		});

		expect(body.filter_expression).toEqual({
			type: 'and',
			conditions: [
				{ type: 'condition', field: 'item_type', op: 'eq', value: 'video' },
				{ type: 'condition', field: 'triage_state', op: 'eq', value: 'archive' }
			]
		});
	});

	it('ands ambient scope with an or-based draft expression', () => {
		const draftConditions: FilterCondition[] = [
			{ id: '1', field: 'domain', op: 'eq', value: 'example.com' },
			{ id: '2', field: 'is_favorite', op: 'eq', value: true }
		];

		const body = buildLibraryItemsQueryBody({
			draftConditions,
			draftConjunction: 'or',
			activeType: 'articles',
			groupBy: 'triage',
			triageTab: 'inbox',
			cursor: 'cursor-1',
			limit: 25
		});

		expect(body).toEqual({
			filter_expression: {
				type: 'and',
				conditions: [
					{
						type: 'or',
						conditions: [
							{ type: 'condition', field: 'domain', op: 'eq', value: 'example.com' },
							{ type: 'condition', field: 'is_favorite', op: 'eq', value: true }
						]
					},
					{
						type: 'and',
						conditions: [
							{ type: 'condition', field: 'item_type', op: 'eq', value: 'article' },
							{ type: 'condition', field: 'triage_state', op: 'eq', value: 'inbox' }
						]
					}
				]
			},
			cursor: 'cursor-1',
			limit: 25
		});
	});

	it('sends the smart list expression verbatim, never composed with a page type scope', () => {
		const savedExpression: FilterExpression = {
			type: 'and',
			conditions: [{ type: 'condition', field: 'sender', op: 'contains', value: 'brew' }]
		};

		const body = buildSmartListItemsQueryBody({
			filterExpression: savedExpression,
			cursor: 'cursor-1',
			limit: 25
		});

		expect(body).toEqual({
			filter_expression: savedExpression,
			cursor: 'cursor-1',
			limit: 25
		});
	});

	it('sends a null expression for a smart list without one', () => {
		const body = buildSmartListItemsQueryBody({
			filterExpression: null,
			cursor: undefined,
			limit: 50
		});

		expect(body).toEqual({
			filter_expression: null,
			cursor: null,
			limit: 50
		});
	});

	it('passes a saved legacy podcast filter through verbatim', () => {
		const savedExpression: FilterExpression = {
			type: 'condition',
			field: 'item_type',
			op: 'eq',
			value: 'podcast'
		};

		expect(
			buildSmartListItemsQueryBody({
				filterExpression: savedExpression,
				cursor: undefined,
				limit: 50
			}).filter_expression
		).toEqual(savedExpression);
	});
});
