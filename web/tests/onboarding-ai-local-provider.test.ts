import { describe, expect, it } from 'vitest';
import {
	localOnboardingPayload,
	localOpenAiBase
} from '../src/routes/(app)/onboarding/ai/local-provider';

describe('localOpenAiBase', () => {
	it.each([
		['http://localhost:1234', 'http://localhost:1234/v1'],
		['http://localhost:1234/', 'http://localhost:1234/v1'],
		['http://localhost:1234/v1', 'http://localhost:1234/v1'],
		['http://localhost:1234/v1/', 'http://localhost:1234/v1']
	])('normalizes %s to one v1 suffix', (input, expected) => {
		expect(localOpenAiBase(input)).toBe(expected);
	});
});

it('builds an onboarding payload with exact local choices', () => {
	expect(
		localOnboardingPayload({
			endpoint: ' http://localhost:1234/ ',
			chatModel: ' gemma-4-e4b-it ',
			embeddingModel: ' text-embedding-nomic-embed-text-v1.5 '
		})
	).toEqual({
		chat_provider: 'ollama',
		embedding_provider: 'ollama',
		chat_endpoint: 'http://localhost:1234',
		embedding_endpoint: 'http://localhost:1234',
		chat_model: 'gemma-4-e4b-it',
		embedding_model: 'text-embedding-nomic-embed-text-v1.5',
		embedding_dim: 768
	});
});
