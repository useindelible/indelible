<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import type { MilaConfigResponse } from '$lib/api';
	import type { MilaConfigDraft, TestState } from '../mila-settings-model';

	interface Props {
		config: MilaConfigResponse | null;
		draft: MilaConfigDraft;
		testMessage: string;
		testState: TestState;
		onChange: (patch: Partial<MilaConfigDraft>) => void;
		onTestConnection: () => void;
	}

	let { config, draft, testMessage, testState, onChange, onTestConnection }: Props = $props();
</script>

<SettingsGroup
	title="Provider"
	meta={draft.byoOn
		? 'BYO endpoint · OpenAI-compatible'
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
				<div class="provider-section">
					<div class="provider-section-title">Chat provider</div>
					<div class="form-group">
						<label class="form-label" for="chat-api-base">Chat API Base URL</label>
						<input
							id="chat-api-base"
							class="form-input"
							type="url"
							value={draft.chatApiBase}
							placeholder="https://api.openai.com/v1"
							oninput={(event) => onChange({ chatApiBase: event.currentTarget.value })}
						/>
					</div>

					<div class="form-group">
						<label class="form-label" for="chat-api-key">
							Chat API Key
							<span class="hint">
								{config?.has_chat_api_key ? 'Already configured · enter to replace' : 'Required'}
							</span>
						</label>
						<div class="password-input-wrap">
							<input
								id="chat-api-key"
								class="form-input"
								type={draft.showChatApiKey ? 'text' : 'password'}
								value={draft.chatApiKey}
								placeholder="sk-..."
								oninput={(event) => onChange({ chatApiKey: event.currentTarget.value })}
							/>
							<button
								type="button"
								class="eye-btn"
								aria-label={draft.showChatApiKey ? 'Hide chat API key' : 'Show chat API key'}
								onclick={() => onChange({ showChatApiKey: !draft.showChatApiKey })}
							>
								{draft.showChatApiKey ? 'Hide' : 'Show'}
							</button>
						</div>
						{#if config?.has_chat_api_key && !draft.chatApiKey.trim()}
							<label class="checkbox-row">
								<input
									type="checkbox"
									checked={draft.clearChatApiKey}
									onchange={(event) => onChange({ clearChatApiKey: event.currentTarget.checked })}
								/>
								<span class="hint-inline">Remove saved chat key on next save</span>
							</label>
						{/if}
					</div>

					<div class="form-group">
						<label class="form-label" for="model-id">Chat model ID</label>
						<input
							id="model-id"
							class="form-input"
							type="text"
							value={draft.chatModel}
							placeholder="gpt-4.1-mini"
							oninput={(event) => onChange({ chatModel: event.currentTarget.value })}
						/>
					</div>

					<label class="checkbox-row">
						<input
							type="checkbox"
							checked={draft.supportsReasoningEffort}
							onchange={(event) =>
								onChange({ supportsReasoningEffort: event.currentTarget.checked })}
						/>
						<span class="hint-inline"> Provider supports reasoning_effort (LM Studio 0.4.8+) </span>
					</label>
				</div>

				<div class="provider-section">
					<div class="provider-section-title">Embedding provider</div>
					<div class="form-group">
						<label class="form-label" for="embedding-api-base">Embedding API Base URL</label>
						<input
							id="embedding-api-base"
							class="form-input"
							type="url"
							value={draft.embeddingApiBase}
							placeholder="http://localhost:11434/v1"
							oninput={(event) => onChange({ embeddingApiBase: event.currentTarget.value })}
						/>
					</div>

					<div class="form-group">
						<label class="form-label" for="embedding-api-key">
							Embedding API Key
							<span class="hint">
								{config?.has_embedding_api_key
									? 'Already configured · enter to replace'
									: 'Optional for local providers'}
							</span>
						</label>
						<div class="password-input-wrap">
							<input
								id="embedding-api-key"
								class="form-input"
								type={draft.showEmbeddingApiKey ? 'text' : 'password'}
								value={draft.embeddingApiKey}
								placeholder="sk-..."
								oninput={(event) => onChange({ embeddingApiKey: event.currentTarget.value })}
							/>
							<button
								type="button"
								class="eye-btn"
								aria-label={draft.showEmbeddingApiKey
									? 'Hide embedding API key'
									: 'Show embedding API key'}
								onclick={() => onChange({ showEmbeddingApiKey: !draft.showEmbeddingApiKey })}
							>
								{draft.showEmbeddingApiKey ? 'Hide' : 'Show'}
							</button>
						</div>
						{#if config?.has_embedding_api_key && !draft.embeddingApiKey.trim()}
							<label class="checkbox-row">
								<input
									type="checkbox"
									checked={draft.clearEmbeddingApiKey}
									onchange={(event) =>
										onChange({ clearEmbeddingApiKey: event.currentTarget.checked })}
								/>
								<span class="hint-inline">Remove saved embedding key on next save</span>
							</label>
						{/if}
					</div>

					<div class="form-group">
						<label class="form-label" for="embedding-model-id">Embedding model ID</label>
						<input
							id="embedding-model-id"
							class="form-input"
							type="text"
							value={draft.embeddingModel}
							placeholder="nomic-embed-text"
							oninput={(event) => onChange({ embeddingModel: event.currentTarget.value })}
						/>
						<span class="hint"
							>Must return 768-dimensional vectors. Changing endpoint or model triggers a Mila
							reindex when saved.</span
						>
					</div>
				</div>

				<div class="test-card">
					<div class="test-card-label">Connection</div>
					<div class="test-state-row" data-state={testState}>
						<span class="test-state-msg">{testMessage}</span>
					</div>
					<div class="test-meta">
						We send a small embedding probe to the embedding provider and a short chat completion to
						the chat provider.
					</div>
					<button
						type="button"
						class="btn violet-soft"
						onclick={onTestConnection}
						disabled={testState === 'testing' ||
							!draft.chatApiBase.trim() ||
							!draft.embeddingApiBase.trim()}
					>
						Test connection
					</button>
				</div>

				<div class="provider-section tuning-section">
					<div class="provider-section-title">Chat budget</div>
					<div class="form-group">
						<label class="form-label" for="model-ctx-window">Model context window</label>
						<input
							id="model-ctx-window"
							class="form-input"
							type="number"
							min="1"
							required
							value={draft.modelContextWindow}
							placeholder="16000"
							oninput={(event) =>
								onChange({ modelContextWindow: Number(event.currentTarget.value) })}
						/>
						<span class="hint"
							>Total token window of your chat model. Sizes summary, tag, and entity inputs.</span
						>
					</div>

					<div class="form-group">
						<label class="form-label" for="chat-ctx-pct">Chat inline context (% of window)</label>
						<input
							id="chat-ctx-pct"
							class="form-input"
							type="number"
							min="1"
							max="100"
							value={draft.chatContextPct}
							oninput={(event) => onChange({ chatContextPct: Number(event.currentTarget.value) })}
						/>
						<span class="hint"
							>Sent inline to chat until the document fills this % of the window; larger items use
							retrieval.</span
						>
					</div>
				</div>
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
		box-shadow: var(--shadow-1);
		transition: box-shadow 200ms;
	}
	.byo-shell.expanded {
		box-shadow:
			var(--shadow-1),
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
		color: var(--text-primary);
	}
	.byo-toggle-sub,
	.hint,
	.hint-inline,
	.test-meta {
		font-size: 11.5px;
		color: var(--text-secondary);
	}
	.provider-card {
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 22px;
	}
	.provider-section {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.provider-section-title {
		font-size: 12px;
		font-weight: 700;
		color: var(--text-primary);
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
		letter-spacing: 0;
		text-transform: uppercase;
		color: var(--text-tertiary);
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
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
		color: var(--text-primary);
		outline: none;
	}
	.password-input-wrap {
		position: relative;
		display: flex;
		align-items: center;
	}
	.password-input-wrap .form-input {
		padding-right: 56px;
	}
	.eye-btn {
		position: absolute;
		right: 6px;
		border: 0;
		border-radius: 8px;
		background: transparent;
		color: var(--text-tertiary);
		font-size: 11px;
		cursor: pointer;
		padding: 6px;
	}
	.checkbox-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	/* The lone inset callout, matching the prototype's test card. */
	.test-card {
		background: var(--bg-secondary);
		border-radius: 12px;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.test-card-label {
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0;
		text-transform: uppercase;
		color: var(--text-tertiary);
	}
	.test-state-row {
		padding: 9px 11px;
		border-radius: 10px;
		font-size: 12.5px;
		background: var(--mila-status-idle-bg);
		color: var(--mila-status-idle-text);
	}
	.test-state-row[data-state='success'] {
		background: var(--mila-status-ok-bg);
		color: var(--mila-status-ok-text);
	}
	.test-state-row[data-state='error'] {
		background: var(--mila-status-err-bg);
		color: var(--mila-status-err-text);
	}
	.test-state-row[data-state='testing'] {
		background: var(--mila-status-test-bg);
		color: var(--mila-status-test-text);
	}
	.btn {
		border: 0;
		border-radius: 9px;
		font-size: 12.5px;
		font-weight: 500;
		cursor: pointer;
		padding: 7px 13px;
	}
	.btn.violet-soft {
		background: var(--mila-violet-soft);
		color: var(--mila-violet);
	}
	.btn:disabled {
		opacity: 0.45;
		cursor: default;
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
	}
	.toggle-track::after {
		content: '';
		position: absolute;
		left: 2px;
		top: 2px;
		width: 17px;
		height: 17px;
		border-radius: 50%;
		background: var(--bg-primary);
	}
	.toggle.on .toggle-track {
		background: var(--mila-violet);
	}
	.toggle.on .toggle-track::after {
		left: 17px;
	}
</style>
