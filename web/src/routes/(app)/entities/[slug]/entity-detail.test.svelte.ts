import { render, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { EntityDetailResponse } from '$lib/api';

const getEntity = vi.fn();
const listEntityDocuments = vi.fn();

vi.mock('$app/state', async () => await import('./entity-detail.test-page.svelte'));
vi.mock('$app/paths', () => ({ resolve: (path: string) => path }));
vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$lib/api', () => ({
	getEntity: (...args: unknown[]) => getEntity(...args),
	listEntityDocuments: (...args: unknown[]) => listEntityDocuments(...args)
}));
vi.mock('$lib/components/library/LibrarySidebar.svelte', () => ({ default: () => ({}) }));

import { page } from './entity-detail.test-page.svelte';
import EntityPage from './+page.svelte';

function entity(id: string, name: string): { data: EntityDetailResponse } {
	return {
		data: {
			id,
			name,
			object: 'entity',
			entity_type: 'person',
			co_occurring: [],
			created_at: '2026-01-01T00:00:00Z',
			first_seen_at: '2026-01-01T00:00:00Z',
			last_seen_at: '2026-01-01T00:00:00Z',
			item_count: 0,
			total_mentions: 0
		} as EntityDetailResponse
	};
}

function deferred<T>() {
	let resolve!: (value: T) => void;
	const promise = new Promise<T>((r) => (resolve = r));
	return { promise, resolve };
}

beforeEach(() => {
	page.params = { slug: 'ent_a' };
	getEntity.mockReset();
	listEntityDocuments.mockReset();
	listEntityDocuments.mockResolvedValue({ data: { data: [] } });
});

describe('entity detail loading', () => {
	it('ignores a superseded response for the previous entity', async () => {
		const slow = deferred<{ data: EntityDetailResponse }>();
		getEntity.mockImplementationOnce(() => slow.promise);
		getEntity.mockImplementationOnce(() => Promise.resolve(entity('ent_b', 'Bravo')));

		render(EntityPage);

		page.params = { slug: 'ent_b' };
		await waitFor(() => expect(document.body.textContent).toContain('Bravo'));

		slow.resolve(entity('ent_a', 'Alpha'));
		await Promise.resolve();
		await waitFor(() => expect(document.body.textContent).toContain('Bravo'));
		expect(document.body.textContent).not.toContain('Alpha');
	});
});
