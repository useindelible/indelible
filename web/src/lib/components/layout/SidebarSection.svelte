<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		label: string;
		defaultExpanded?: boolean;
		children: Snippet;
	}

	let { label, defaultExpanded = true, children }: Props = $props();
	let expanded = $state(defaultExpanded);

	function toggle() {
		expanded = !expanded;
	}
</script>

<div class="sidebar-section">
	<button type="button" class="section-header" aria-expanded={expanded} onclick={toggle}>
		<span class="section-label">{label}</span>
		<span class="section-chevron" class:collapsed={!expanded} aria-hidden="true">
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<polyline points="6 9 12 15 18 9" />
			</svg>
		</span>
	</button>
	{#if expanded}
		<div class="section-content">
			{@render children()}
		</div>
	{/if}
</div>

<style>
	.sidebar-section {
		display: flex;
		flex-direction: column;
	}

	.section-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 12px 6px;
		background: none;
		border: none;
		cursor: pointer;
		width: 100%;
		text-align: left;
	}

	.section-header:hover .section-chevron {
		opacity: 1;
	}

	.section-label {
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		line-height: 1.2;
	}

	.section-chevron {
		width: 14px;
		height: 14px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-tertiary);
		opacity: 0;
		transition:
			opacity 0.15s ease,
			transform 0.15s ease;
	}

	.section-chevron.collapsed {
		transform: rotate(-90deg);
	}

	.section-chevron svg {
		width: 12px;
		height: 12px;
	}

	.section-content {
		display: flex;
		flex-direction: column;
		gap: 1px;
	}
</style>
