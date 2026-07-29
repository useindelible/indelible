import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { EntitySummaryResponse } from '$lib/api/generated/types.gen';

const mockListDocumentEntities = vi.fn();

vi.mock('$lib/api/generated', async () => {
	const actual = await vi.importActual<typeof import('$lib/api/generated')>('$lib/api/generated');
	return {
		...actual,
		listDocumentEntities: (...args: unknown[]) => mockListDocumentEntities(...args)
	};
});

import { listDocumentEntities } from '$lib/api';

function entity(): EntitySummaryResponse {
	return {
		created_at: '2026-05-01T00:00:00Z',
		entity_type: 'person',
		first_seen_at: '2026-05-01T00:00:00Z',
		id: 'ent_1',
		item_count: 2,
		last_seen_at: '2026-05-02T00:00:00Z',
		name: 'Ada Lovelace',
		object: 'entity',
		total_mentions: 5
	};
}

describe('listDocumentEntities adapter', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('forwards a document id to the generated entities endpoint', async () => {
		mockListDocumentEntities.mockResolvedValue({ data: [entity()] });

		const { data } = await listDocumentEntities({ path: { document_id: 'doc_abc' } });

		expect(mockListDocumentEntities).toHaveBeenCalledWith({ path: { document_id: 'doc_abc' } });
		expect(data?.[0]?.id).toBe('ent_1');
	});
});
