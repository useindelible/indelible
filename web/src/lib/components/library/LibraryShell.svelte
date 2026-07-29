<script lang="ts">
	import { onMount, type Snippet } from 'svelte';
	import { afterNavigate } from '$app/navigation';
	import { getLibrary } from '$lib/stores/library.svelte';
	import { getViewport } from '$lib/stores/viewport.svelte';

	interface Props {
		sidebar: Snippet;
		content: Snippet;
	}

	let { sidebar, content }: Props = $props();
	const lib = getLibrary();
	const vp = getViewport();

	onMount(() => {
		lib.loadPreferences();
	});

	afterNavigate(() => {
		vp.closeMobileNav();
	});

	function handleDrawerKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' && vp.mobileNavOpen) {
			e.preventDefault();
			vp.closeMobileNav();
		}
	}

	const breakpoint = $derived(vp.breakpoint);
	const effectiveMode = $derived(lib.sidebarSessionOverride ?? lib.sidebarMode);

	const sidebarVisible = $derived.by(() => {
		if (breakpoint === 'xs') return false;
		if (effectiveMode === 'collapsed') return false;
		if (effectiveMode === 'auto') return breakpoint === 'desktop';
		return true;
	});
</script>

<svelte:window onkeydown={vp.mobileNavOpen ? handleDrawerKeydown : undefined} />

<div class="library-shell">
	<!-- Vibrancy blobs positioned behind all panels -->
	<div class="blobs" aria-hidden="true">
		<div class="blob blob-1"></div>
		<div class="blob blob-2"></div>
		<div class="blob blob-3"></div>
		<div class="blob blob-4"></div>
	</div>

	{#if breakpoint !== 'xs'}
		<div class="sidebar-slot" class:is-collapsed={!sidebarVisible}>
			<div class="sidebar-full">
				{@render sidebar()}
			</div>
			<button
				type="button"
				class="sidebar-reveal-btn"
				onclick={() => lib.toggleSidebarVisibility(sidebarVisible)}
				aria-label="Show sidebar"
				title="Show sidebar"
			>
				<svg
					viewBox="0 0 20 20"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
					stroke-linejoin="round"
					aria-hidden="true"
				>
					<rect x="3" y="4" width="14" height="12" rx="1.5" />
					<line x1="8" y1="4" x2="8" y2="16" />
				</svg>
			</button>
		</div>
	{/if}

	<div class="content-slot">
		{@render content()}
	</div>

	{#if breakpoint === 'xs' && vp.mobileNavOpen}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="drawer-scrim" onclick={() => vp.closeMobileNav()}></div>
		<div class="drawer">
			{@render sidebar()}
		</div>
	{/if}
</div>

<style>
	.library-shell {
		display: flex;
		width: 100%;
		height: 100vh;
		overflow: hidden;
		position: relative;
		background: var(--bg-primary);
	}

	.blobs {
		position: absolute;
		inset: 0;
		pointer-events: none;
		z-index: 0;
	}

	.blob {
		position: absolute;
		border-radius: 50%;
	}

	.blob-1 {
		width: 400px;
		height: 400px;
		background: var(--blob-accent);
		filter: blur(100px);
		top: -80px;
		left: -60px;
	}

	.blob-2 {
		width: 350px;
		height: 350px;
		background: var(--blob-purple);
		filter: blur(90px);
		top: 300px;
		left: 40px;
	}

	.blob-3 {
		width: 300px;
		height: 300px;
		background: var(--blob-green);
		filter: blur(80px);
		bottom: -40px;
		right: 100px;
	}

	.blob-4 {
		width: 280px;
		height: 280px;
		background: var(--blob-orange);
		filter: blur(100px);
		top: 100px;
		right: -40px;
	}

	.sidebar-slot {
		width: 220px;
		min-width: 220px;
		flex-shrink: 0;
		position: relative;
		z-index: 2;
		overflow: hidden;
		height: 100%;
		transition:
			width 340ms cubic-bezier(0.4, 0, 0.2, 1),
			min-width 340ms cubic-bezier(0.4, 0, 0.2, 1);
	}

	.sidebar-slot.is-collapsed {
		width: 40px;
		min-width: 40px;
		border-right: 0.5px solid var(--border-primary);
		background: var(--sidebar-bg);
	}

	.sidebar-full {
		min-width: 220px;
		height: 100%;
		display: flex;
		flex-direction: column;
		opacity: 1;
		transition: opacity 140ms ease;
	}

	.is-collapsed .sidebar-full {
		opacity: 0;
		pointer-events: none;
	}

	.sidebar-reveal-btn {
		position: absolute;
		top: 14px;
		left: 6px;
		width: 28px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-tertiary);
		cursor: pointer;
		opacity: 0;
		pointer-events: none;
		transition: opacity 160ms ease;
	}

	.is-collapsed .sidebar-reveal-btn {
		opacity: 1;
		pointer-events: auto;
		transition: opacity 160ms ease 180ms;
	}

	.sidebar-reveal-btn svg {
		width: 18px;
		height: 18px;
	}

	.sidebar-reveal-btn:hover {
		background: var(--fill-hover);
		color: var(--text-secondary);
	}

	.content-slot {
		flex: 1;
		min-width: 0;
		display: flex;
		overflow: hidden;
		position: relative;
		z-index: 1;
	}

	.drawer-scrim {
		position: absolute;
		inset: 0;
		background: var(--overlay-backdrop, rgba(0, 0, 0, 0.34));
		z-index: 30;
	}

	.drawer {
		position: absolute;
		top: 0;
		left: 0;
		bottom: 0;
		width: 280px;
		z-index: 31;
		display: flex;
		box-shadow: 18px 0 56px rgba(0, 0, 0, 0.28);
	}

	.drawer > :global(.sidebar) {
		width: 100%;
		min-width: 0;
	}
</style>
