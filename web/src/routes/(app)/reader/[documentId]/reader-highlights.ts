import {
	READER_AI_DOMAIN_EVENT_TYPES,
	READER_HIGHLIGHT_DOMAIN_EVENT_TYPES
} from '$lib/realtime/event-types';

export const READER_HIGHLIGHT_EVENTS = new Set<string>(READER_HIGHLIGHT_DOMAIN_EVENT_TYPES);
export const READER_AI_EVENTS = new Set<string>(READER_AI_DOMAIN_EVENT_TYPES);

export function readerEventItemId(event: {
	aggregate_type: string;
	aggregate_id: string;
	payload: unknown;
}): string | null {
	if (event.payload && typeof event.payload === 'object' && 'document_id' in event.payload) {
		const eventDocumentId = (event.payload as { document_id?: unknown }).document_id;
		if (typeof eventDocumentId === 'string' && eventDocumentId.length > 0) return eventDocumentId;
	}
	return event.aggregate_type === 'document' ? event.aggregate_id : null;
}
