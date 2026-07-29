<script module lang="ts">
	export type IntegrationConnectionCardVariant = 'card' | 'banner';
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';
	import IntegrationStatusPill, {
		type IntegrationStatusPillVariant
	} from './IntegrationStatusPill.svelte';

	interface Props {
		title: string;
		tagline?: string;
		variant?: IntegrationConnectionCardVariant;
		statusLabel?: string;
		statusVariant?: IntegrationStatusPillVariant;
		statusPulse?: boolean;
		statusCheck?: boolean;
		markClass?: string;
		errorMessage?: string | null;
		testId?: string;
		mark?: Snippet;
		body?: Snippet;
		actions?: Snippet;
	}

	let {
		title,
		tagline,
		variant = 'card',
		statusLabel,
		statusVariant,
		statusPulse = false,
		statusCheck = false,
		markClass,
		errorMessage = null,
		testId = 'integration-connection-card',
		mark,
		body,
		actions
	}: Props = $props();

	const markClasses = $derived(['conn-mark', markClass].filter(Boolean).join(' '));
</script>

<article class="connection-card" class:banner={variant === 'banner'} data-testid={testId}>
	<div class="conn-head">
		{#if mark}
			<div class={markClasses}>{@render mark()}</div>
		{/if}
		<div class="conn-meta">
			<div class="conn-name">
				{title}
				{#if statusLabel && statusVariant}
					<IntegrationStatusPill
						variant={statusVariant}
						label={statusLabel}
						pulse={statusPulse}
						check={statusCheck}
					/>
				{/if}
			</div>
			{#if tagline}
				<div class="conn-tagline">{tagline}</div>
			{/if}
		</div>
	</div>

	{#if body}
		<div class="conn-body">{@render body()}</div>
	{/if}

	{#if errorMessage}
		<p class="action-error" role="alert">{errorMessage}</p>
	{/if}

	{#if actions}
		<div class="conn-actions">{@render actions()}</div>
	{/if}
</article>

<style>
	.connection-card {
		background: var(--bg-elevated);
		border-radius: 14px;
		padding: 18px;
		box-shadow: var(--int-shadow-card);
		display: flex;
		flex-direction: column;
		min-height: 168px;
		transition:
			box-shadow 200ms,
			transform 200ms;
		position: relative;
		overflow: hidden;
	}

	.connection-card:hover {
		box-shadow: var(--int-shadow-card-hover);
		transform: translateY(-1px);
	}

	.connection-card.banner {
		min-height: auto;
		padding: 20px 22px;
		display: grid;
		grid-template-columns: minmax(220px, 0.9fr) 2fr;
		column-gap: 28px;
		row-gap: 14px;
		align-items: center;
	}

	.banner .conn-head {
		margin-bottom: 0;
		align-items: center;
	}

	.banner .conn-body {
		flex: none;
		min-width: 0;
		overflow: hidden;
	}

	.conn-head {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		margin-bottom: 12px;
	}

	.conn-mark {
		width: 40px;
		height: 40px;
		border-radius: 10px;
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--bg-secondary);
		color: var(--text-secondary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		font-weight: 700;
		font-size: 16px;
	}

	.conn-mark :global(svg) {
		width: 22px;
		height: 22px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.7;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.conn-mark.email {
		color: var(--integration-mark-email, var(--text-secondary));
	}

	.conn-meta {
		flex: 1;
		min-width: 0;
	}

	.conn-name {
		font-size: 14px;
		font-weight: 600;
		letter-spacing: -0.015em;
		color: var(--text-primary);
		margin-bottom: 3px;
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.conn-tagline {
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.4;
	}

	.conn-body {
		flex: 1;
		min-width: 0;
	}

	.conn-actions {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
		margin-top: auto;
	}

	.action-error {
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--destructive);
		margin: 0;
	}

	@media (max-width: 899px) {
		.connection-card.banner {
			grid-template-columns: 1fr;
			row-gap: 14px;
		}
	}
</style>
