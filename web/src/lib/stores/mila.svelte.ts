import {
	createMilaSession,
	getMilaConfigApi,
	getMilaSessionMessages,
	listMilaSessions,
	streamMilaChat
} from '$lib/api';
import type { MilaMessageResponse } from '$lib/api/generated/types.gen';

// -- Config singleton --

let configLoading = $state(false);
let configLoaded = $state(false);
let configConfigured = $state(false);

export function getMilaConfig() {
	async function load() {
		if (configLoaded || configLoading) return;
		configLoading = true;
		try {
			const result = await getMilaConfigApi({ throwOnError: false });
			if (result.data) {
				configConfigured = result.data.enabled;
			} else {
				configConfigured = false;
			}
		} catch {
			configConfigured = false;
		} finally {
			configLoading = false;
			configLoaded = true;
		}
	}

	return {
		get loading() {
			return configLoading;
		},
		get loaded() {
			return configLoaded;
		},
		get configured() {
			return configConfigured;
		},
		load
	};
}

// -- Per-instance chat factory --

export type ChatScope =
	| { type: 'single_document'; documentId: string }
	| { type: 'collection'; collectionId: string };

export interface ChatMessage {
	id: string;
	role: 'user' | 'assistant';
	content: string;
	source_refs: MilaMessageResponse['source_refs'];
	streaming: boolean;
}

export function createMilaChat(scope: ChatScope) {
	let messages = $state<ChatMessage[]>([]);
	let loading = $state(false);
	let streaming = $state(false);
	let error = $state<string | null>(null);
	let retrievalWarning = $state<string | null>(null);
	let lastQuestion = $state('');
	let sessionId = $state<string | null>(null);
	let abortController: AbortController | null = null;

	async function initialize() {
		loading = true;
		error = null;
		retrievalWarning = null;
		messages = [];
		sessionId = null;

		try {
			const listResult = await listMilaSessions({ throwOnError: false });
			if (listResult.data) {
				const existing = listResult.data.sessions.find((s) => {
					if (scope.type === 'single_document') {
						return s.session_type === 'single_document' && s.document_id === scope.documentId;
					} else {
						return s.session_type === 'collection' && s.collection_id === scope.collectionId;
					}
				});
				if (existing) {
					sessionId = existing.id;
					await loadMessages(existing.id);
					return;
				}
			}

			await startNewSession();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to initialize chat';
		} finally {
			loading = false;
		}
	}

	async function startNewSession() {
		const body =
			scope.type === 'single_document'
				? { session_type: 'single_document' as const, document_id: scope.documentId }
				: { session_type: 'collection' as const, collection_id: scope.collectionId };

		const result = await createMilaSession({ body, throwOnError: true });
		sessionId = result.data.id;
		messages = [];
	}

	async function loadMessages(sid: string) {
		const result = await getMilaSessionMessages({
			path: { session_id: sid },
			throwOnError: false
		});
		if (result.data) {
			messages = result.data.messages.map((m) => ({
				id: m.id,
				role: m.role as 'user' | 'assistant',
				content: m.content,
				source_refs: m.source_refs,
				streaming: false
			}));
		}
	}

	async function sendMessage(question: string) {
		if (!sessionId || streaming) return;

		lastQuestion = question;
		error = null;
		retrievalWarning = null;

		const userMsg: ChatMessage = {
			id: crypto.randomUUID(),
			role: 'user',
			content: question,
			source_refs: [],
			streaming: false
		};
		const assistantMsg: ChatMessage = {
			id: crypto.randomUUID(),
			role: 'assistant',
			content: '',
			source_refs: [],
			streaming: true
		};
		messages = [...messages, userMsg, assistantMsg];
		streaming = true;

		abortController = new AbortController();

		try {
			// The generated SSE client swallows non-2xx responses and retries forever by
			// default; cap it at one attempt and surface the failure through onSseError.
			let streamFailure: unknown = null;
			const sseResult = await streamMilaChat({
				query: {
					session_id: sessionId,
					question
				},
				signal: abortController.signal,
				throwOnError: false,
				sseMaxRetryAttempts: 1,
				onSseError: (sseError: unknown) => {
					streamFailure = sseError;
				}
			});

			for await (const event of sseResult.stream) {
				const raw = event as unknown;
				if (typeof raw === 'string') {
					if (raw === '[DONE]') break;
					continue;
				}
				if (typeof raw === 'object' && raw !== null) {
					if ('error' in raw) {
						const errObj = raw as { error: string };
						throw new Error(errObj.error);
					}
					if ('delta' in raw) {
						const deltaObj = raw as { delta: string; retrieval_degraded?: string };
						if (deltaObj.retrieval_degraded) {
							retrievalWarning = formatRetrievalWarning(deltaObj.retrieval_degraded);
						}
						const last = messages[messages.length - 1];
						if (last && last.streaming) {
							last.content += deltaObj.delta;
						}
					}
				}
			}

			if (streamFailure) {
				throw toChatStreamError(streamFailure);
			}

			// Reload canonical messages with server IDs and source_refs
			await loadMessages(sessionId);
		} catch (e) {
			if ((e as { name?: string }).name === 'AbortError') return;
			error = e instanceof Error ? e.message : 'Chat failed';
			// Remove the partial streaming assistant message on error
			messages = messages.filter((m) => !m.streaming);
		} finally {
			streaming = false;
			const last = messages[messages.length - 1];
			if (last?.streaming) {
				last.streaming = false;
			}
		}
	}

	function retry() {
		if (!lastQuestion) return;
		// Drop the optimistic user bubble left by the failed attempt so the resend
		// does not accumulate duplicates (the failed send persisted nothing).
		const last = messages[messages.length - 1];
		if (last && last.role === 'user' && last.content === lastQuestion) {
			messages = messages.slice(0, -1);
		}
		return sendMessage(lastQuestion);
	}

	function destroy() {
		abortController?.abort();
	}

	return {
		get messages() {
			return messages;
		},
		get loading() {
			return loading;
		},
		get streaming() {
			return streaming;
		},
		get error() {
			return error;
		},
		get retrievalWarning() {
			return retrievalWarning;
		},
		initialize,
		sendMessage,
		retry,
		destroy
	};
}

function toChatStreamError(failure: unknown): Error {
	const message = failure instanceof Error ? failure.message : String(failure);
	if (message.includes('503')) {
		return new Error(
			'Your AI provider is unreachable. Start it (e.g. LM Studio), then press Retry.'
		);
	}
	return new Error('Chat failed — please try again.');
}

export function formatRetrievalWarning(reason: string) {
	const reasons = reason.split(',').map((part) => part.trim());
	if (reasons.includes('fts_failed') && reasons.includes('vector_failed')) {
		return 'Mila used partial collection search; lexical and semantic retrieval both degraded.';
	}
	if (reasons.includes('fts_failed') && reasons.includes('embedding_failed')) {
		return 'Mila used partial collection search; lexical search and embeddings were unavailable.';
	}
	if (reasons.includes('fts_failed')) {
		return 'Mila used semantic matches only; lexical search was unavailable.';
	}
	if (reasons.includes('embedding_failed')) {
		return 'Mila used lexical matches only; embeddings were unavailable.';
	}
	if (reasons.includes('vector_failed')) {
		return 'Mila used lexical matches only; semantic search was unavailable.';
	}
	return 'Mila used partial collection search for this answer.';
}
