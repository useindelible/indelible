<script module lang="ts">
	export type IntegrationStatusPillVariant = 'active' | 'syncing' | 'attention' | 'coming';
</script>

<script lang="ts">
	interface Props {
		variant: IntegrationStatusPillVariant;
		label: string;
		pulse?: boolean;
		check?: boolean;
	}

	let { variant, label, pulse = false, check = false }: Props = $props();
</script>

<span class="status-pill {variant}" data-testid="integration-status-pill" data-variant={variant}>
	{#if pulse}
		<span class="pulse-dot"></span>
	{:else if check}
		<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12l4 4 10-10" /></svg>
	{/if}
	{label}
</span>

<style>
	.status-pill {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		border-radius: 980px;
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0.005em;
		flex-shrink: 0;
	}

	.status-pill svg {
		width: 9px;
		height: 9px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.status-pill.active {
		background: var(--integration-status-active-bg, var(--int-status-active-bg));
		color: var(--integration-status-active-text, var(--int-status-active-text));
	}

	.status-pill.syncing {
		background: var(--integration-status-syncing-bg, var(--int-status-syncing-bg));
		color: var(--integration-status-syncing-text, var(--int-status-syncing-text));
	}

	.status-pill.attention {
		background: var(--integration-status-attention-bg, var(--int-status-attention-bg));
		color: var(--integration-status-attention-text, var(--int-status-attention-text));
	}

	.status-pill.coming {
		background: var(--integration-status-coming-bg, var(--int-status-coming-bg));
		color: var(--integration-status-coming-text, var(--int-status-coming-text));
	}

	.pulse-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: currentColor;
		animation: pulse-dot 1.6s ease-in-out infinite;
	}

	@keyframes pulse-dot {
		0%,
		100% {
			opacity: 1;
			transform: scale(1);
		}
		50% {
			opacity: 0.45;
			transform: scale(0.85);
		}
	}
</style>
