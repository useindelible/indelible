import { describe, it, expect } from 'vitest';
import { INTEGRATION_PROVIDERS, findProvider } from '$lib/integrations/providers';

describe('INTEGRATION_PROVIDERS', () => {
	it('declares the M10-wave providers', () => {
		const ids = INTEGRATION_PROVIDERS.map((p) => p.id).sort();
		expect(ids).toEqual(['notion', 'obsidian', 'readwise']);
	});

	it('every provider has a non-empty display name and description', () => {
		for (const provider of INTEGRATION_PROVIDERS) {
			expect(provider.displayName.length).toBeGreaterThan(0);
			expect(provider.descriptionKey.length).toBeGreaterThan(0);
		}
	});

	it('every importUpload provider declares an importSlug', () => {
		const importable = INTEGRATION_PROVIDERS.filter((p) => p.capabilities.includes('importUpload'));
		expect(importable.length).toBeGreaterThan(0);
		for (const provider of importable) {
			expect(provider.importSlug).toBeDefined();
			expect(provider.importSlug?.length).toBeGreaterThan(0);
		}
	});

	it('non-importUpload providers do not need an importSlug', () => {
		for (const provider of INTEGRATION_PROVIDERS) {
			if (!provider.capabilities.includes('importUpload')) {
				expect(provider.importSlug).toBeUndefined();
			}
		}
	});
});

describe('findProvider', () => {
	it('finds a provider by id', () => {
		expect(findProvider('obsidian')?.id).toBe('obsidian');
	});

	it('returns undefined for unknown ids', () => {
		expect(findProvider('does-not-exist')).toBeUndefined();
	});
});
