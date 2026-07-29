<script lang="ts">
	import * as api from '$lib/api';
	import type { CollectionResponse } from '$lib/api/generated/types.gen';
	import { fetchAllPages } from '$lib/api/pagination';

	interface Props {
		value?: string | null;
	}

	let { value = $bindable(null) }: Props = $props();

	let collections = $state<CollectionResponse[]>([]);
	let open = $state(false);
	let loaded = $state(false);
	let pickerEl = $state<HTMLDivElement | undefined>(undefined);

	$effect(() => {
		if (!open) return;
		function handleClickOutside(e: MouseEvent) {
			if (pickerEl && !pickerEl.contains(e.target as Node)) {
				open = false;
			}
		}
		document.addEventListener('click', handleClickOutside, true);
		return () => document.removeEventListener('click', handleClickOutside, true);
	});

	const selectedName = $derived(collections.find((c) => c.id === value)?.name ?? 'Inbox');

	async function loadCollections() {
		if (loaded) return;
		loaded = true;
		try {
			const results = await fetchAllPages(async (cursor) => {
				const resp = await api.listCollections({
					query: { cursor, limit: 100 }
				});
				if (!resp.data) return undefined;
				return {
					data: resp.data.data as CollectionResponse[],
					page: { next_cursor: resp.data.page.next_cursor ?? null }
				};
			});
			collections = results;
		} catch {
			loaded = false;
		}
	}

	function toggle() {
		open = !open;
		if (open) loadCollections();
	}

	function select(id: string | null) {
		value = id;
		open = false;
	}

	function depthPrefix(col: CollectionResponse): string {
		let depth = 0;
		let current = col;
		while (current.parent_id) {
			depth++;
			const parent = collections.find((c) => c.id === current.parent_id);
			if (!parent) break;
			current = parent;
		}
		return '\u00A0\u00A0'.repeat(depth);
	}
</script>

<div class="collection-picker" bind:this={pickerEl}>
	<button type="button" class="picker-trigger" onclick={toggle} aria-expanded={open}>
		<span class="collection-label">{selectedName}</span>
		<span class="collection-chevron" aria-hidden="true">
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

	{#if open}
		<div class="picker-dropdown">
			<button
				type="button"
				class="picker-option"
				class:selected={value === null}
				onclick={() => select(null)}
			>
				Inbox (no collection)
			</button>
			{#each collections as col (col.id)}
				<button
					type="button"
					class="picker-option"
					class:selected={value === col.id}
					onclick={() => select(col.id)}
				>
					{depthPrefix(col)}{col.icon ?? ''}{col.icon ? ' ' : ''}{col.name}
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	.collection-picker {
		position: relative;
		width: 100%;
	}

	.picker-trigger {
		width: 100%;
		height: 44px;
		border-radius: 10px;
		border: 1px solid var(--border-primary);
		background: transparent;
		padding: 0 14px;
		display: flex;
		align-items: center;
		justify-content: space-between;
		cursor: pointer;
		font-family: var(--font-sans);
	}

	.collection-label {
		font-size: 14px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--text-primary);
	}

	.collection-chevron {
		color: var(--text-tertiary);
		display: flex;
		align-items: center;
	}

	.collection-chevron svg {
		width: 16px;
		height: 16px;
	}

	.picker-dropdown {
		position: absolute;
		top: 100%;
		left: 0;
		right: 0;
		margin-top: 4px;
		max-height: 240px;
		overflow-y: auto;
		background: var(--bg-primary);
		border: 1px solid var(--border-primary);
		border-radius: 10px;
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
		z-index: 10;
		padding: 4px;
	}

	.picker-option {
		display: block;
		width: 100%;
		padding: 8px 12px;
		border: none;
		background: transparent;
		text-align: left;
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-primary);
		border-radius: 6px;
		cursor: pointer;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		transition: background 0.1s ease;
	}

	.picker-option:hover {
		background: var(--fill-hover);
	}

	.picker-option.selected {
		background: var(--fill-selected);
		font-weight: 500;
	}
</style>
