<script lang="ts">
	import type { SearchSuggestionResponse } from '$lib/api/generated/types.gen';

	interface Props {
		suggestions: SearchSuggestionResponse[];
		highlightedIndex: number;
		onSelect: (suggestion: SearchSuggestionResponse) => void;
	}

	let { suggestions, highlightedIndex, onSelect }: Props = $props();

	function kindIcon(kind: string): string {
		switch (kind) {
			case 'tag':
				return 'tag';
			case 'collection':
				return 'folder';
			case 'filter':
				return 'filter';
			case 'recent':
				return 'clock';
			case 'entity':
				return 'entity';
			case 'sender':
			case 'author':
				return 'entity';
			case 'list':
				return 'folder';
			default:
				return 'filter';
		}
	}
</script>

<div class="autocomplete-dropdown" role="listbox" aria-label="Search suggestions">
	{#each suggestions as suggestion, i (suggestion.insert_text)}
		<button
			type="button"
			class="autocomplete-item"
			class:highlighted={i === highlightedIndex}
			role="option"
			aria-selected={i === highlightedIndex}
			onmousedown={(e) => {
				e.preventDefault();
				onSelect(suggestion);
			}}
		>
			<span class="suggestion-icon" aria-hidden="true">
				{#if kindIcon(suggestion.kind) === 'tag'}
					<svg viewBox="0 0 24 24"
						><path
							d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"
						/><circle cx="7" cy="7" r="1.5" fill="currentColor" stroke="none" /></svg
					>
				{:else if kindIcon(suggestion.kind) === 'folder'}
					<svg viewBox="0 0 24 24"
						><path
							d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z"
						/></svg
					>
				{:else if kindIcon(suggestion.kind) === 'clock'}
					<svg viewBox="0 0 24 24"
						><circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" /></svg
					>
				{:else if kindIcon(suggestion.kind) === 'entity'}
					<svg
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="1.5"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
						<circle cx="12" cy="7" r="4" />
					</svg>
				{:else}
					<svg viewBox="0 0 24 24"
						><polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" /></svg
					>
				{/if}
			</span>
			<span class="suggestion-label">{suggestion.label}</span>
			{#if suggestion.description}
				<span class="suggestion-desc">{suggestion.description}</span>
			{/if}
		</button>
	{/each}
</div>

<style>
	.autocomplete-dropdown {
		position: absolute;
		top: 100%;
		left: 0;
		right: 0;
		margin-top: 4px;
		background: var(--bg-elevated);
		border: 0.5px solid var(--border-secondary);
		border-radius: 10px;
		box-shadow: var(--shadow-3);
		overflow: hidden;
		z-index: 100;
		max-height: 320px;
		overflow-y: auto;
	}

	.autocomplete-item {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 9px 14px;
		border: none;
		background: transparent;
		cursor: pointer;
		text-align: left;
		transition: background 80ms ease;
	}

	.autocomplete-item:hover,
	.autocomplete-item.highlighted {
		background: var(--fill-hover);
	}

	.autocomplete-item.highlighted {
		background: var(--fill-selected);
	}

	.suggestion-icon {
		width: 16px;
		height: 16px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		color: var(--text-tertiary);
	}

	.suggestion-icon svg {
		width: 14px;
		height: 14px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.suggestion-label {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		flex: 1;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.suggestion-desc {
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 400;
		color: var(--text-tertiary);
		flex-shrink: 0;
	}
</style>
