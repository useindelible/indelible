<script lang="ts">
	import type { LocalProbe } from './local-provider';

	let {
		endpoint = $bindable(),
		chatModel = $bindable(),
		embeddingModel = $bindable(),
		probe,
		disabled = false
	}: {
		endpoint: string;
		chatModel: string;
		embeddingModel: string;
		probe: LocalProbe | null;
		disabled?: boolean;
	} = $props();
</script>

<div class="local-form">
	<div class="local-form-header">
		<span class="local-form-title">Local server details</span>
		<span class="local-form-meta">OpenAI-compatible</span>
	</div>

	<label class="field">
		<span class="field-label">Server URL</span>
		<input
			type="url"
			class="field-input"
			bind:value={endpoint}
			placeholder="http://host.docker.internal:11434"
			aria-label="OpenAI-compatible server URL"
			{disabled}
		/>
	</label>
	<p class="field-hint">
		A server on the machine hosting Indelible is http://host.docker.internal:11434 from inside
		Docker, never localhost. Reaching it also needs EGRESS_ALLOW_PRIVATE_TARGETS=true in the
		server's environment.
	</p>

	<div class="model-grid">
		<label class="field">
			<span class="field-label">Chat model ID <small>Required</small></span>
			<input
				type="text"
				class="field-input model-input"
				bind:value={chatModel}
				placeholder="qwen3:8b"
				aria-label="Chat model ID"
				{disabled}
			/>
		</label>
		<label class="field">
			<span class="field-label">Embedding model ID <small>768 dimensions</small></span>
			<input
				type="text"
				class="field-input model-input"
				bind:value={embeddingModel}
				placeholder="nomic-embed-text"
				aria-label="Embedding model ID"
				{disabled}
			/>
		</label>
	</div>
	<p class="field-hint">
		Use the exact model IDs exposed by your server. Both models must be loaded before continuing.
	</p>

	{#if probe}
		<div class="probe-results" aria-live="polite">
			<div class="probe-result" class:ok={probe.chatOk} class:failed={!probe.chatOk}>
				<span class="probe-icon">{probe.chatOk ? '✓' : '!'}</span>
				<strong>Chat model</strong>
				<span>{probe.chatMessage}</span>
			</div>
			<div class="probe-result" class:ok={probe.embeddingOk} class:failed={!probe.embeddingOk}>
				<span class="probe-icon">{probe.embeddingOk ? '✓' : '!'}</span>
				<strong>Embeddings</strong>
				<span>{probe.embeddingMessage}</span>
			</div>
		</div>
	{/if}
</div>

<style>
	.local-form {
		margin-top: 16px;
		padding-top: 15px;
		border-top: 1px solid var(--onboarding-ai-section-border);
	}

	.local-form-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		margin-bottom: 13px;
	}

	.local-form-title {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		font-size: 13px;
		font-weight: 680;
		color: var(--text-primary);
	}

	.local-form-title::before {
		content: '';
		width: 7px;
		height: 7px;
		border-radius: 2px;
		background: var(--success);
		box-shadow: 0 0 0 4px var(--fill-success);
	}

	.local-form-meta {
		font-size: 11px;
		color: var(--text-tertiary);
	}

	.field {
		display: block;
	}

	.field-label {
		font-family: var(--font-sans);
		margin-bottom: 6px;
		font-size: 11.5px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-secondary);
		display: flex;
		justify-content: space-between;
		gap: 8px;
	}

	.field-label small {
		font-size: 10.5px;
		font-weight: 450;
		color: var(--text-tertiary);
	}

	.field-hint {
		color: var(--text-tertiary);
		font-size: 10.5px;
		line-height: 1.4;
		margin: 8px 0 0;
	}

	.field-input {
		width: 100%;
		height: 43px;
		padding: 0 13px;
		font-family: var(--font-sans);
		font-size: 12.5px;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		background: var(--onboarding-ai-field-bg);
		border: 1px solid var(--onboarding-ai-field-border);
		border-radius: 9px;
		outline: none;
		transition:
			border-color 0.12s ease,
			box-shadow 0.12s ease;
		box-sizing: border-box;
	}

	.field-input:focus {
		border-color: var(--accent);
		box-shadow: 0 0 0 3px var(--fill-selected-strong);
	}

	.model-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 10px;
		margin-top: 9px;
	}

	.model-input {
		font-family: var(--font-mono);
		font-size: 11.5px;
		letter-spacing: -0.02em;
	}

	.probe-results {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 8px;
		margin-top: 11px;
	}

	.probe-result {
		min-width: 0;
		display: grid;
		grid-template-columns: 22px 1fr;
		align-items: center;
		gap: 2px 9px;
		padding: 9px 11px;
		border: 1px solid var(--border-primary);
		border-radius: var(--radius-md);
	}

	.probe-result.ok {
		background: var(--fill-success);
	}

	.probe-result.failed {
		background: var(--fill-danger);
	}

	.probe-icon {
		grid-row: 1 / span 2;
		width: 22px;
		height: 22px;
		display: grid;
		place-items: center;
		border-radius: 6px;
		background: var(--bg-elevated);
		font-size: 11px;
		font-weight: 700;
	}

	.probe-result.ok .probe-icon {
		color: var(--success);
	}

	.probe-result.failed .probe-icon {
		color: var(--destructive);
	}

	.probe-result strong {
		font-size: 12px;
		line-height: 1.2;
		color: var(--text-primary);
	}

	.probe-result > span:last-child {
		min-width: 0;
		font-size: 10px;
		line-height: 1.3;
		color: var(--text-secondary);
		overflow-wrap: anywhere;
	}

	@media (max-width: 620px) {
		.model-grid,
		.probe-results {
			grid-template-columns: 1fr;
		}
	}
</style>
