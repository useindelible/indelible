import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

import { checkCatalogs } from '../scripts/i18n-check.mjs';

const testDirectory = dirname(fileURLToPath(import.meta.url));
const fixture = (name: string) => resolve(testDirectory, 'fixtures/i18n', name);

describe('i18n catalog checker', () => {
	it('accepts reference parity and reports missing optional translations', async () => {
		const result = await checkCatalogs({ localesDir: fixture('valid') });

		expect(result.errors).toEqual([]);
		expect(result.summary.de).toEqual({ total: 2, missing: 1 });
	});

	it.each([
		['unsorted', 'b', 'sorted'],
		['duplicate', 'a', 'duplicate'],
		['empty-value', 'a', 'non-empty'],
		['bad-icu', 'a', 'ICU'],
		['stale-key', 'stale', 'unknown'],
		['placeholder-mismatch', 'greeting', 'arguments'],
		['reference-missing', 'b', 'missing']
	])('rejects %s catalogs', async (name, key, rule) => {
		const result = await checkCatalogs({ localesDir: fixture(name) });

		expect(result.errors.join('\n')).toContain(key);
		expect(result.errors.join('\n').toLowerCase()).toContain(rule.toLowerCase());
	});

	it('allows missing keys in non-reference locales', async () => {
		const result = await checkCatalogs({ localesDir: fixture('non-reference-missing') });

		expect(result.errors).toEqual([]);
		expect(result.summary.de?.missing).toBe(1);
	});

	it('rejects user-visible source literals without flagging machine values or product names', async () => {
		const result = await checkCatalogs({
			localesDir: fixture('valid'),
			sourcesDir: fixture('source-literals')
		});
		const errors = result.errors.join('\n');

		expect(errors).toContain('Open menu');
		expect(errors).toContain('Save changes');
		expect(errors).toContain('pending');
		expect(errors).toContain('Connected workspace');
		expect(errors).toContain('Could not save');
		expect(errors).not.toContain('Indelible');
		expect(errors).not.toContain('connected_state');
	});

	it('rejects catalog keys outside the configured feature prefixes', async () => {
		const result = await checkCatalogs({
			localesDir: fixture('bad-prefix'),
			allowedPrefixes: ['common_']
		});

		expect(result.errors.join('\n')).toContain('misc_label: disallowed key prefix');
	});
});
