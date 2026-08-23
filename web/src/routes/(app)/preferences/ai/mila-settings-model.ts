import type {
	MilaConfigResponse,
	MilaPromptPresetResponse,
	MilaPromptPresetsResponse,
	TestMilaConfigBodyWritable,
	UpsertMilaConfigBodyWritable
} from '$lib/api';
import type { MessageKey } from '$lib/i18n';

export type ActionKey = 'summary' | 'tags' | 'entities' | 'chat' | 'custom';
export type TestState = 'idle' | 'testing' | 'success' | 'error';

export interface ActionMeta {
	key: ActionKey;
	nameKey: MessageKey;
	descKey: MessageKey;
}

export interface MilaConfigDraft {
	enabled: boolean;
	byoOn: boolean;
	chatApiBase: string;
	chatApiKey: string;
	showChatApiKey: boolean;
	clearChatApiKey: boolean;
	chatModel: string;
	embeddingApiBase: string;
	embeddingApiKey: string;
	showEmbeddingApiKey: boolean;
	clearEmbeddingApiKey: boolean;
	embeddingModel: string;
	embeddingDim: number;
	modelContextWindow: number;
	chatContextPct: number;
	topK: number;
	crossItemTopK: number;
	crossItemMaxPerItem: number;
	supportsReasoningEffort: boolean;
}

export interface PresetEditorState {
	mode: 'add' | 'edit';
	action: ActionKey;
	id?: string;
	name: string;
	system_prompt: string;
	is_default: boolean;
}

export const ACTIONS: ActionMeta[] = [
	{
		key: 'summary',
		nameKey: 'prefs_ai_action_summary',
		descKey: 'prefs_ai_action_summary_hint'
	},
	{ key: 'tags', nameKey: 'prefs_ai_action_tags', descKey: 'prefs_ai_action_tags_hint' },
	{
		key: 'entities',
		nameKey: 'prefs_ai_action_entities',
		descKey: 'prefs_ai_action_entities_hint'
	},
	{
		key: 'chat',
		nameKey: 'prefs_ai_action_chat',
		descKey: 'prefs_ai_action_chat_hint'
	},
	{
		key: 'custom',
		nameKey: 'prefs_ai_action_custom',
		descKey: 'prefs_ai_action_custom_hint'
	}
];

export function emptyMilaDraft(): MilaConfigDraft {
	return {
		enabled: true,
		byoOn: false,
		chatApiBase: '',
		chatApiKey: '',
		showChatApiKey: false,
		clearChatApiKey: false,
		chatModel: '',
		embeddingApiBase: '',
		embeddingApiKey: '',
		showEmbeddingApiKey: false,
		clearEmbeddingApiKey: false,
		embeddingModel: '',
		embeddingDim: 768,
		modelContextWindow: 16000,
		chatContextPct: 70,
		topK: 6,
		crossItemTopK: 20,
		crossItemMaxPerItem: 3,
		supportsReasoningEffort: false
	};
}

export function applyMilaConfig(config: MilaConfigResponse): MilaConfigDraft {
	return {
		enabled: config.enabled,
		byoOn: config.byo_enabled,
		chatApiBase: config.chat_api_base,
		chatApiKey: '',
		showChatApiKey: false,
		clearChatApiKey: false,
		chatModel: config.chat_model,
		embeddingApiBase: config.embedding_api_base,
		embeddingApiKey: '',
		showEmbeddingApiKey: false,
		clearEmbeddingApiKey: false,
		embeddingModel: config.embedding_model,
		embeddingDim: config.embedding_dim,
		modelContextWindow: config.model_context_window,
		chatContextPct: config.chat_context_pct,
		topK: config.top_k,
		crossItemTopK: config.cross_item_top_k,
		crossItemMaxPerItem: config.cross_item_max_per_item,
		supportsReasoningEffort: config.supports_reasoning_effort
	};
}

export function milaConfigSnapshot(draft: MilaConfigDraft): string {
	return JSON.stringify({
		enabled: draft.enabled,
		byoOn: draft.byoOn,
		chatApiBase: draft.chatApiBase,
		chatModel: draft.chatModel,
		clearChatApiKey: draft.clearChatApiKey,
		embeddingApiBase: draft.embeddingApiBase,
		embeddingModel: draft.embeddingModel,
		embeddingDim: draft.embeddingDim,
		clearEmbeddingApiKey: draft.clearEmbeddingApiKey,
		modelContextWindow: draft.modelContextWindow,
		chatContextPct: draft.chatContextPct,
		supportsReasoningEffort: draft.supportsReasoningEffort
	});
}

export function buildMilaTestBody(draft: MilaConfigDraft): TestMilaConfigBodyWritable {
	const body: TestMilaConfigBodyWritable = {
		chat_api_base: draft.chatApiBase.trim(),
		chat_model: draft.chatModel.trim(),
		supports_reasoning_effort: draft.supportsReasoningEffort,
		embedding_api_base: draft.embeddingApiBase.trim(),
		embedding_model: draft.embeddingModel.trim(),
		embedding_dim: draft.embeddingDim
	};
	if (draft.chatApiKey.trim()) body.chat_api_key = draft.chatApiKey.trim();
	if (draft.embeddingApiKey.trim()) body.embedding_api_key = draft.embeddingApiKey.trim();
	return body;
}

export function buildMilaSaveBody(draft: MilaConfigDraft): UpsertMilaConfigBodyWritable {
	const body: UpsertMilaConfigBodyWritable = {
		chat_api_base: draft.chatApiBase.trim(),
		chat_model: draft.chatModel.trim(),
		embedding_api_base: draft.embeddingApiBase.trim(),
		embedding_model: draft.embeddingModel.trim(),
		embedding_dim: draft.embeddingDim,
		model_context_window: draft.modelContextWindow,
		chat_context_pct: draft.chatContextPct,
		top_k: draft.topK,
		cross_item_top_k: draft.crossItemTopK,
		cross_item_max_per_item: draft.crossItemMaxPerItem,
		enabled: draft.enabled,
		byo_enabled: draft.byoOn,
		clear_chat_api_key: draft.clearChatApiKey,
		clear_embedding_api_key: draft.clearEmbeddingApiKey,
		supports_reasoning_effort: draft.supportsReasoningEffort
	};
	if (draft.chatApiKey.trim()) body.chat_api_key = draft.chatApiKey.trim();
	if (draft.embeddingApiKey.trim()) body.embedding_api_key = draft.embeddingApiKey.trim();
	return body;
}

export function presetsForAction(
	presets: MilaPromptPresetsResponse | null,
	action: ActionKey
): MilaPromptPresetResponse[] {
	const group = presets?.groups.find((item) => item.action === action);
	return group ? group.presets : [];
}

export function createPresetEditor(action: ActionKey): PresetEditorState {
	return {
		mode: 'add',
		action,
		name: '',
		system_prompt: '',
		is_default: false
	};
}

export function editPresetEditor(
	action: ActionKey,
	preset: MilaPromptPresetResponse
): PresetEditorState | null {
	if (!preset.id) return null;
	return {
		mode: 'edit',
		action,
		id: preset.id,
		name: preset.name,
		system_prompt: preset.system_prompt,
		is_default: preset.is_default
	};
}
