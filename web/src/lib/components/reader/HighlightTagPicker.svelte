<script lang="ts">
	import type { TagResponse } from '$lib/api/generated/types.gen';
	import { t } from '$lib/i18n';

	interface Props {
		x: number;
		y: number;
		above: boolean;
		tags: string[];
		tagInput: string;
		suggestions: TagResponse[];
		suggestionIndex: number;
		onTagInputChange: (value: string) => void;
		onSuggestionIndexChange: (index: number) => void;
		onAddTag: (name: string) => void | Promise<void>;
		onRemoveTag: (name: string) => void | Promise<void>;
		onClose: () => void;
	}

	let {
		x,
		y,
		above,
		tags,
		tagInput,
		suggestions,
		suggestionIndex,
		onTagInputChange,
		onSuggestionIndexChange,
		onAddTag,
		onRemoveTag,
		onClose
	}: Props = $props();

	function nextSuggestionIndex(offset: number): number {
		const length = Math.max(suggestions.length, 1);
		return (suggestionIndex + offset + length) % length;
	}

	async function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			onSuggestionIndexChange(nextSuggestionIndex(1));
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			onSuggestionIndexChange(nextSuggestionIndex(-1));
		} else if (e.key === 'Enter') {
			e.preventDefault();
			const suggested = suggestions[suggestionIndex];
			if (suggested) {
				await onAddTag(suggested.name);
			} else if (tagInput.trim()) {
				await onAddTag(tagInput.trim());
			}
		} else if (e.key === 'Escape') {
			onClose();
		} else if (e.key === 'Backspace' && !tagInput && tags.length > 0) {
			await onRemoveTag(tags[tags.length - 1]!);
		}
	}
</script>

<div
	class="highlight-tag-picker"
	style:left="{x}px"
	style:top="{y - 8}px"
	style:position="fixed"
	style:transform={above ? 'translateX(-50%) translateY(-100%)' : 'translateX(-50%)'}
>
	{#if tags.length > 0}
		<div class="hl-applied-tags">
			{#each tags as tag (tag)}
				<span class="hl-applied-tag">
					{tag}
					<button
						type="button"
						class="hl-applied-tag-remove"
						onclick={() => onRemoveTag(tag)}
						aria-label={$t('reader_remove_tag', { values: { tag } })}
					>
						<svg
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2.5"
							stroke-linecap="round"
							stroke-linejoin="round"
							><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg
						>
					</button>
				</span>
			{/each}
		</div>
	{/if}
	<div class="hl-tag-field-wrap">
		<svg
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
			><circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" /></svg
		>
		<input
			type="text"
			class="hl-tag-field"
			placeholder={$t('library_tag_add')}
			value={tagInput}
			oninput={(e) => {
				onTagInputChange((e.target as HTMLInputElement).value);
				onSuggestionIndexChange(0);
			}}
			onkeydown={handleKeydown}
		/>
	</div>
	{#if suggestions.length > 0}
		<div class="hl-suggestions-list">
			{#each suggestions as suggestion, i (suggestion.id)}
				<button
					type="button"
					class="hl-suggestion"
					class:active={i === suggestionIndex}
					onmouseenter={() => onSuggestionIndexChange(i)}
					onclick={() => onAddTag(suggestion.name)}
				>
					<span class="hl-suggestion-dot" style:background={suggestion.color ?? undefined}></span>
					{suggestion.name}
					{#if suggestion.highlight_count > 0}
						<span class="hl-suggestion-count">{suggestion.highlight_count}</span>
					{/if}
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	.highlight-tag-picker {
		background: var(--bg-elevated);
		backdrop-filter: blur(20px) saturate(180%);
		-webkit-backdrop-filter: blur(20px) saturate(180%);
		border-radius: 10px;
		box-shadow: var(--shadow-3);
		z-index: 20;
		padding: 8px;
		width: 240px;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.hl-applied-tags {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}

	.hl-applied-tag {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		border-radius: 980px;
		background: var(--fill-selected);
		color: var(--accent);
		font-size: 11px;
		font-weight: 500;
		letter-spacing: -0.005em;
		font-family: var(--font-sans);
	}

	.hl-applied-tag-remove {
		background: none;
		border: none;
		padding: 0;
		cursor: pointer;
		display: flex;
		align-items: center;
		color: var(--accent);
		opacity: 0.6;
	}

	.hl-applied-tag-remove:hover {
		opacity: 1;
	}

	.hl-applied-tag-remove svg {
		width: 9px;
		height: 9px;
	}

	.hl-tag-field-wrap {
		display: flex;
		align-items: center;
		gap: 6px;
		background: var(--bg-secondary);
		border-radius: 8px;
		padding: 6px 8px;
	}

	.hl-tag-field-wrap svg {
		width: 12px;
		height: 12px;
		color: var(--text-tertiary);
		flex-shrink: 0;
	}

	.hl-tag-field {
		border: none;
		background: transparent;
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-primary);
		outline: none;
		width: 100%;
		letter-spacing: -0.01em;
	}

	.hl-tag-field::placeholder {
		color: var(--text-tertiary);
	}

	.hl-suggestions-list {
		border-radius: 8px;
		overflow: hidden;
		border: 0.5px solid var(--border-secondary);
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
	}

	.hl-suggestion {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 7px 10px;
		font-size: 12px;
		font-weight: 400;
		color: var(--text-primary);
		cursor: pointer;
		letter-spacing: -0.005em;
		font-family: var(--font-sans);
		background: var(--bg-elevated);
		border: none;
		width: 100%;
		text-align: left;
		transition: background 100ms ease;
	}

	.hl-suggestion.active {
		background: var(--fill-hover);
	}

	.hl-suggestion-dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		flex-shrink: 0;
		background: var(--border-secondary);
	}

	.hl-suggestion-count {
		margin-left: auto;
		font-size: 11px;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
	}
</style>
