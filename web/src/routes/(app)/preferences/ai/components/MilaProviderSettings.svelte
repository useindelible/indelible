<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import type { MilaConfigResponse } from '$lib/api';
	import type { MilaConfigDraft, TestState } from '../mila-settings-model';
	import ApiKeyField from './ApiKeyField.svelte';
	import CheckRow from './CheckRow.svelte';
	import ProviderChatBudget from './ProviderChatBudget.svelte';
	import ProviderConnectionCard from './ProviderConnectionCard.svelte';

	interface Props {
		config: MilaConfigResponse | null;
		draft: MilaConfigDraft;
		testMessage: string;
		testState: TestState;
		onChange: (patch: Partial<MilaConfigDraft>) => void;
		onTestConnection: () => void;
	}

	let { config, draft, testMessage, testState, onChange, onTestConnection }: Props = $props();

	const uid = $props.id();
	const chatBaseId = `${uid}-chat-base`;
	const chatModelId = `${uid}-chat-model`;
	const embeddingBaseId = `${uid}-embedding-base`;
	const embeddingModelId = `${uid}-embedding-model`;

	const REASONING_HELP = {
		on: 'On — standard sampling controls are omitted. No reasoning_effort value is sent; the provider uses its default reasoning level.',
		off: 'Off — Indelible sends its normal per-task sampling controls; chat includes temperature and top_p. For LM Studio reasoning models, compatibility requires LM Studio 0.4.8 or newer.'
	};

	const testDisabled = $derived(!draft.chatApiBase.trim() || !draft.embeddingApiBase.trim());
</script>

<SettingsGroup
	title="Provider"
	meta={draft.byoOn
		? 'Bring your own OpenAI-compatible endpoint'
		: 'Powered by Indelible — included in your plan'}
>
	<div class="byo-shell" class:expanded={draft.byoOn}>
		<div class="byo-toggle-row">
			<div class="byo-toggle-text">
				<div class="byo-toggle-title">Use my own AI provider</div>
				<div class="byo-toggle-sub">
					Connect any OpenAI-compatible endpoint, including OpenRouter for Claude
				</div>
			</div>
			<button
				type="button"
				class="toggle"
				class:on={draft.byoOn}
				role="switch"
				aria-checked={draft.byoOn}
				aria-label={draft.byoOn ? 'Disable BYO provider' : 'Enable BYO provider'}
				onclick={() => onChange({ byoOn: !draft.byoOn })}
			>
				<span class="toggle-track"></span>
			</button>
		</div>
		{#if draft.byoOn}
			<div class="byo-divider"></div>
			<div class="provider-card">
				<!-- Chat and embeddings are separate endpoints: a hosted chat model
				     alongside local embeddings is the common setup. -->
				<div class="section chat">
					<div class="section-title">Chat provider</div>

					<div class="form-group">
						<label class="form-label" for={chatBaseId}>Chat API base URL</label>
						<input
							id={chatBaseId}
							class="form-input"
							type="url"
							value={draft.chatApiBase}
							placeholder="https://api.openai.com/v1"
							oninput={(event) => onChange({ chatApiBase: event.currentTarget.value })}
						/>
					</div>

					<ApiKeyField
						label="Chat API key"
						value={draft.chatApiKey}
						show={draft.showChatApiKey}
						hasStoredKey={Boolean(config?.has_chat_api_key)}
						emptyHint="Required"
						clearLabel="Remove the saved chat key when I save"
						clear={draft.clearChatApiKey}
						onValueChange={(value) =>
							onChange({
								chatApiKey: value,
								clearChatApiKey: value.trim() ? false : draft.clearChatApiKey
							})}
						onToggleShow={() => onChange({ showChatApiKey: !draft.showChatApiKey })}
						onClearChange={(clear) => onChange({ clearChatApiKey: clear })}
					/>

					<div class="form-group">
						<label class="form-label" for={chatModelId}>Chat model ID</label>
						<input
							id={chatModelId}
							class="form-input"
							type="text"
							value={draft.chatModel}
							placeholder="gpt-4.1-mini"
							oninput={(event) => onChange({ chatModel: event.currentTarget.value })}
						/>
						<span class="field-hint">
							Whatever your provider exposes — including OpenRouter model slugs.
						</span>
					</div>

					<CheckRow
						checked={draft.supportsReasoningEffort}
						label="Reasoning model compatibility"
						onChange={(checked) => onChange({ supportsReasoningEffort: checked })}
					>
						{#snippet help()}
							{draft.supportsReasoningEffort ? REASONING_HELP.on : REASONING_HELP.off}
						{/snippet}
					</CheckRow>
				</div>

				<div class="section embed">
					<div class="section-title">Embedding provider</div>

					<div class="form-group">
						<label class="form-label" for={embeddingBaseId}>Embedding API base URL</label>
						<input
							id={embeddingBaseId}
							class="form-input"
							type="url"
							value={draft.embeddingApiBase}
							placeholder="http://host.docker.internal:11434/v1"
							oninput={(event) => onChange({ embeddingApiBase: event.currentTarget.value })}
						/>
					</div>

					<ApiKeyField
						label="Embedding API key"
						value={draft.embeddingApiKey}
						show={draft.showEmbeddingApiKey}
						hasStoredKey={Boolean(config?.has_embedding_api_key)}
						emptyHint="Optional for local providers"
						clearLabel="Remove the saved embedding key when I save"
						clear={draft.clearEmbeddingApiKey}
						onValueChange={(value) =>
							onChange({
								embeddingApiKey: value,
								clearEmbeddingApiKey: value.trim() ? false : draft.clearEmbeddingApiKey
							})}
						onToggleShow={() => onChange({ showEmbeddingApiKey: !draft.showEmbeddingApiKey })}
						onClearChange={(clear) => onChange({ clearEmbeddingApiKey: clear })}
					/>

					<div class="form-group">
						<label class="form-label" for={embeddingModelId}>Embedding model ID</label>
						<input
							id={embeddingModelId}
							class="form-input"
							type="text"
							value={draft.embeddingModel}
							placeholder="nomic-embed-text"
							oninput={(event) => onChange({ embeddingModel: event.currentTarget.value })}
						/>
						<span class="field-hint">
							Must return <strong>{draft.embeddingDim}-dimensional</strong> vectors. Changing the endpoint
							or model rebuilds every embedding when you save.
						</span>
					</div>
				</div>

				<ProviderConnectionCard
					{testState}
					{testMessage}
					disabled={testDisabled}
					onTest={onTestConnection}
				/>

				<ProviderChatBudget
					modelContextWindow={draft.modelContextWindow}
					chatContextPct={draft.chatContextPct}
					onWindowChange={(value) => onChange({ modelContextWindow: value })}
					onPctChange={(value) => onChange({ chatContextPct: value })}
				/>
			</div>
		{/if}
	</div>
</SettingsGroup>

<style>
	/* The shell is the only card; the form and its sections sit flat inside it,
	   matching the prototype's single-card layout. */
	.byo-shell {
		background: var(--bg-elevated);
		border-radius: 16px;
		box-shadow: var(--mila-card-shadow);
		transition: box-shadow 200ms;
	}
	.byo-shell.expanded {
		box-shadow:
			var(--mila-card-shadow),
			0 0 0 1px var(--mila-violet-soft);
	}
	.byo-toggle-row {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 14px 16px;
	}
	.byo-divider {
		height: 0.5px;
		background: var(--border-primary);
		margin: 0 16px;
	}
	.byo-toggle-text {
		flex: 1;
		min-width: 0;
	}
	.byo-toggle-title {
		font-size: 13.5px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
	}
	.byo-toggle-sub {
		font-size: 11.5px;
		letter-spacing: -0.005em;
		color: var(--text-secondary);
	}

	.provider-card {
		padding: 16px;
		display: grid;
		grid-template-columns: 1fr 1fr;
		align-items: start;
		gap: 24px 26px;
	}
	.section {
		position: relative;
		display: flex;
		flex-direction: column;
		gap: 14px;
		min-width: 0;
	}
	/* Column rule: the two endpoint blocks are rarely the same height, and the
	   hairline keeps the shorter one from reading as an unfinished column. */
	.section.chat::after {
		content: '';
		position: absolute;
		top: 2px;
		bottom: 2px;
		right: -13px;
		width: 0.5px;
		background: var(--border-primary);
	}
	.section-title {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 11px;
		font-weight: 700;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--text-secondary);
	}
	.section-title::before {
		content: '';
		width: 2px;
		height: 12px;
		border-radius: 2px;
		flex-shrink: 0;
	}
	.section.chat .section-title::before {
		background: var(--mila-action-chat);
	}
	.section.embed .section-title::before {
		background: var(--mila-action-summary);
	}

	.form-group {
		display: flex;
		flex-direction: column;
		gap: 6px;
		min-width: 0;
	}
	.form-label {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-tertiary);
	}
	.form-input {
		width: 100%;
		box-sizing: border-box;
		padding: 10px 12px;
		border-radius: 10px;
		background: var(--input-bg);
		box-shadow: var(--mila-input-shadow);
		border: 0;
		font-size: 13.5px;
		letter-spacing: -0.005em;
		color: var(--text-primary);
		outline: none;
		transition: box-shadow 150ms;
	}
	.form-input:focus {
		box-shadow:
			inset 0 0 0 1.5px var(--mila-violet),
			0 0 0 4px var(--mila-violet-soft);
	}
	.form-input::placeholder {
		color: var(--text-tertiary);
	}
	.field-hint {
		font-size: 11.5px;
		line-height: 1.4;
		letter-spacing: -0.005em;
		color: var(--text-tertiary);
	}
	.field-hint strong {
		font-weight: 600;
		color: var(--text-secondary);
	}

	.toggle {
		border: 0;
		background: transparent;
		cursor: pointer;
		padding: 0;
	}
	.toggle-track {
		width: 36px;
		height: 21px;
		border-radius: 980px;
		background: var(--mila-status-idle-bg);
		position: relative;
		display: block;
		transition: background 160ms;
	}
	.toggle-track::after {
		content: '';
		position: absolute;
		left: 2px;
		top: 2px;
		width: 17px;
		height: 17px;
		border-radius: 50%;
		background: var(--switch-thumb);
		transition: left 180ms;
	}
	.toggle.on .toggle-track {
		background: var(--mila-violet);
	}
	.toggle.on .toggle-track::after {
		left: 17px;
	}

	@media (max-width: 900px) {
		.provider-card {
			grid-template-columns: 1fr;
			gap: 22px;
		}
		.section.chat::after {
			display: none;
		}
	}
</style>
