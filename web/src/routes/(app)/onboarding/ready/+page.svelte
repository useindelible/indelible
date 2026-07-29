<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { getOnboarding } from '$lib/stores/onboarding.svelte';
	import { getAuth } from '$lib/stores/auth.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import ProgressBar from '$lib/components/onboarding/ProgressBar.svelte';

	const onboarding = getOnboarding();
	const auth = getAuth();

	let submitting = $state(false);

	async function handleGoToLibrary() {
		submitting = true;
		try {
			if (!(await onboarding.completeStep(5))) return;
			if (!(await auth.refresh())) return;
			await goto(resolve('/library'));
		} finally {
			submitting = false;
		}
	}
</script>

<div class="ready">
	<!-- Backdrop matching other onboarding cards -->
	<svg class="backdrop" viewBox="0 0 600 520" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
		<defs>
			<pattern id="ready-dot-grid" x="0" y="0" width="24" height="24" patternUnits="userSpaceOnUse">
				<circle cx="1" cy="1" r="1" fill="#0071E3" fill-opacity="0.07" />
			</pattern>
		</defs>
		<rect width="600" height="520" fill="url(#ready-dot-grid)" />
		<g opacity="0.055" stroke="#0071E3" stroke-width="1" fill="none">
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
		<g opacity="0.055" stroke="#0071E3" stroke-width="1" fill="none">
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

	<div class="success-icon" aria-hidden="true">
		<svg width="40" height="40" viewBox="0 0 40 40" fill="none">
			<path
				d="M11 20l6 6 12-14"
				stroke="#34C759"
				stroke-width="3"
				stroke-linecap="round"
				stroke-linejoin="round"
			/>
		</svg>
	</div>

	<h1 class="ready-title">You're all set!</h1>
	<p class="ready-subtitle">
		Your reading archive is ready. Here are a few tips to get the most out of Indelible.
	</p>

	<ul class="tips-list">
		<li class="tip-row">
			<div class="tip-icon" aria-hidden="true">
				<svg width="18" height="18" viewBox="0 0 24 24" fill="none">
					<rect
						x="2"
						y="6"
						width="20"
						height="12"
						rx="2"
						stroke="currentColor"
						stroke-width="1.5"
					/>
					<path
						d="M6 10h2M10 10h4M16 10h2M8 14h8"
						stroke="currentColor"
						stroke-width="1.5"
						stroke-linecap="round"
					/>
				</svg>
			</div>
			<div class="tip-text">
				<span class="tip-title">Keyboard shortcuts</span>
				<span class="tip-desc">Press ? anywhere to see all shortcuts</span>
			</div>
		</li>
		<li class="tip-row">
			<div class="tip-icon" aria-hidden="true">
				<svg width="18" height="18" viewBox="0 0 24 24" fill="none">
					<path
						d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"
						stroke="currentColor"
						stroke-width="1.5"
						stroke-linecap="round"
						stroke-linejoin="round"
					/>
					<path
						d="M18.5 2.5a2.12 2.12 0 013 3L12 15l-4 1 1-4 9.5-9.5z"
						stroke="currentColor"
						stroke-width="1.5"
						stroke-linecap="round"
						stroke-linejoin="round"
					/>
				</svg>
			</div>
			<div class="tip-text">
				<span class="tip-title">Highlight text</span>
				<span class="tip-desc">Select any text while reading to highlight and annotate</span>
			</div>
		</li>
		<li class="tip-row">
			<div class="tip-icon" aria-hidden="true">
				<svg width="18" height="18" viewBox="0 0 24 24" fill="none">
					<circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.5" />
					<path
						d="M12 6v6l4 2"
						stroke="currentColor"
						stroke-width="1.5"
						stroke-linecap="round"
						stroke-linejoin="round"
					/>
				</svg>
			</div>
			<div class="tip-text">
				<span class="tip-title">Daily review</span>
				<span class="tip-desc">Resurface saved highlights every morning</span>
			</div>
		</li>
		<li class="tip-row">
			<div class="tip-icon" aria-hidden="true">
				<svg width="18" height="18" viewBox="0 0 24 24" fill="none">
					<rect
						x="3"
						y="4"
						width="18"
						height="14"
						rx="2"
						stroke="currentColor"
						stroke-width="1.5"
					/>
					<path d="M3 9h18" stroke="currentColor" stroke-width="1.5" />
					<circle cx="6" cy="6.5" r="1" fill="currentColor" />
					<circle cx="9" cy="6.5" r="1" fill="currentColor" />
				</svg>
			</div>
			<div class="tip-text">
				<span class="tip-title">Save from anywhere</span>
				<span class="tip-desc">Install the browser extension for one-click saving</span>
			</div>
		</li>
	</ul>

	<div class="cta-wrapper">
		{#if onboarding.error || auth.error}
			<p class="submit-error" role="alert">{onboarding.error || auth.error}</p>
		{/if}
		<Button variant="primary" size="xl" fullWidth loading={submitting} onclick={handleGoToLibrary}>
			Go to Library
			{#snippet iconTrailing()}
				<svg width="18" height="18" viewBox="0 0 18 18" fill="none">
					<path
						d="M6 3.5l6 5.5-6 5.5"
						stroke="white"
						stroke-width="1.5"
						stroke-linecap="round"
						stroke-linejoin="round"
					/>
				</svg>
			{/snippet}
		</Button>
	</div>

	<div class="progress-wrapper">
		<ProgressBar currentStep={6} />
	</div>
</div>

<style>
	.ready {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
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

	:global([data-theme='dark']) .ready {
		box-shadow:
			0 1px 3px rgba(0, 0, 0, 0.4),
			0 0 0 1px rgba(255, 255, 255, 0.07);
	}

	.backdrop {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		pointer-events: none;
		user-select: none;
	}

	.success-icon {
		position: relative;
		width: 80px;
		height: 80px;
		border-radius: 50%;
		background: rgba(52, 199, 89, 0.1);
		display: flex;
		align-items: center;
		justify-content: center;
		margin-bottom: 16px;
		flex-shrink: 0;
	}

	:global([data-theme='dark']) .success-icon {
		background: rgba(48, 209, 88, 0.14);
	}

	.ready-title {
		position: relative;
		font-family: var(--font-sans);
		font-size: 28px;
		font-weight: 700;
		letter-spacing: -0.03em;
		line-height: 1.15;
		color: var(--text-primary);
		margin: 0 0 8px;
		text-align: center;
	}

	.ready-subtitle {
		position: relative;
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.5;
		color: var(--text-secondary);
		margin: 0 0 24px;
		text-align: center;
	}

	.tips-list {
		position: relative;
		list-style: none;
		padding: 0;
		margin: 0 0 32px;
		width: 100%;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.tip-row {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 14px 16px;
		border-radius: 10px;
	}

	.tip-icon {
		width: 36px;
		height: 36px;
		border-radius: 10px;
		background: rgba(0, 0, 0, 0.04);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		color: var(--text-primary);
	}

	:global([data-theme='dark']) .tip-icon {
		background: rgba(255, 255, 255, 0.08);
	}

	.tip-text {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.tip-title {
		font-family: var(--font-sans);
		font-size: 14px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
	}

	.tip-desc {
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 400;
		letter-spacing: -0.005em;
		line-height: 1.4;
		color: var(--text-secondary);
	}

	.cta-wrapper {
		position: relative;
		width: 100%;
		max-width: 320px;
	}

	.submit-error {
		margin: 0 0 12px;
		color: var(--color-error, #ff3b30);
		font-size: 14px;
		text-align: center;
	}

	.progress-wrapper {
		position: relative;
		padding-top: 24px;
	}
</style>
