<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import * as apiSdk from '$lib/api';
	import StepLayout from '$lib/components/onboarding/StepLayout.svelte';
	import { getOnboarding } from '$lib/stores/onboarding.svelte';
	import LocalProviderForm from './LocalProviderForm.svelte';
	import { localOnboardingPayload, localOpenAiBase, type LocalProbe } from './local-provider';

	const onboarding = getOnboarding();

	type Provider = 'ollama' | 'openai' | 'skip' | null;

	const providers = [
		{
			id: 'ollama' as const,
			label: 'Local server',
			description: 'Ollama, LM Studio, llama.cpp',
			iconBg: 'var(--fill-success)',
			iconColor: 'var(--success)'
		},
		{
			id: 'openai' as const,
			label: 'OpenAI',
			description: 'Use your API key',
			iconBg: 'var(--fill-selected)',
			iconColor: 'var(--accent)'
		},
		{
			id: 'skip' as const,
			label: 'Skip',
			description: 'Configure later',
			iconBg: 'var(--fill-secondary)',
			iconColor: 'var(--text-tertiary)'
		}
	];

	let selectedProvider = $state<Provider>('ollama');
	let sharedOpenAiKey = $state('');
	let ollamaEndpoint = $state('http://localhost:11434');
	let localChatModel = $state('');
	let localEmbeddingModel = $state('');
	let submitting = $state(false);
	let testError = $state('');
	let localProbe = $state<LocalProbe | null>(null);

	const showSharedOpenAiKey = $derived(selectedProvider === 'openai');
	const showEndpoint = $derived(selectedProvider === 'ollama');
	const continueLabel = $derived(
		selectedProvider === 'skip'
			? 'Continue without AI'
			: localProbe && (!localProbe.chatOk || !localProbe.embeddingOk)
				? 'Try again'
				: 'Continue'
	);

	function selectProvider(id: Provider) {
		selectedProvider = id;
		testError = '';
		localProbe = null;
	}

	async function handleSkip() {
		if (await onboarding.completeStep(4)) goto(resolve('/onboarding/ready'));
	}

	async function handleContinue() {
		submitting = true;
		testError = '';
		try {
			if (!selectedProvider || selectedProvider === 'skip') {
				if (!(await onboarding.completeStep(4))) return;
			} else {
				if (selectedProvider === 'openai' && !sharedOpenAiKey.trim()) {
					testError = 'Enter an OpenAI API key, or choose Skip to configure Mila later.';
					return;
				}
				if (
					selectedProvider === 'ollama' &&
					(!localChatModel.trim() || !localEmbeddingModel.trim())
				) {
					testError = 'Enter both model IDs exposed by your local server.';
					return;
				}
				const testBody = buildTestBody();
				if (testBody) {
					const { data: result } = await apiSdk.testConfig({ body: testBody });
					if (selectedProvider === 'ollama' && result) {
						localProbe = {
							chatOk: result.chat_model_ok,
							embeddingOk: result.embedding_model_ok,
							chatMessage: result.chat_model_ok
								? `Connected to ${localChatModel.trim()}`
								: (result.chat_error ?? 'Chat model connection failed'),
							embeddingMessage: result.embedding_model_ok
								? `Connected, ${result.embedding_dim ?? 768} dimensions`
								: (result.embedding_error ?? 'Embedding model connection failed')
						};
					}
					if (!result?.success) {
						if (selectedProvider === 'openai') {
							testError = result?.error ?? 'Connection test failed. Check your credentials.';
						}
						return;
					}
				}
				const payload =
					selectedProvider === 'ollama'
						? localOnboardingPayload({
								endpoint: ollamaEndpoint,
								chatModel: localChatModel,
								embeddingModel: localEmbeddingModel
							})
						: {
								chat_provider: 'openai',
								embedding_provider: 'openai',
								chat_api_key: sharedOpenAiKey.trim(),
								embedding_api_key: sharedOpenAiKey.trim()
							};
				if (!(await onboarding.completeStep(4, payload))) return;
			}
			goto(resolve('/onboarding/ready'));
		} finally {
			submitting = false;
		}
	}

	function buildTestBody() {
		if (selectedProvider === 'ollama') {
			const base = localOpenAiBase(ollamaEndpoint || 'http://localhost:11434');
			return {
				chat_api_base: base,
				chat_model: localChatModel.trim(),
				embedding_api_base: base,
				embedding_model: localEmbeddingModel.trim(),
				embedding_dim: 768
			};
		}
		if (selectedProvider === 'openai') {
			return {
				chat_api_base: 'https://api.openai.com/v1',
				chat_api_key: sharedOpenAiKey.trim() || undefined,
				chat_model: 'gpt-5.4-mini',
				embedding_api_base: 'https://api.openai.com/v1',
				embedding_api_key: sharedOpenAiKey.trim() || undefined,
				embedding_model: 'text-embedding-3-small',
				embedding_dim: 768
			};
		}
		return null;
	}
</script>

<StepLayout
	title="Supercharge your reading with Mila"
	description="Mila can summarize, auto-tag, and chat about your documents. Connect a provider now or set it up later."
	currentStep={4}
	{continueLabel}
	showSkip
	variant="compact-wide"
	{submitting}
	onContinue={handleContinue}
	onSkip={handleSkip}
>
	<div class="ai-content">
		<p class="section-label">Choose a provider</p>

		<div class="provider-list">
			{#each providers as provider (provider.id)}
				<button
					type="button"
					class="provider-row"
					class:selected={selectedProvider === provider.id}
					onclick={() => selectProvider(provider.id)}
				>
					<div class="provider-icon" style="background: {provider.iconBg};" aria-hidden="true">
						{#if provider.id === 'ollama'}
							<svg
								width="20"
								height="20"
								viewBox="0 0 24 24"
								fill="none"
								stroke={provider.iconColor}
								stroke-width="2"
								stroke-linecap="round"
								stroke-linejoin="round"
							>
								<circle cx="12" cy="12" r="3" />
								<path
									d="M12 1v4M12 19v4M4.22 4.22l2.83 2.83M16.95 16.95l2.83 2.83M1 12h4M19 12h4M4.22 19.78l2.83-2.83M16.95 7.05l2.83-2.83"
								/>
							</svg>
						{:else if provider.id === 'openai'}
							<svg width="20" height="20" viewBox="130 210 300 300" fill={provider.iconColor}>
								<path
									d="M249.176 323.434V298.276C249.176 296.158 249.971 294.569 251.825 293.509L302.406 264.381C309.29 260.409 317.5 258.555 325.973 258.555C357.75 258.555 377.877 283.185 377.877 309.399C377.877 311.253 377.877 313.371 377.611 315.49L325.178 284.771C322.001 282.919 318.822 282.919 315.645 284.771L249.176 323.434ZM367.283 421.415V361.301C367.283 357.592 365.694 354.945 362.516 353.092L296.048 314.43L317.763 301.982C319.617 300.925 321.206 300.925 323.058 301.982L373.639 331.112C388.205 339.586 398.003 357.592 398.003 375.069C398.003 395.195 386.087 413.733 367.283 421.412V421.415ZM233.553 368.452L211.838 355.742C209.986 354.684 209.19 353.095 209.19 350.975V292.718C209.19 264.383 230.905 242.932 260.301 242.932C271.423 242.932 281.748 246.641 290.49 253.26L238.321 283.449C235.146 285.303 233.555 287.951 233.555 291.659V368.455L233.553 368.452ZM280.292 395.462L249.176 377.985V340.913L280.292 323.436L311.407 340.913V377.985L280.292 395.462ZM300.286 475.968C289.163 475.968 278.837 472.259 270.097 465.64L322.264 435.449C325.441 433.597 327.03 430.949 327.03 427.239V350.445L349.011 363.155C350.865 364.213 351.66 365.802 351.66 367.922V426.179C351.66 454.514 329.679 475.965 300.286 475.965V475.968ZM237.525 416.915L186.944 387.785C172.378 379.31 162.582 361.305 162.582 343.827C162.582 323.436 174.763 305.164 193.563 297.485V357.861C193.563 361.571 195.154 364.217 198.33 366.071L264.535 404.467L242.82 416.915C240.967 417.972 239.377 417.972 237.525 416.915ZM234.614 460.343C204.689 460.343 182.71 437.833 182.71 410.028C182.71 407.91 182.976 405.792 183.238 403.672L235.405 433.863C238.582 435.715 241.763 435.715 244.938 433.863L311.407 395.466V420.622C311.407 422.742 310.612 424.331 308.758 425.389L258.179 454.519C251.293 458.491 243.083 460.343 234.611 460.343H234.614ZM300.286 491.854C332.329 491.854 359.073 469.082 365.167 438.892C394.825 431.211 413.892 403.406 413.892 375.073C413.892 356.535 405.948 338.529 391.648 325.552C392.972 319.991 393.766 314.43 393.766 308.87C393.766 271.003 363.048 242.666 327.562 242.666C320.413 242.666 313.528 243.723 306.644 246.109C294.725 234.457 278.307 227.042 260.301 227.042C228.258 227.042 201.513 249.815 195.42 280.004C165.761 287.685 146.694 315.49 146.694 343.824C146.694 362.362 154.638 380.368 168.938 393.344C167.613 398.906 166.819 404.467 166.819 410.027C166.819 447.894 197.538 476.231 233.024 476.231C240.172 476.231 247.058 475.173 253.943 472.788C265.859 484.441 282.278 491.854 300.286 491.854Z"
								/>
							</svg>
						{:else}
							<svg
								width="20"
								height="20"
								viewBox="0 0 24 24"
								fill="none"
								stroke={provider.iconColor}
								stroke-width="2"
								stroke-linecap="round"
							>
								<circle cx="12" cy="12" r="10" />
								<path d="M15 9l-6 6M9 9l6 6" />
							</svg>
						{/if}
					</div>
					<div class="provider-text">
						<span class="provider-name">{provider.label}</span>
						<span class="provider-desc">{provider.description}</span>
					</div>
					<div class="radio" aria-hidden="true">
						{#if selectedProvider === provider.id}
							<div class="radio-dot"></div>
						{/if}
					</div>
				</button>
			{/each}
		</div>

		{#if showEndpoint}
			<LocalProviderForm
				bind:endpoint={ollamaEndpoint}
				bind:chatModel={localChatModel}
				bind:embeddingModel={localEmbeddingModel}
				probe={localProbe}
			/>
		{/if}

		{#if showSharedOpenAiKey}
			<label class="field">
				<span class="field-label">API key</span>
				<input
					type="password"
					class="field-input"
					bind:value={sharedOpenAiKey}
					placeholder="sk-..."
				/>
			</label>
		{/if}

		{#if testError}
			<p class="test-error">{testError}</p>
		{/if}

		<p class="note">AI is optional. All features work without it.</p>
	</div>
</StepLayout>

<style>
	.ai-content {
		display: flex;
		flex-direction: column;
	}

	.section-label {
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-secondary);
		margin: 18px 0 7px;
	}

	.provider-list {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 8px;
	}

	.provider-row {
		min-width: 0;
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 9px 11px;
		border-radius: 10px;
		border: 1px solid var(--border-primary);
		background: var(--onboarding-card-bg);
		cursor: pointer;
		text-align: left;
		width: 100%;
		outline: none;
		transition:
			background 0.12s ease,
			border-color 0.12s ease,
			transform 0.1s ease;
	}

	.provider-row:hover {
		background: var(--fill-hover);
	}

	.provider-row.selected {
		background: var(--fill-selected);
		border-color: var(--accent);
		box-shadow: inset 0 0 0 1px var(--accent);
	}

	.provider-row:active {
		transform: scale(0.985);
	}

	.provider-row:focus-visible {
		box-shadow: inset 0 0 0 1px var(--accent);
	}

	.provider-icon {
		width: 32px;
		height: 32px;
		border-radius: 9px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.provider-text {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.provider-name {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 650;
		letter-spacing: -0.01em;
		color: var(--text-primary);
	}

	.provider-desc {
		font-family: var(--font-sans);
		font-size: 10.5px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--text-secondary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.radio {
		display: none;
	}

	.provider-row.selected .radio {
		border-color: var(--accent);
	}

	.radio-dot {
		width: 10px;
		height: 10px;
		border-radius: 50%;
		background: var(--accent);
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin: 0;
	}

	.field-label {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-secondary);
		display: flex;
		justify-content: space-between;
		gap: 8px;
	}

	.field-input {
		width: 100%;
		height: 43px;
		padding: 0 13px;
		font-family: var(--font-sans);
		font-size: 15px;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		background: var(--fill-success);
		border: 1px solid var(--border-secondary);
		border-radius: var(--radius-md);
		outline: none;
		transition: border-color 0.15s ease;
		box-sizing: border-box;
	}

	.field-input:focus {
		border-color: var(--accent);
	}

	.test-error {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--destructive);
		margin: 0 0 8px;
	}

	.note {
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--text-secondary);
		text-align: center;
		margin: 10px 0 0;
	}

	@media (max-width: 620px) {
		.provider-list {
			grid-template-columns: 1fr;
		}

		.provider-desc {
			white-space: normal;
		}
	}
</style>
