import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import type { RealtimeEventResponse } from '$lib/api/generated/types.gen';

let eventHandler: ((event: RealtimeEventResponse) => void) | undefined;

vi.mock('$app/paths', () => ({ resolve: (path: string) => path }));
vi.mock('$lib/api', () => ({
	listDocumentEntities: vi.fn(async () => ({ data: [] }))
}));
vi.mock('$lib/realtime/domain-events', () => ({
	addDomainEventHandler: vi.fn((handler: (event: RealtimeEventResponse) => void) => {
		eventHandler = handler;
		return vi.fn();
	})
}));

import EntitiesSection from '../../src/lib/components/library/EntitiesSection.svelte';

describe('EntitiesSection', () => {
	it('shows friendly entity failure copy without exposing the provider message', async () => {
		render(EntitiesSection, { itemId: 'doc_1' });
		await screen.findByText('No entities extracted yet.');

		eventHandler?.({
			id: 'evt_1',
			type: 'ai.output.failed',
			aggregate_type: 'document',
			aggregate_id: 'doc_1',
			payload: {
				document_id: 'doc_1',
				action: 'entities',
				ai_run_id: 'airun_1',
				message: 'raw provider stack trace'
			},
			created_at: '2026-08-12T10:00:00Z'
		} as unknown as RealtimeEventResponse);

		await waitFor(() => expect(screen.getByText("Mila couldn't extract entities.")).toBeTruthy());
		expect(screen.queryByText('raw provider stack trace')).toBeNull();
	});
});
