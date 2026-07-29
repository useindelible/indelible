import { addDomainEventHandler } from '$lib/realtime/domain-events';
import { READER_AI_EVENTS, READER_HIGHLIGHT_EVENTS, readerEventItemId } from './reader-highlights';

interface ReaderRealtimeCallbacks {
	onHighlightsChanged: () => void;
	onAiCompleted: () => void;
	onAiFailed: (message: string) => void;
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
			callbacks.onAiCompleted();
			return;
		}
		const payload = event.payload as { action?: unknown; message?: unknown };
		const action = typeof payload.action === 'string' ? payload.action : 'AI output';
		const message = typeof payload.message === 'string' ? payload.message : 'generation failed';
		callbacks.onAiFailed(`Mila ${action} failed: ${message}`);
	});
}
