<script lang="ts">
	import type { Snippet } from 'svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import ProgressBar from '$lib/components/onboarding/ProgressBar.svelte';

	let {
		title,
		description = '',
		continueLabel = 'Continue',
		showSkip = false,
		showContinue = true,
		variant = 'default',
		submitting = false,
		currentStep = 0,
		onContinue,
		onSkip,
		children
	}: {
		title: string;
		description?: string;
		continueLabel?: string;
		showSkip?: boolean;
		showContinue?: boolean;
		variant?: 'default' | 'compact-wide';
		submitting?: boolean;
		currentStep?: number;
		onContinue?: () => void;
		onSkip?: () => void;
		children: Snippet;
	} = $props();
</script>

<div class="step-card" class:compact-wide={variant === 'compact-wide'}>
	<svg class="backdrop" viewBox="0 0 600 520" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
		<defs>
			<pattern id="step-dot-grid" x="0" y="0" width="24" height="24" patternUnits="userSpaceOnUse">
				<circle cx="1" cy="1" r="1" fill="var(--accent)" fill-opacity="0.07" />
			</pattern>
		</defs>
		<rect width="600" height="520" fill="url(#step-dot-grid)" />
		<!-- Article card outlines — top-left -->
		<g opacity="0.055" stroke="var(--accent)" stroke-width="1" fill="none">
			<rect x="28" y="32" width="110" height="148" rx="6" transform="rotate(-6 83 106)" />
			<rect x="48" y="44" width="110" height="148" rx="6" transform="rotate(-3 103 118)" />
			<line x1="58" y1="80" x2="140" y2="80" stroke-linecap="round" transform="rotate(-3 99 80)" />
			<line x1="58" y1="92" x2="130" y2="92" stroke-linecap="round" transform="rotate(-3 94 92)" />
			<line
				x1="58"
				y1="104"
				x2="120"
				y2="104"
				stroke-linecap="round"
				transform="rotate(-3 89 104)"
			/>
		</g>
		<!-- Article card outlines — top-right -->
		<g opacity="0.055" stroke="var(--accent)" stroke-width="1" fill="none">
			<rect x="462" y="28" width="110" height="148" rx="6" transform="rotate(5 517 102)" />
			<rect x="444" y="42" width="110" height="148" rx="6" transform="rotate(2 499 116)" />
			<line x1="454" y1="82" x2="536" y2="82" stroke-linecap="round" transform="rotate(2 495 82)" />
			<line x1="454" y1="94" x2="526" y2="94" stroke-linecap="round" transform="rotate(2 490 94)" />
			<line
				x1="454"
				y1="106"
				x2="516"
				y2="106"
				stroke-linecap="round"
				transform="rotate(2 485 106)"
			/>
		</g>
	</svg>

	<div class="step-header">
		<h1 class="step-title">{title}</h1>
		{#if description}
			<p class="step-description">{description}</p>
		{/if}
	</div>

	<div class="step-content">
		{@render children()}
	</div>

	<div class="step-footer">
		{#if showContinue}
			<Button
				variant="primary"
				size={variant === 'compact-wide' ? 'lg' : 'xl'}
				loading={submitting}
				onclick={onContinue}
			>
				{continueLabel}
			</Button>
		{/if}
		{#if showSkip}
			<button class="btn-secondary" disabled={submitting} onclick={onSkip}> Skip </button>
		{/if}
	</div>

	<div class="step-progress">
		<ProgressBar {currentStep} />
	</div>
</div>

<style>
	.step-card {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 32px;
		width: 100%;
		max-width: 600px;
		border-radius: 14px;
		padding: 40px;
		overflow: hidden;
		background: var(--bg-elevated);
		box-shadow:
			0 1px 3px rgba(0, 0, 0, 0.08),
			0 0 0 1px rgba(0, 0, 0, 0.07);
	}

	.backdrop {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		pointer-events: none;
		user-select: none;
	}

	:global([data-theme='dark']) .backdrop circle {
		fill-opacity: 0.1;
	}

	:global([data-theme='dark']) .step-card {
		background: var(--bg-elevated);
	}

	.step-header {
		position: relative;
		text-align: center;
	}

	.step-title {
		font-family: var(--font-sans);
		font-size: 22px;
		font-weight: 700;
		letter-spacing: -0.03em;
		color: var(--text-primary);
		margin: 0;
		line-height: 1.2;
	}

	.step-description {
		font-family: var(--font-sans);
		font-size: 15px;
		letter-spacing: -0.01em;
		color: var(--text-secondary);
		margin: 8px 0 0;
		line-height: 1.5;
	}

	.step-content {
		position: relative;
		width: 100%;
	}

	.step-footer {
		position: relative;
		display: flex;
		flex-direction: row;
		align-items: center;
		justify-content: center;
		gap: 12px;
		width: 100%;
		margin: 0 auto;
	}

	.btn-secondary {
		padding: 8px 16px;
		background: none;
		color: var(--text-secondary);
		border: none;
		font-family: var(--font-sans);
		font-size: 15px;
		text-align: center;
		cursor: pointer;
		transition: color 0.15s ease;
	}

	.btn-secondary:hover:not(:disabled) {
		color: var(--text-primary);
	}

	.btn-secondary:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.step-progress {
		position: relative;
		padding-top: 8px;
	}

	.step-card.compact-wide {
		max-width: 680px;
		gap: 0;
		padding: 28px 40px 22px;
		border: 1px solid var(--border-primary);
		border-radius: 16px;
		background: var(--onboarding-card-bg);
		box-shadow: var(--shadow-3);
	}

	.compact-wide .step-title {
		font-size: 26px;
		line-height: 1.15;
		letter-spacing: -0.04em;
	}

	.compact-wide .step-description {
		max-width: 520px;
		margin-top: 7px;
		font-size: 14px;
		line-height: 1.48;
	}

	.compact-wide .step-footer {
		margin-top: 12px;
		gap: 10px;
	}

	.compact-wide .btn-secondary {
		min-height: 40px;
		padding: 0 10px;
		font-size: 13px;
	}

	.compact-wide :global(.btn-primary) {
		min-height: 40px;
		padding: 0 22px;
		border-radius: 9px;
		font-size: 13px;
		font-weight: 650;
	}

	.compact-wide .backdrop g {
		display: none;
	}

	.compact-wide .step-progress {
		margin-top: 14px;
		padding-top: 0;
	}

	@media (max-width: 720px) {
		.step-card.compact-wide {
			padding: 28px 20px 24px;
		}
	}
</style>
