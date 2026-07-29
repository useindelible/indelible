import { describe, expect, it } from 'vitest';

import {
	buildSearchQuery,
	FILTER_HINTS,
	getDomain,
	parseEntityPrefix
} from '../../src/routes/(app)/search/search-page-model';

describe('search page model', () => {
	it('parses entity-prefixed URL queries', () => {
		expect(parseEntityPrefix('entity:"Mila" retrieval')).toEqual({
			entityName: 'Mila',
			entityType: '',
			remainder: 'retrieval'
		});
		expect(parseEntityPrefix('entity:"Mila"')).toEqual({
			entityName: 'Mila',
			entityType: '',
			remainder: ''
		});
		expect(parseEntityPrefix('tag:research')).toBeNull();
	});

	it('builds active entity queries around trimmed user input', () => {
		expect(buildSearchQuery('  vector notes  ', null)).toBe('vector notes');
		expect(buildSearchQuery('', { name: 'Mila', entityType: 'PERSON' })).toBe('entity:"Mila"');
		expect(buildSearchQuery(' retrieval ', { name: 'Mila', entityType: 'PERSON' })).toBe(
			'entity:"Mila" retrieval'
		);
	});

	it('extracts bare domains safely', () => {
		expect(getDomain('https://www.example.com/post')).toBe('example.com');
		expect(getDomain('not a url')).toBe('');
	});

	it('keeps the advanced filter hints in one shared list', () => {
		expect(FILTER_HINTS).toContain('sender_domain:');
		expect(FILTER_HINTS).toContain('pinned:');
	});
});
