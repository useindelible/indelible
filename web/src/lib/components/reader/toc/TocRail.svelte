<script lang="ts">
	import type { ArticleTocEntry } from '$lib/api';
	import TocPanel from './TocPanel.svelte';

	interface Props {
		entries: ArticleTocEntry[];
		activeIndex: number;
		progress?: number | null;
		onNavigate: (entry: ArticleTocEntry, index: number) => void;
	}

	let { entries, activeIndex, progress = null, onNavigate }: Props = $props();

	// Long outlines (backend caps at 200 entries) tighten the tick rhythm so
	// the rail stays within the viewport; the overflow guard catches the rest.
	const dense = $derived(entries.length > 40);

	let flyoutScrollEl = $state<HTMLDivElement | undefined>(undefined);

	// Keep the active row centered in the card's scroll area. The card stays
	// laid out while hidden (opacity, not display), so positioning it on every
	// active-index change means it is already showing your place on hover.
	$effect(() => {
		const scrollEl = flyoutScrollEl;
		const index = activeIndex;
		if (!scrollEl) return;
		if (index < 0) {
			scrollEl.scrollTop = 0;
			return;
		}
		const active = scrollEl.querySelector<HTMLElement>('.toc-item.active');
		if (!active) return;
		const target = active.offsetTop - (scrollEl.clientHeight - active.offsetHeight) / 2;
		scrollEl.scrollTop = Math.max(0, target);
	});
</script>

<nav class="toc-rail-zone" aria-label="Table of contents">
	<div class="toc-rail" class:dense>
		{#each entries as entry, index (`${entry.source_heading_index}-${entry.id}`)}
			<button
				type="button"
				class="tick"
				class:nested={entry.depth > 0}
				class:done={index < activeIndex}
				class:active={index === activeIndex}
				aria-label={entry.title}
				aria-current={index === activeIndex ? 'true' : undefined}
				onclick={() => onNavigate(entry, index)}
			></button>
		{/each}
	</div>
	<div class="toc-flyout">
		<div class="toc-flyout-scroll" bind:this={flyoutScrollEl}>
			<TocPanel {entries} {activeIndex} {progress} {onNavigate} />
		</div>
	</div>
</nav>

<style>
	.toc-rail-zone {
		position: absolute;
		left: 0;
		top: 0;
		bottom: 0;
		width: 56px;
		z-index: 20;
		pointer-events: none;
	}

	.toc-rail {
		position: absolute;
		left: 14px;
		top: 50%;
		transform: translateY(-50%);
		display: flex;
		flex-direction: column;
		gap: 1px;
		padding: 6px 4px;
		max-height: calc(100% - 96px);
		overflow-y: auto;
		scrollbar-width: none;
		pointer-events: auto;
	}

	.toc-rail::-webkit-scrollbar {
		display: none;
	}

	/* The visible bar is a ::before inside a taller transparent button so the
	   2px mark still has a comfortable hit target. */
	.tick {
		position: relative;
		width: 20px;
		height: 10px;
		padding: 0;
		border: none;
		background: transparent;
		cursor: pointer;
		flex-shrink: 0;
	}

	.toc-rail.dense .tick {
		height: 6px;
	}

	.tick::before {
		content: '';
		position: absolute;
		left: 0;
		top: 50%;
		transform: translateY(-50%);
		width: 16px;
		height: 2px;
		border-radius: 1px;
		background: var(--text-quaternary);
		transition:
			background 150ms ease,
			width 150ms ease;
	}

	.tick.nested::before {
		width: 9px;
	}

	.tick.done::before {
		background: var(--text-tertiary);
	}

	.tick:hover::before {
		background: var(--text-secondary);
	}

	.tick.active::before {
		width: 20px;
		height: 3px;
		background: var(--accent);
	}

	.tick:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 1px;
		border-radius: 3px;
	}

	.toc-flyout {
		position: absolute;
		left: 62px;
		top: 50%;
		transform: translateY(-50%) translateX(-8px);
		width: 288px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-primary);
		border-radius: 12px;
		box-shadow: var(--shadow-3);
		opacity: 0;
		pointer-events: none;
		transition:
			opacity 160ms ease 140ms,
			transform 240ms cubic-bezier(0.25, 1, 0.3, 1) 140ms;
	}

	/* Scrolling lives on an inner wrapper: overflow on the flyout itself would
	   clip the ::before hover bridge. */
	.toc-flyout-scroll {
		/* Positioned so the active row's offsetTop resolves against this
		   scroll container, not the flyout. */
		position: relative;
		max-height: min(70vh, 640px);
		overflow-y: auto;
		border-radius: 12px;
	}

	/* Invisible bridge over the rail-to-card gap so the pointer can cross it. */
	.toc-flyout::before {
		content: '';
		position: absolute;
		left: -26px;
		top: 0;
		bottom: 0;
		width: 26px;
	}

	.toc-rail:hover + .toc-flyout,
	.toc-flyout:hover,
	.toc-rail:focus-within + .toc-flyout {
		opacity: 1;
		pointer-events: auto;
		transform: translateY(-50%) translateX(0);
		transition-delay: 0ms;
	}
</style>
