import { streamEvents } from '$lib/api';
import type { RealtimeEventResponse } from '$lib/api/generated/types.gen';

const CURSOR_KEY_PREFIX = 'indelible:domain-events:last-seen:';

let activeUserId: string | null = null;
let activeSubscriptionKey: string | null = null;
let activeController: AbortController | null = null;
const handlers = new Set<DomainEventHandler>();

export type DomainEventHandler = (event: RealtimeEventResponse) => void | Promise<void>;

export type DomainEventStreamOptions = {
	eventTypes: readonly string[];
};

function normalizeEventTypes(eventTypes: readonly string[]): string[] {
	return [...new Set(eventTypes.map((eventType) => eventType.trim()).filter(Boolean))].sort();
}

function subscriptionKey(eventTypes: readonly string[]): string {
	return eventTypes.join(',');
}

function cursorKey(userId: string, eventTypes: readonly string[]): string {
	return `${CURSOR_KEY_PREFIX}${userId}:${subscriptionKey(eventTypes)}`;
}

function readLastSeenId(userId: string, eventTypes: readonly string[]): string | null {
	if (typeof localStorage === 'undefined') return null;
	return localStorage.getItem(cursorKey(userId, eventTypes));
}

function writeLastSeenId(userId: string, eventTypes: readonly string[], eventId: string): void {
	if (typeof localStorage === 'undefined') return;
	localStorage.setItem(cursorKey(userId, eventTypes), eventId);
}

export function addDomainEventHandler(handler: DomainEventHandler): () => void {
	handlers.add(handler);
	return () => {
		handlers.delete(handler);
	};
}

async function dispatchDomainEvent(event: RealtimeEventResponse): Promise<void> {
	for (const handler of [...handlers]) {
		try {
			await handler(event);
		} catch (error) {
			console.warn('domain event handler failed', error);
		}
	}
}

export function startDomainEventStream(userId: string, options: DomainEventStreamOptions): void {
	const eventTypes = normalizeEventTypes(options.eventTypes);
	if (eventTypes.length === 0) {
		console.warn('domain event stream not started: no event types requested');
		return;
	}

	const nextSubscriptionKey = subscriptionKey(eventTypes);
	if (
		activeUserId === userId &&
		activeSubscriptionKey === nextSubscriptionKey &&
		activeController &&
		!activeController.signal.aborted
	) {
		return;
	}

	stopDomainEventStream();

	const controller = new AbortController();
	activeUserId = userId;
	activeSubscriptionKey = nextSubscriptionKey;
	activeController = controller;
	const lastSeenId = readLastSeenId(userId, eventTypes);

	void (async () => {
		try {
			const { stream } = await streamEvents({
				headers: lastSeenId ? { 'Last-Event-ID': lastSeenId } : undefined,
				query: { event_type: eventTypes },
				signal: controller.signal
			});

			for await (const event of stream) {
				if (controller.signal.aborted) break;
				await dispatchDomainEvent(event);
				writeLastSeenId(userId, eventTypes, event.id);
			}
		} catch (error) {
			if (!controller.signal.aborted) {
				console.warn('domain event stream stopped', error);
			}
		}
	})();
}

export function stopDomainEventStream(): void {
	activeController?.abort();
	activeController = null;
	activeUserId = null;
	activeSubscriptionKey = null;
}
