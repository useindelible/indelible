<script lang="ts">
	import type { Snippet } from 'svelte';

	type Variant =
		| 'primary'
		| 'secondary'
		| 'tertiary'
		| 'destructive'
		| 'destructive-outline'
		| 'destructive-text'
		| 'ghost'
		| 'pill';
	type Size = 'sm' | 'md' | 'lg' | 'xl';

	interface Props {
		variant?: Variant;
		size?: Size;
		type?: 'button' | 'submit' | 'reset';
		href?: string;
		loading?: boolean;
		disabled?: boolean;
		fullWidth?: boolean;
		onclick?: (e: MouseEvent) => void;
		children: Snippet;
		icon?: Snippet;
		iconTrailing?: Snippet;
	}

	let {
		variant = 'primary',
		size = 'md',
		type = 'button',
		href,
		loading = false,
		disabled = false,
		fullWidth = false,
		onclick,
		children,
		icon,
		iconTrailing
	}: Props = $props();
</script>

{#if href}
	<!-- eslint-disable svelte/no-navigation-without-resolve -- generic component, callers must pass resolved hrefs -->
	<a {href} class="btn btn-{variant} btn-{size}" class:btn-full={fullWidth}>
		{#if icon}
			<span class="btn-icon" aria-hidden="true">{@render icon()}</span>
		{/if}
		{@render children()}
		{#if iconTrailing}
			<span class="btn-icon" aria-hidden="true">{@render iconTrailing()}</span>
		{/if}
	</a>
	<!-- eslint-enable svelte/no-navigation-without-resolve -->
{:else}
	<button
		{type}
		{onclick}
		class="btn btn-{variant} btn-{size}"
		class:btn-full={fullWidth}
		class:is-loading={loading}
		disabled={loading || disabled}
		aria-busy={loading}
	>
		{#if loading}
			<span class="btn-spinner" aria-hidden="true"></span>
		{/if}
		{#if icon && !loading}
			<span class="btn-icon" aria-hidden="true">{@render icon()}</span>
		{/if}
		<span class:sr-only={loading}>
			{@render children()}
		</span>
		{#if iconTrailing && !loading}
			<span class="btn-icon" aria-hidden="true">{@render iconTrailing()}</span>
		{/if}
	</button>
{/if}

<style>
	.btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		font-family: var(--font-sans);
		font-weight: 500;
		letter-spacing: -0.01em;
		line-height: 1;
		white-space: nowrap;
		cursor: pointer;
		border: none;
		text-decoration: none;
		transition:
			background 120ms ease,
			opacity 120ms ease,
			transform 100ms ease,
			border-color 120ms ease;
		position: relative;
		flex-shrink: 0;
	}

	/* Sizes */
	.btn-sm {
		padding: 5px 12px;
		font-size: 12px;
		border-radius: 6px;
		min-height: 28px;
	}

	.btn-md {
		padding: 8px 20px;
		font-size: 13px;
		border-radius: 8px;
		min-height: 34px;
	}

	.btn-lg {
		padding: 10px 24px;
		font-size: 15px;
		border-radius: 10px;
		min-height: 40px;
	}

	.btn-xl {
		padding: 12px 24px;
		font-size: 17px;
		border-radius: var(--radius-sm);
		min-height: 48px;
	}

	/* Variants */
	.btn-primary {
		background: var(--accent);
		color: var(--text-on-color);
	}
	.btn-primary:hover:not(:disabled) {
		background: var(--accent-hover);
	}
	.btn-primary:active:not(:disabled) {
		transform: scale(0.98);
	}

	.btn-secondary {
		background: var(--bg-elevated);
		color: var(--text-primary);
		border: 1px solid var(--border-primary);
	}
	.btn-secondary:hover:not(:disabled) {
		background: var(--fill-hover);
		border-color: var(--border-secondary);
	}

	.btn-tertiary {
		background: rgba(0, 0, 0, 0.04);
		color: var(--text-primary);
	}
	:global([data-theme='dark']) .btn-tertiary {
		background: rgba(255, 255, 255, 0.08);
	}
	.btn-tertiary:hover:not(:disabled) {
		opacity: 0.8;
	}

	.btn-destructive {
		background: var(--destructive);
		color: var(--text-on-color);
	}
	.btn-destructive:hover:not(:disabled) {
		opacity: 0.9;
	}
	.btn-destructive:active:not(:disabled) {
		transform: scale(0.98);
	}

	.btn-destructive-outline {
		background: transparent;
		color: var(--destructive);
		border: 1px solid var(--destructive);
	}
	.btn-destructive-outline:hover:not(:disabled) {
		background: var(--fill-danger);
	}

	.btn-destructive-text {
		background: none;
		color: var(--destructive);
	}
	.btn-destructive-text:hover:not(:disabled) {
		text-decoration: underline;
	}

	.btn-ghost {
		background: none;
		color: var(--accent);
	}
	.btn-ghost:hover:not(:disabled) {
		text-decoration: underline;
	}

	.btn-pill {
		background: var(--bg-elevated);
		color: var(--text-primary);
		border: 1px solid var(--border-primary);
		border-radius: 980px;
		padding: 9px 16px;
	}
	.btn-pill:hover:not(:disabled) {
		border-color: var(--border-secondary);
		background: var(--fill-hover);
	}
	.btn-pill:active:not(:disabled) {
		transform: scale(0.98);
	}

	/* State modifiers */
	.btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.is-loading {
		cursor: wait;
	}

	.btn-full {
		width: 100%;
	}

	/* Spinner */
	.btn-spinner {
		display: inline-block;
		width: 14px;
		height: 14px;
		border: 2px solid rgba(255, 255, 255, 0.3);
		border-top-color: var(--text-on-color);
		border-radius: 50%;
		animation: btn-spin 0.6s linear infinite;
		flex-shrink: 0;
	}

	.btn-icon {
		display: inline-flex;
		align-items: center;
		flex-shrink: 0;
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
		border: 0;
	}

	@keyframes btn-spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
