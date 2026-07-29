import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
	createMilaSession: vi.fn(),
	getMilaConfigApi: vi.fn(),
	getMilaSessionMessages: vi.fn(),
	listMilaSessions: vi.fn(),
	streamMilaChat: vi.fn()
}));

vi.mock('$lib/api', () => mocks);

import { createMilaChat } from './mila.svelte';

function sessionListResult() {
	return {
		data: {
			sessions: [{ id: 'ses_1', session_type: 'single_document', document_id: 'doc_1' }]
		}
	};
}

async function initializedChat() {
	mocks.listMilaSessions.mockResolvedValue(sessionListResult());
	mocks.getMilaSessionMessages.mockResolvedValue({ data: { messages: [] } });
	const chat = createMilaChat({ type: 'single_document', documentId: 'doc_1' });
	await chat.initialize();
	return chat;
}

function providerDownStream() {
	return (options: { onSseError?: (error: unknown) => void }) => {
		options.onSseError?.(new Error('SSE failed: 503 Service Unavailable'));
		return Promise.resolve({ stream: (async function* () {})() });
	};
}

describe('mila chat provider outage handling', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('bounds SSE retries so a dead provider cannot spin the stream forever', async () => {
		mocks.streamMilaChat.mockImplementation(providerDownStream());
		const chat = await initializedChat();

		await chat.sendMessage('why is the sky blue?');

		const options = mocks.streamMilaChat.mock.calls.at(0)?.[0];
		expect(options?.sseMaxRetryAttempts).toBe(1);
	});

	it('maps a pre-stream 503 to the friendly provider-offline message', async () => {
		mocks.streamMilaChat.mockImplementation(providerDownStream());
		const chat = await initializedChat();

		await chat.sendMessage('why is the sky blue?');

		expect(chat.error).toBe(
			'Your AI provider is unreachable. Start it (e.g. LM Studio), then press Retry.'
		);
		expect(chat.streaming).toBe(false);
		expect(chat.messages.some((m) => m.streaming)).toBe(false);
	});

	it('retry resends the question that failed', async () => {
		mocks.streamMilaChat.mockImplementation(providerDownStream());
		const chat = await initializedChat();

		await chat.sendMessage('why is the sky blue?');
		await chat.retry();

		expect(mocks.streamMilaChat).toHaveBeenCalledTimes(2);
		const retryOptions = mocks.streamMilaChat.mock.calls.at(1)?.[0];
		expect(retryOptions?.query.question).toBe('why is the sky blue?');
	});

	it('repeated retries do not accumulate duplicate optimistic questions', async () => {
		mocks.streamMilaChat.mockImplementation(providerDownStream());
		const chat = await initializedChat();

		await chat.sendMessage('why is the sky blue?');
		await chat.retry();
		await chat.retry();

		const userMessages = chat.messages.filter((m) => m.role === 'user');
		expect(userMessages).toHaveLength(1);
		expect(userMessages[0]?.content).toBe('why is the sky blue?');
	});
});
