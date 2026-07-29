import { describe, expect, it } from 'vitest';
import { formatRetrievalWarning } from '$lib/stores/mila.svelte';

describe('Mila chat store', () => {
	it('formats degraded retrieval warnings for collection chat', () => {
		expect(formatRetrievalWarning('fts_failed')).toBe(
			'Mila used semantic matches only; lexical search was unavailable.'
		);
		expect(formatRetrievalWarning('vector_failed')).toBe(
			'Mila used lexical matches only; semantic search was unavailable.'
		);
		expect(formatRetrievalWarning('embedding_failed')).toBe(
			'Mila used lexical matches only; embeddings were unavailable.'
		);
	});
});
