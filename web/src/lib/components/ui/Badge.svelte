<script lang="ts">
	import type { Snippet } from 'svelte';

	type Variant = 'default' | 'accent' | 'success' | 'destructive' | 'warning';
	type Size = 'sm' | 'md';

	interface Props {
		variant?: Variant;
		size?: Size;
		children: Snippet;
	}

	let { variant = 'default', size = 'sm', children }: Props = $props();
</script>

<span class="badge badge-{variant} badge-{size}">
	{@render children()}
</span>

<style>
	.badge {
		display: inline-flex;
		align-items: center;
		font-family: var(--font-sans);
		font-weight: 500;
		letter-spacing: -0.005em;
		border-radius: 6px;
		white-space: nowrap;
		line-height: 1;
	}

	.badge-sm {
		font-size: 11px;
		padding: 2px 6px;
		min-height: 18px;
	}

	.badge-md {
		font-size: 13px;
		padding: 3px 8px;
		min-height: 22px;
	}

	.badge-default {
		background: var(--fill-hover);
		border: 1px solid var(--border-primary);
		color: var(--text-secondary);
	}

	.badge-accent {
		background: var(--fill-selected);
		color: var(--accent);
	}

	.badge-success {
		background: var(--fill-success);
		color: var(--success);
	}

	.badge-destructive {
		background: var(--fill-danger);
		color: var(--destructive);
	}

	.badge-warning {
		background: var(--fill-warning);
		color: var(--warning);
	}
</style>
