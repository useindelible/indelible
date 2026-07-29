<script lang="ts">
	interface Props {
		tags: string[];
		addingTag: boolean;
		newTagValue: string;
		onAddTag: (raw: string) => void;
		onRemoveTag: (tag: string) => void;
		onTagKeydown: (event: KeyboardEvent) => void;
	}

	let {
		tags,
		addingTag = $bindable(),
		newTagValue = $bindable(),
		onAddTag,
		onRemoveTag,
		onTagKeydown
	}: Props = $props();

	let inputEl = $state<HTMLInputElement | undefined>(undefined);

	$effect(() => {
		if (addingTag && inputEl) {
			inputEl.focus();
		}
	});
</script>

{#each tags as tag (tag)}
	<span class="tag-pill">
		{tag}
		<button
			type="button"
			class="tag-x"
			aria-label="Remove tag {tag}"
			onclick={() => onRemoveTag(tag)}>&times;</button
		>
	</span>
{/each}

{#if addingTag}
	<input
		bind:this={inputEl}
		bind:value={newTagValue}
		class="tag-inline-input"
		type="text"
		placeholder="tag name"
		onkeydown={onTagKeydown}
		onblur={() => {
			if (newTagValue.trim()) onAddTag(newTagValue);
			else addingTag = false;
		}}
	/>
{:else}
	<button
		type="button"
		class="tag-add-sm"
		aria-label="Add tag"
		onclick={() => {
			addingTag = true;
		}}>+</button
	>
{/if}

<style>
	.tag-pill {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 4px 6px 4px 8px;
		border-radius: 6px;
		background: var(--fill-selected-strong);
		color: var(--accent);
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 500;
	}

	.tag-x {
		background: none;
		border: none;
		padding: 0;
		margin: 0;
		cursor: pointer;
		font-size: 14px;
		line-height: 1;
		color: var(--accent);
		opacity: 0.6;
		display: flex;
		align-items: center;
	}

	.tag-x:hover {
		opacity: 1;
	}

	.tag-add-sm {
		width: 24px;
		height: 24px;
		border-radius: 6px;
		border: 1.5px dashed var(--border-secondary);
		background: none;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		font-size: 14px;
		color: var(--text-tertiary);
		flex-shrink: 0;
		padding: 0;
		line-height: 1;
	}

	.tag-add-sm:hover {
		border-color: var(--accent);
		color: var(--accent);
	}

	.tag-inline-input {
		height: 24px;
		border-radius: 6px;
		border: 1.5px solid var(--accent);
		background: none;
		padding: 0 6px;
		font-family: var(--font-sans);
		font-size: 11px;
		color: var(--text-primary);
		outline: none;
		min-width: 80px;
		max-width: 120px;
	}
</style>
