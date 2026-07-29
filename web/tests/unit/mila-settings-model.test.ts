import { describe, expect, it } from 'vitest';
import type { MilaConfigResponse, MilaPromptPresetsResponse } from '$lib/api';
import {
	ACTIONS,
	applyMilaConfig,
	buildMilaSaveBody,
	buildMilaTestBody,
	createPresetEditor,
	milaConfigSnapshot,
	presetsForAction
} from '../../src/routes/(app)/preferences/ai/mila-settings-model';

function config(overrides: Partial<MilaConfigResponse> = {}): MilaConfigResponse {
	return {
		chat_api_base: 'https://chat.example.com/v1',
		chat_model: 'gpt-test',
		model_context_window: 16000,
		chat_context_pct: 70,
		cross_item_max_per_item: 3,
		cross_item_top_k: 20,
		embedding_api_base: 'https://embed.example.com/v1',
		embedding_dim: 768,
		embedding_model: 'text-embedding-test',
		enabled: true,
		byo_enabled: true,
		has_chat_api_key: true,
		has_embedding_api_key: true,
		supports_structured_output: true,
		supports_reasoning_effort: true,
		top_k: 6,
		...overrides
	};
}

describe('mila settings model', () => {
	it('creates a UI draft from server config and snapshots dirty fields', () => {
		const draft = applyMilaConfig(config({ enabled: false, chat_api_base: '' }));

		expect(draft.enabled).toBe(false);
		expect(draft.byoOn).toBe(true);
		expect(milaConfigSnapshot(draft)).toEqual(
			JSON.stringify({
				enabled: false,
				byoOn: true,
				chatApiBase: '',
				chatModel: 'gpt-test',
				clearChatApiKey: false,
				embeddingApiBase: 'https://embed.example.com/v1',
				embeddingModel: 'text-embedding-test',
				embeddingDim: 768,
				clearEmbeddingApiKey: false,
				modelContextWindow: 16000,
				chatContextPct: 70,
				supportsReasoningEffort: true
			})
		);
	});

	it('builds trimmed test and save bodies', () => {
		const draft = applyMilaConfig(config());
		draft.chatApiBase = ' https://chat.test/v1 ';
		draft.chatApiKey = ' sk-chat-test ';
		draft.chatModel = ' gpt-4.1-mini ';
		draft.embeddingApiBase = ' https://embed.test/v1 ';
		draft.embeddingApiKey = ' sk-embed-test ';
		draft.embeddingModel = ' text-embedding-3-small ';

		expect(buildMilaTestBody(draft)).toEqual({
			chat_api_base: 'https://chat.test/v1',
			chat_api_key: 'sk-chat-test',
			chat_model: 'gpt-4.1-mini',
			embedding_api_base: 'https://embed.test/v1',
			embedding_api_key: 'sk-embed-test',
			embedding_model: 'text-embedding-3-small',
			embedding_dim: 768
		});
		expect(buildMilaSaveBody(draft).embedding_dim).toBe(768);
		expect(buildMilaSaveBody(draft).cross_item_top_k).toBe(20);
		expect(buildMilaSaveBody(draft).supports_reasoning_effort).toBe(true);
	});

	it('omits api keys when retesting saved keys without re-entering them', () => {
		const draft = applyMilaConfig(config({ has_chat_api_key: true, has_embedding_api_key: true }));

		expect(buildMilaTestBody(draft)).toEqual({
			chat_api_base: 'https://chat.example.com/v1',
			chat_model: 'gpt-test',
			embedding_api_base: 'https://embed.example.com/v1',
			embedding_model: 'text-embedding-test',
			embedding_dim: 768
		});
	});

	it('groups prompt presets by action and creates add editors', () => {
		const presets: MilaPromptPresetsResponse = {
			groups: [
				{
					action: 'summary',
					presets: [
						{
							action: 'summary',
							id: 'preset_1',
							is_built_in: false,
							is_default: true,
							name: 'Brief',
							system_prompt: 'Summarize briefly.'
						}
					]
				}
			]
		};

		expect(ACTIONS.map((action) => action.key)).toEqual([
			'summary',
			'tags',
			'entities',
			'chat',
			'custom'
		]);
		expect(presetsForAction(presets, 'summary')).toHaveLength(1);
		expect(presetsForAction(presets, 'chat')).toEqual([]);
		expect(createPresetEditor('chat')).toMatchObject({ mode: 'add', action: 'chat' });
	});
});
