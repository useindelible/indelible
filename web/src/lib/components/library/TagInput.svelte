<script lang="ts">
	import { t } from '$lib/i18n';
	import * as api from '$lib/api';
	import type { TagResponse } from '$lib/api/generated/types.gen';
	import { fetchAllPages } from '$lib/api/pagination';
	import { sanitizeColor } from '$lib/utils/color';

	interface Props {
		tags?: string[];
		autofocus?: boolean;
	}

	let { tags = $bindable([]), autofocus = false }: Props = $props();

	let inputValue = $state('');
	let inputEl = $state<HTMLInputElement | undefined>(undefined);
	let allKnownTags = $state<TagResponse[]>([]);
	let loaded = $state(false);
	let isFocused = $state(false);

	function normalizeTagName(value: string): string {
		return value.trim().toLowerCase();
	}

	const suggestions = $derived.by(() => {
		if (!isFocused || !inputValue.trim()) return [];
		const q = inputValue.trim().toLowerCase();
		return allKnownTags
			.filter(
				(t) =>
					(t.name.toLowerCase().includes(q) ||
						t.aliases.some((a) => a.toLowerCase().includes(q))) &&
					!tags.some((tag) => normalizeTagName(tag) === normalizeTagName(t.name))
			)
			.slice(0, 8);
	});

	async function loadTags() {
		if (loaded) return;
		loaded = true;
		try {
			const results = await fetchAllPages(async (cursor) => {
				const resp = await api.listTags({
					query: { cursor, limit: 100 }
				});
				if (!resp.data) return undefined;
				return {
					data: resp.data.data as TagResponse[],
					page: { next_cursor: resp.data.page.next_cursor ?? null }
				};
			});
			allKnownTags = results;
		} catch {
			loaded = false;
		}
	}

	function addTag(raw: string) {
		const trimmed = raw.trim();
		if (trimmed && !tags.some((tag) => normalizeTagName(tag) === normalizeTagName(trimmed))) {
			tags = [...tags, trimmed];
		}
	}

	function removeTag(tag: string) {
		tags = tags.filter((t) => t !== tag);
	}

	function selectSuggestion(tagName: string) {
		addTag(tagName);
		inputValue = '';
		inputEl?.focus();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' || e.key === ',') {
			e.preventDefault();
			addTag(inputValue);
			inputValue = '';
		} else if (e.key === 'Backspace' && inputValue === '' && tags.length > 0) {
			tags = tags.slice(0, -1);
		} else if (e.key === 'Escape') {
			isFocused = false;
			inputEl?.blur();
		}
	}

	function handleFocus() {
		isFocused = true;
		loadTags();
	}

	function handleBlur() {
		// Delay to allow suggestion click to register before hiding
		setTimeout(() => {
			isFocused = false;
			if (inputValue.trim()) {
				addTag(inputValue);
				inputValue = '';
			}
		}, 150);
	}

	function tagColor(name: string): string | undefined {
		const found = allKnownTags.find((t) => t.name.toLowerCase() === name.toLowerCase());
		return sanitizeColor(found?.color);
	}
</script>

<div class="tag-input-wrapper">
	<div class="tag-input" role="group" aria-label={$t('common_tags')}>
		{#each tags as tag (tag)}
			<span class="tag-pill">
				{#if tagColor(tag)}
					<span class="pill-dot" style="background: {tagColor(tag)}"></span>
				{/if}
				{tag}
				<button
					type="button"
					class="tag-pill-x"
					aria-label={$t('library_tag_remove', { values: { tag } })}
					onclick={(e) => {
						e.stopPropagation();
						removeTag(tag);
					}}
				>
					<svg
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2.5"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<line x1="18" y1="6" x2="6" y2="18" />
						<line x1="6" y1="6" x2="18" y2="18" />
					</svg>
				</button>
			</span>
		{/each}
		<input
			bind:this={inputEl}
			bind:value={inputValue}
			type="text"
			class="tag-text-input"
			placeholder={tags.length === 0 ? $t('library_tag_search_or_add') : ''}
			aria-label={$t('library_tag_add')}
			autofocus={autofocus || undefined}
			onkeydown={handleKeydown}
			onfocus={handleFocus}
			onblur={handleBlur}
		/>
	</div>

	{#if suggestions.length > 0}
		<div class="suggestions-dropdown" role="listbox">
			{#each suggestions as suggestion (suggestion.id)}
				<button
					type="button"
					class="suggestion-item"
					role="option"
					onmousedown={(e) => {
						e.preventDefault();
						selectSuggestion(suggestion.name);
					}}
				>
					<span
						class="suggestion-dot"
						style="background: {sanitizeColor(suggestion.color) ?? 'var(--text-tertiary)'}"
					></span>
					<span class="suggestion-name">{suggestion.name}</span>
					<span class="suggestion-count">{suggestion.item_count}</span>
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	.tag-input-wrapper {
		position: relative;
	}

	.tag-input {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px;
		min-height: 44px;
		border-radius: 10px;
		border: 1px solid var(--border-primary);
		padding: 6px 10px;
		cursor: text;
		transition:
			border-color 0.15s ease,
			box-shadow 0.15s ease;
	}

	.tag-input:focus-within {
		border-color: var(--accent);
		box-shadow: 0 0 0 3px var(--fill-selected);
	}

	.tag-pill {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 4px 8px;
		border-radius: 7px;
		background: var(--fill-selected);
		color: var(--accent);
		font-size: 13px;
		font-weight: 500;
		letter-spacing: -0.01em;
		line-height: 1;
	}

	.pill-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.tag-pill-x {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 14px;
		height: 14px;
		background: none;
		border: none;
		padding: 0;
		cursor: pointer;
		opacity: 0.6;
		color: var(--accent);
	}

	.tag-pill-x:hover {
		opacity: 1;
	}

	.tag-pill-x svg {
		width: 10px;
		height: 10px;
	}

	.tag-text-input {
		flex: 1;
		min-width: 80px;
		border: none;
		background: transparent;
		outline: none;
		font-family: var(--font-sans);
		font-size: 14px;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		padding: 2px 0;
	}

	.tag-text-input::placeholder {
		color: var(--text-tertiary);
	}

	.suggestions-dropdown {
		position: absolute;
		top: 100%;
		left: 0;
		right: 0;
		margin-top: 4px;
		max-height: 200px;
		overflow-y: auto;
		background: var(--bg-elevated);
		border: 0.5px solid var(--border-secondary);
		border-radius: 10px;
		box-shadow: var(--shadow-3);
		z-index: 10;
		padding: 4px;
	}

	.suggestion-item {
		display: flex;
		align-items: center;
		gap: 8px;
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
		transition: background 0.1s ease;
	}

	.suggestion-item:hover {
		background: var(--fill-hover);
	}

	.suggestion-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.suggestion-name {
		flex: 1;
	}

	.suggestion-count {
		font-size: 11px;
		color: var(--text-tertiary);
	}
</style>
