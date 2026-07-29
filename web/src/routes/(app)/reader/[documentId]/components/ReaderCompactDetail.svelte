<script lang="ts">
	import type { DocumentListEntry } from '$lib/api';
	import DetailPanel from '$lib/components/library/DetailPanel.svelte';

	import ReaderSidePanel from './ReaderSidePanel.svelte';

	interface Props {
		item: DocumentListEntry;
		isCompact: boolean;
		isMobile: boolean;
		compactDetailOpen: boolean;
		showDetailPanel: boolean;
		onClose: () => void;
	}

	let { item, isCompact, isMobile, compactDetailOpen, showDetailPanel, onClose }: Props = $props();
</script>

{#if isCompact}
	{#if compactDetailOpen}
		{#if isMobile}
			<div class="reader-detail m-detail">
				<div class="m-detailbar">
					<button type="button" class="m-back" onclick={onClose} aria-label="Back to article">
						<svg
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
							aria-hidden="true"
						>
							<polyline points="15 18 9 12 15 6" />
						</svg>
					</button>
					<span class="m-dtitle">{item.title}</span>
				</div>
				<DetailPanel {item} />
			</div>
		{:else}
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="detail-scrim" onclick={onClose}></div>
			<div class="reader-detail detail-overlay">
				<DetailPanel {item} />
			</div>
		{/if}
	{/if}
{:else if showDetailPanel}
	<ReaderSidePanel {item} />
{/if}

<style>
	.reader-detail {
		display: contents;
		--detail-tabs-height: 44px;
	}

	.detail-scrim {
		position: absolute;
		inset: 0;
		background: rgba(0, 0, 0, 0.1);
		z-index: 20;
	}

	.reader-detail.detail-overlay {
		display: flex;
		position: absolute;
		top: 0;
		right: 0;
		bottom: 0;
		width: 330px;
		z-index: 21;
		background: var(--bg-elevated);
		box-shadow: -18px 0 56px rgba(0, 0, 0, 0.18);
	}

	.detail-overlay :global(.detail-panel) {
		width: 100%;
		min-width: 0;
		background: var(--bg-elevated);
		backdrop-filter: none;
		-webkit-backdrop-filter: none;
	}

	.reader-detail.m-detail {
		display: flex;
		position: absolute;
		inset: 0;
		z-index: 21;
		flex-direction: column;
		background: var(--bg-content);
	}

	.m-detailbar {
		height: 52px;
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 0 8px;
		border-bottom: 0.5px solid var(--border-primary);
		flex-shrink: 0;
		background: var(--bg-content);
	}

	.m-back {
		width: 34px;
		height: 34px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		flex-shrink: 0;
	}

	.m-back:hover {
		background: var(--fill-hover);
	}

	.m-back svg {
		width: 20px;
		height: 20px;
	}

	.m-dtitle {
		font-size: 15px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		min-width: 0;
	}

	.m-detail :global(.detail-panel) {
		width: 100%;
		min-width: 0;
		flex: 1;
		border-left: none;
		background: var(--bg-content);
		backdrop-filter: none;
		-webkit-backdrop-filter: none;
	}
</style>
