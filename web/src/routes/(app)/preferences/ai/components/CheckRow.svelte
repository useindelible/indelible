<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		checked: boolean;
		label: string;
		subtle?: boolean;
		onChange: (checked: boolean) => void;
		help?: Snippet;
	}

	let { checked, label, subtle = false, onChange, help }: Props = $props();

	const helpId = $props.id();
</script>

<label class="check-row" class:subtle>
	<!-- The label element wraps the help text too, so the accessible name is
	     pinned to the label itself rather than the whole block. -->
	<input
		type="checkbox"
		{checked}
		aria-label={label}
		aria-describedby={help ? helpId : undefined}
		onchange={(event) => onChange(event.currentTarget.checked)}
	/>
	<span class="check-box" aria-hidden="true">
		<svg viewBox="0 0 24 24"><path d="M4 12.5l5 5L20 7" /></svg>
	</span>
	<span class="check-text">
		<span class="check-label">{label}</span>
		{#if help}<span class="check-help" id={helpId}>{@render help()}</span>{/if}
	</span>
</label>

<style>
	.check-row {
		display: flex;
		align-items: flex-start;
		gap: 9px;
		cursor: pointer;
		-webkit-tap-highlight-color: transparent;
	}
	.check-row input {
		position: absolute;
		opacity: 0;
		pointer-events: none;
	}
	.check-box {
		width: 16px;
		height: 16px;
		border-radius: 5px;
		flex-shrink: 0;
		margin-top: 1px;
		background: var(--input-bg);
		box-shadow: inset 0 0 0 1px var(--border-primary);
		display: flex;
		align-items: center;
		justify-content: center;
		color: transparent;
		transition:
			background 140ms,
			box-shadow 140ms,
			color 140ms;
	}
	.check-box svg {
		width: 10px;
		height: 10px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2.6;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
	.check-row input:checked + .check-box {
		background: var(--mila-violet);
		box-shadow: inset 0 0 0 1px var(--mila-violet);
		color: var(--text-on-color);
	}
	.check-row input:focus-visible + .check-box {
		box-shadow:
			inset 0 0 0 1px var(--mila-violet),
			0 0 0 4px var(--mila-violet-soft);
	}
	.check-text {
		display: flex;
		flex-direction: column;
		gap: 3px;
		min-width: 0;
	}
	.check-label {
		font-size: 12.5px;
		color: var(--text-primary);
		letter-spacing: -0.005em;
	}
	.check-row.subtle .check-label {
		font-size: 12px;
		color: var(--text-secondary);
	}
	.check-help {
		font-size: 11.5px;
		color: var(--text-tertiary);
		letter-spacing: -0.005em;
		line-height: 1.45;
	}
</style>
