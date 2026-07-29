<script lang="ts">
	import type { CollectionResponse } from '$lib/api/generated/types.gen';
	import { SvelteSet } from 'svelte/reactivity';
	import { getCollections } from '$lib/stores/collections.svelte';

	interface Props {
		collection?: CollectionResponse | null;
		parentId?: string | null;
		allCollections?: CollectionResponse[];
		onClose: () => void;
		onSaved: (collection: CollectionResponse) => void;
	}

	let {
		collection = null,
		parentId = null,
		allCollections = [],
		onClose,
		onSaved
	}: Props = $props();

	const store = getCollections();
	const isEdit = $derived(collection !== null);

	let name = $state(collection?.name ?? '');
	let description = $state(collection?.description ?? '');
	let icon = $state(collection?.icon ?? '');
	let selectedParentId = $state<string | null>(collection?.parent_id ?? parentId);
	let saving = $state(false);
	let error = $state<string | null>(null);

	const canSave = $derived(name.trim().length > 0 && !saving);
	const parentSource = $derived(allCollections.length > 0 ? allCollections : store.allCollections);
	const descendantIds = $derived.by(() => {
		if (!collection) return new SvelteSet<string>();

		const descendants = new SvelteSet<string>();
		const queue = [collection.id];

		while (queue.length > 0) {
			const currentId = queue.shift();
			if (!currentId) continue;

			for (const candidate of parentSource) {
				if (candidate.parent_id !== currentId || descendants.has(candidate.id)) continue;
				descendants.add(candidate.id);
				queue.push(candidate.id);
			}
		}

		return descendants;
	});
	const parentOptions = $derived.by(() =>
		parentSource.filter(
			(candidate) => candidate.id !== collection?.id && !descendantIds.has(candidate.id)
		)
	);

	async function handleSubmit() {
		if (!canSave) return;
		saving = true;
		error = null;

		const body = {
			name: name.trim(),
			description: description.trim() || null,
			icon: icon.trim() || null,
			color: null,
			parent_id: selectedParentId
		};

		if (isEdit && collection) {
			const updated = await store.updateCollection(collection.id, body);
			if (updated) {
				onSaved(updated);
				onClose();
			} else {
				error = 'Failed to update collection';
			}
		} else {
			const created = await store.createCollection(body);
			if (created) {
				onSaved(created);
				onClose();
			} else {
				error = 'Failed to create collection';
			}
		}

		saving = false;
	}
</script>

<div
	class="cmd-backdrop"
	role="dialog"
	aria-modal="true"
	aria-label={isEdit ? 'Edit collection' : 'New collection'}
	tabindex="-1"
	onclick={onClose}
	onkeydown={(e) => {
		if (e.key === 'Escape') {
			e.preventDefault();
			onClose();
		}
	}}
>
	<div class="cmd-card" role="none" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
		<div class="cmd-input-zone">
			<div class="cmd-input-wrap">
				<span class="cmd-folder-icon" aria-hidden="true">
					<svg
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="1.6"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<path
							d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z"
						/>
					</svg>
				</span>
				<input
					type="text"
					class="cmd-name-input"
					bind:value={name}
					placeholder="Collection name"
					required
					autofocus
				/>
			</div>
		</div>

		<form
			class="cmd-body"
			onsubmit={(e) => {
				e.preventDefault();
				handleSubmit();
			}}
		>
			{#if error}
				<div class="error-msg" role="alert">{error}</div>
			{/if}

			<div class="cmd-field">
				<span class="cmd-label">Icon (optional)</span>
				<div class="emoji-row">
					<span class="emoji-preview" aria-hidden="true">{icon || '📨'}</span>
					<input
						type="text"
						class="emoji-input"
						bind:value={icon}
						placeholder="Paste or type an emoji…"
						maxlength="2"
					/>
				</div>
				<p class="emoji-hint">Leave blank to use 📨 as the default</p>
			</div>

			<div class="cmd-field">
				<span class="cmd-label">Description (optional)</span>
				<textarea
					class="cmd-textarea"
					bind:value={description}
					placeholder="A short description of this collection"
					rows="2"
				></textarea>
			</div>

			<div class="cmd-field">
				<span class="cmd-label">Parent collection</span>
				<div class="select-wrap">
					<select class="cmd-select" bind:value={selectedParentId}>
						<option value={null}>None (top-level)</option>
						{#each parentOptions as col (col.id)}
							<option value={col.id}>{col.name}</option>
						{/each}
					</select>
					<span class="select-chevron" aria-hidden="true">
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
				</div>
			</div>

			<div class="cmd-controls">
				<button type="button" class="cmd-secondary" onclick={onClose}>Cancel</button>
				<button type="submit" class="cmd-action" disabled={!canSave}>
					{saving ? 'Saving…' : isEdit ? 'Save Collection' : 'Create Collection'}
				</button>
			</div>
		</form>
	</div>
</div>

<style>
	.cmd-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.4);
		backdrop-filter: blur(4px);
		-webkit-backdrop-filter: blur(4px);
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 80px;
		z-index: 300;
		box-sizing: border-box;
	}

	:global([data-theme='dark']) .cmd-backdrop {
		background: rgba(0, 0, 0, 0.6);
	}

	.cmd-card {
		width: 460px;
		max-width: calc(100vw - 32px);
		background: var(--bg-elevated);
		border-radius: 14px;
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.22),
			0 0 0 0.5px rgba(0, 0, 0, 0.06);
	}

	:global([data-theme='dark']) .cmd-card {
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.55),
			0 0 0 0.5px rgba(255, 255, 255, 0.08);
	}

	/* Name input zone */
	.cmd-input-zone {
		padding: 16px 16px 0;
	}

	.cmd-input-wrap {
		position: relative;
		display: flex;
		align-items: center;
	}

	.cmd-folder-icon {
		position: absolute;
		left: 12px;
		width: 18px;
		height: 18px;
		color: var(--text-tertiary);
		display: flex;
		align-items: center;
		justify-content: center;
		pointer-events: none;
	}

	.cmd-folder-icon svg {
		width: 18px;
		height: 18px;
	}

	.cmd-name-input {
		width: 100%;
		height: 48px;
		border-radius: var(--radius-md);
		background: var(--bg-secondary);
		border: none;
		padding: 0 14px 0 40px;
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 500;
		color: var(--text-primary);
		outline: none;
		transition: box-shadow 0.15s ease;
	}

	.cmd-name-input::placeholder {
		color: var(--text-tertiary);
		font-weight: 400;
	}

	.cmd-name-input:focus {
		box-shadow: 0 0 0 2px var(--accent);
	}

	/* Body */
	.cmd-body {
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.error-msg {
		padding: 10px 14px;
		border-radius: var(--radius-md);
		background: var(--fill-danger);
		color: var(--destructive);
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
	}

	.cmd-field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.cmd-label {
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.05em;
		text-transform: uppercase;
		color: var(--text-tertiary);
	}

	/* Emoji field */
	.emoji-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.emoji-preview {
		width: 36px;
		height: 36px;
		border-radius: var(--radius-sm);
		background: var(--bg-secondary);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 20px;
		line-height: 1;
		flex-shrink: 0;
	}

	.emoji-input {
		flex: 1;
		height: 36px;
		border-radius: var(--radius-md);
		background: var(--bg-secondary);
		border: none;
		padding: 0 12px;
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--text-primary);
		outline: none;
		transition: box-shadow 0.15s ease;
	}

	.emoji-input::placeholder {
		color: var(--text-tertiary);
	}

	.emoji-input:focus {
		box-shadow: 0 0 0 2px var(--accent);
	}

	.emoji-hint {
		font-family: var(--font-sans);
		font-size: 11px;
		color: var(--text-tertiary);
		margin: 0;
	}

	/* Description textarea */
	.cmd-textarea {
		border-radius: var(--radius-md);
		background: var(--bg-secondary);
		border: none;
		padding: 10px 12px;
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--text-primary);
		min-height: 72px;
		resize: vertical;
		outline: none;
		line-height: 1.5;
		transition: box-shadow 0.15s ease;
	}

	.cmd-textarea::placeholder {
		color: var(--text-tertiary);
	}

	.cmd-textarea:focus {
		box-shadow: 0 0 0 2px var(--accent);
	}

	/* Parent select */
	.select-wrap {
		position: relative;
	}

	.cmd-select {
		width: 100%;
		height: 40px;
		border-radius: var(--radius-md);
		background: var(--bg-secondary);
		border: none;
		padding: 0 36px 0 12px;
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--text-primary);
		outline: none;
		appearance: none;
		cursor: pointer;
		transition: box-shadow 0.15s ease;
	}

	.cmd-select:focus {
		box-shadow: 0 0 0 2px var(--accent);
	}

	.select-chevron {
		position: absolute;
		right: 10px;
		top: 50%;
		transform: translateY(-50%);
		width: 16px;
		height: 16px;
		color: var(--text-tertiary);
		pointer-events: none;
	}

	.select-chevron svg {
		width: 16px;
		height: 16px;
	}

	/* Controls */
	.cmd-controls {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 16px 14px;
	}

	.cmd-secondary {
		padding: 6px 14px;
		border-radius: 980px;
		border: 1px solid var(--border-primary);
		background: transparent;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-secondary);
		cursor: pointer;
		transition: background 120ms ease;
		letter-spacing: -0.01em;
	}

	.cmd-secondary:hover {
		background: var(--fill-hover);
	}

	.cmd-action {
		margin-left: auto;
		padding: 6px 16px;
		border-radius: 980px;
		border: none;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		letter-spacing: -0.01em;
		color: var(--text-on-color);
		background: var(--accent);
		flex-shrink: 0;
		transition: opacity 120ms ease;
	}

	.cmd-action:hover:not(:disabled) {
		opacity: 0.88;
	}

	.cmd-action:disabled {
		opacity: 0.32;
		cursor: not-allowed;
	}
</style>
