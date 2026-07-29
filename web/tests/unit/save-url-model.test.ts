import { describe, expect, it } from 'vitest';

import {
	addSaveUrlTag,
	duplicateFromConflictError,
	formatDuplicateSavedDate,
	getSelectedCollectionName,
	messageForUrlValidation,
	removeSaveUrlTag,
	validateSaveUrl
} from '$lib/components/library/save-url-model';

describe('save url model', () => {
	it('validates URL input before saving', () => {
		expect(validateSaveUrl('')).toBe('empty');
		expect(validateSaveUrl('ftp://example.com')).toBe('invalid');
		expect(validateSaveUrl('not a url')).toBe('invalid');
		expect(validateSaveUrl('https://example.com')).toBe('');
		expect(messageForUrlValidation('empty')).toBe('Please paste a URL.');
		expect(messageForUrlValidation('invalid')).toBe('That does not look like a valid URL.');
	});

	it('normalizes and deduplicates inline tags', () => {
		expect(addSaveUrlTag([], ' Machine Learning ')).toEqual(['machine-learning']);
		expect(addSaveUrlTag(['machine-learning'], 'machine learning')).toEqual(['machine-learning']);
		expect(removeSaveUrlTag(['research', 'ai'], 'research')).toEqual(['ai']);
	});

	it('extracts duplicate preview data from conflict errors', () => {
		expect(
			duplicateFromConflictError({
				id: 'item_1',
				title: 'Existing item',
				domain: 'example.com',
				created_at: '2026-01-02T00:00:00Z'
			})
		).toEqual({
			id: 'item_1',
			title: 'Existing item',
			domain: 'example.com',
			savedDate: '2026-01-02T00:00:00Z'
		});

		expect(duplicateFromConflictError({ id: 'item_2' })).toEqual({
			id: 'item_2',
			title: 'Already saved',
			domain: null,
			savedDate: null
		});
		expect(duplicateFromConflictError({ title: 'missing id' })).toBeNull();
	});

	it('formats duplicate dates and collection labels', () => {
		expect(formatDuplicateSavedDate(null)).toBe('');
		expect(formatDuplicateSavedDate('bad date')).toBe('');
		expect(formatDuplicateSavedDate('2026-01-02T00:00:00Z')).toMatch(/2026/);
		expect(getSelectedCollectionName(null, [])).toBe('Inbox');
		expect(getSelectedCollectionName('col_1', [{ id: 'col_1', name: 'Reading' }])).toBe('Reading');
		expect(getSelectedCollectionName('col_404', [])).toBe('Collection');
	});
});
