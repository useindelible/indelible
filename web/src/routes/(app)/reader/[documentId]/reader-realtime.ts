import { addDomainEventHandler } from '$lib/realtime/domain-events';
import { READER_AI_EVENTS, READER_HIGHLIGHT_EVENTS, readerEventItemId } from './reader-highlights';

interface ReaderRealtimeCallbacks {
	onHighlightsChanged: () => void;
	onAiCompleted: (completion: ReaderAiCompletion) => void;
	onAiFailed: (failure: ReaderAiFailure) => void;
}

export interface ReaderAiCompletion {
	action: string;
	aiRunId: string;
}

export interface ReaderAiFailure {
	documentId: string;
	action: string;
	aiRunId: string;
	message: string;
}

export function subscribeReaderRealtime(
	documentId: string,
	callbacks: ReaderRealtimeCallbacks
): () => void {
	return addDomainEventHandler((event) => {
		if (readerEventItemId(event) !== documentId) return;
		if (READER_HIGHLIGHT_EVENTS.has(event.type)) {
			callbacks.onHighlightsChanged();
			return;
		}
		if (!READER_AI_EVENTS.has(event.type)) return;
		if (event.type === 'ai.output.completed') {
			const payload = event.payload as { action?: unknown; ai_run_id?: unknown };
			callbacks.onAiCompleted({
				action: typeof payload.action === 'string' ? payload.action : '',
				aiRunId: typeof payload.ai_run_id === 'string' ? payload.ai_run_id : ''
			});
			return;
		}
		const payload = event.payload as {
			document_id?: unknown;
			action?: unknown;
			ai_run_id?: unknown;
			message?: unknown;
		};
		const failedDocumentId =
			typeof payload.document_id === 'string' ? payload.document_id : documentId;
		const action = typeof payload.action === 'string' ? payload.action : 'AI output';
		const message = typeof payload.message === 'string' ? payload.message : 'generation failed';
		const aiRunId = typeof payload.ai_run_id === 'string' ? payload.ai_run_id : '';
		callbacks.onAiFailed({ documentId: failedDocumentId, action, aiRunId, message });
	});
}
