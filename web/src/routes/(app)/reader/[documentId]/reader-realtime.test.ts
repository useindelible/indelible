import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { RealtimeEventResponse } from '$lib/api/generated/types.gen';

let eventHandler: ((event: RealtimeEventResponse) => void) | undefined;

vi.mock('$lib/realtime/domain-events', () => ({
	addDomainEventHandler: vi.fn((handler: (event: RealtimeEventResponse) => void) => {
		eventHandler = handler;
		return vi.fn();
	})
}));

import { subscribeReaderRealtime } from './reader-realtime';

beforeEach(() => {
	eventHandler = undefined;
});

describe('reader realtime Mila failures', () => {
	it('preserves the completed action and run id', () => {
		const onAiCompleted = vi.fn();
		subscribeReaderRealtime('doc_1', {
			onHighlightsChanged: vi.fn(),
			onAiCompleted,
			onAiFailed: vi.fn()
		});

		eventHandler?.({
			id: 'evt_1',
			type: 'ai.output.completed',
			aggregate_type: 'document',
			aggregate_id: 'doc_1',
			payload: {
				document_id: 'doc_1',
				action: 'tags',
				ai_run_id: 'airun_2'
			},
			created_at: '2026-08-12T10:00:00Z'
		} as unknown as RealtimeEventResponse);

		expect(onAiCompleted).toHaveBeenCalledWith({ action: 'tags', aiRunId: 'airun_2' });
	});

	it('preserves the document, action, run id, and provider message', () => {
		const onAiFailed = vi.fn();
		subscribeReaderRealtime('doc_1', {
			onHighlightsChanged: vi.fn(),
			onAiCompleted: vi.fn(),
			onAiFailed
		});

		eventHandler?.({
			id: 'evt_1',
			type: 'ai.output.failed',
			aggregate_type: 'document',
			aggregate_id: 'doc_1',
			payload: {
				document_id: 'doc_1',
				action: 'summary',
				ai_run_id: 'airun_1',
				message: 'provider timed out'
			},
			created_at: '2026-08-12T10:00:00Z'
		} as unknown as RealtimeEventResponse);

		expect(onAiFailed).toHaveBeenCalledWith({
			documentId: 'doc_1',
			action: 'summary',
			aiRunId: 'airun_1',
			message: 'provider timed out'
		});
	});
});
