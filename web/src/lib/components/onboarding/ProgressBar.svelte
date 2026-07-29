<script lang="ts">
	import { ONBOARDING_STEPS } from '$lib/stores/onboarding.svelte';

	let { currentStep }: { currentStep: number } = $props();
</script>

<nav class="progress-bar" aria-label="Onboarding progress">
	<ol>
		{#each ONBOARDING_STEPS as step, i (step.path)}
			<li
				class="progress-dot"
				class:completed={i < currentStep}
				class:current={i === currentStep}
				aria-current={i === currentStep ? 'step' : undefined}
			>
				<span class="sr-only"
					>Step {i + 1}: {step.label}{i < currentStep
						? ' (completed)'
						: i === currentStep
							? ' (current)'
							: ''}</span
				>
			</li>
		{/each}
	</ol>
</nav>

<style>
	.progress-bar {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
	}

	ol {
		display: flex;
		gap: 7px;
		list-style: none;
		padding: 0;
		margin: 0;
	}

	.progress-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--border-secondary);
		transition:
			background 0.2s ease,
			opacity 0.2s ease,
			width 0.2s ease;
	}

	.progress-dot.completed {
		background: var(--accent);
		opacity: 0.4;
	}

	.progress-dot.current {
		width: 20px;
		border-radius: var(--radius-full);
		background: var(--accent);
		opacity: 1;
	}

	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border-width: 0;
	}
</style>
