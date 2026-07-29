export interface LocalProviderSelection {
	endpoint: string;
	chatModel: string;
	embeddingModel: string;
}

export interface LocalProbe {
	chatOk: boolean;
	embeddingOk: boolean;
	chatMessage: string;
	embeddingMessage: string;
}

export function localOpenAiBase(endpoint: string): string {
	const base = endpoint.trim().replace(/\/+$/, '');
	return base.endsWith('/v1') ? base : `${base}/v1`;
}

export function localOnboardingPayload(selection: LocalProviderSelection) {
	const endpoint = selection.endpoint.trim().replace(/\/+$/, '');
	return {
		chat_provider: 'ollama',
		embedding_provider: 'ollama',
		chat_endpoint: endpoint,
		embedding_endpoint: endpoint,
		chat_model: selection.chatModel.trim(),
		embedding_model: selection.embeddingModel.trim(),
		embedding_dim: 768
	};
}
