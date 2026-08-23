<script lang="ts">
	import { onMount } from 'svelte';
	import TagColorPicker from '$lib/components/tags/TagColorPicker.svelte';
	import { t } from '$lib/i18n';

	interface Props {
		color: string | null;
		name: string;
		parentId: string | null;
		error: string | null;
		existingTag: { id: string; name: string } | null;
		onClose: () => void;
		onColorChange: (color: string | null) => void;
		onNameChange: (name: string) => void;
		onOpenExisting: () => void;
		onSubmit: () => void;
	}

	let {
		color,
		name,
		parentId,
		error,
		existingTag,
		onClose,
		onColorChange,
		onNameChange,
		onOpenExisting,
		onSubmit
	}: Props = $props();
	let nameInputEl: HTMLInputElement | undefined = $state();

	onMount(() => {
		nameInputEl?.focus();
	});

	$effect(() => {
		if (!error) return;
		nameInputEl?.focus();
		nameInputEl?.select();
	});
</script>

<div
	class="cmd-backdrop"
	role="dialog"
	aria-modal="true"
	aria-label={$t('tag_new')}
	tabindex="-1"
	onclick={onClose}
	onkeydown={(event) => {
		if (event.key === 'Escape') onClose();
	}}
>
	<div
		class="cmd-card"
		role="none"
		onclick={(event) => event.stopPropagation()}
		onkeydown={() => {}}
	>
		<div class="cmd-input-zone">
			<div class="cmd-input-wrap">
				<svg
					class="cmd-icon"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
				>
					<path
						d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"
					/>
					<line x1="7" y1="7" x2="7.01" y2="7" />
				</svg>
				<input
					bind:this={nameInputEl}
					class="cmd-input"
					type="text"
					value={name}
					placeholder={$t(parentId ? 'tag_child_name_placeholder' : 'tag_name_placeholder')}
					aria-invalid={error ? 'true' : undefined}
					aria-describedby={error ? 'create-tag-error' : undefined}
					oninput={(event) => onNameChange(event.currentTarget.value)}
					onkeydown={(event) => {
						if (event.key === 'Enter' && name.trim()) onSubmit();
					}}
				/>
			</div>
		</div>
		<div class="cmd-body">
			{#if error}
				<div class="cmd-error" id="create-tag-error" role="alert">
					<span>{error}</span>
					{#if existingTag}
						<button type="button" class="cmd-existing" onclick={onOpenExisting}>
							{$t('tag_open_existing', { values: { name: existingTag.name } })}
						</button>
					{/if}
				</div>
			{/if}
			<div class="cmd-color-row">
				<span class="cmd-color-label">{$t('tag_color')}</span>
				<TagColorPicker value={color} onChange={onColorChange} />
			</div>
		</div>
		<div class="cmd-controls">
			<button type="button" class="cmd-secondary" onclick={onClose}>{$t('common_cancel')}</button>
			<button type="button" class="cmd-action" disabled={!name.trim()} onclick={onSubmit}>
				{$t('tag_create')}
			</button>
		</div>
	</div>
</div>

<style>
	.cmd-backdrop {
		position: fixed;
		inset: 0;
		z-index: 300;
		display: flex;
		align-items: flex-start;
		justify-content: center;
		background: var(--overlay-backdrop);
		backdrop-filter: blur(4px);
		-webkit-backdrop-filter: blur(4px);
		padding-top: 80px;
		box-sizing: border-box;
	}
	.cmd-card {
		width: 460px;
		max-width: calc(100vw - 32px);
		border-radius: 14px;
		background: var(--bg-elevated);
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.22),
			0 0 0 0.5px rgba(0, 0, 0, 0.06);
		overflow: hidden;
	}
	.cmd-input-zone {
		padding: 8px 8px 0;
	}
	.cmd-body {
		padding: 8px 8px 4px;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.cmd-error {
		margin: 0 8px;
		padding: 9px 10px;
		border-radius: 8px;
		background: color-mix(in oklab, var(--destructive) 9%, transparent);
		color: var(--destructive);
		font-size: 12px;
		line-height: 1.4;
	}
	.cmd-existing {
		display: block;
		margin-top: 5px;
		border: 0;
		padding: 0;
		background: transparent;
		color: var(--accent);
		font-size: 12px;
		font-weight: 600;
		text-decoration: underline;
		text-underline-offset: 2px;
	}
	.cmd-controls {
		padding: 10px 16px 14px;
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.cmd-input-wrap {
		position: relative;
	}
	.cmd-icon {
		position: absolute;
		left: 14px;
		top: 50%;
		transform: translateY(-50%);
		width: 16px;
		height: 16px;
		stroke: var(--text-tertiary);
		fill: none;
		pointer-events: none;
	}
	.cmd-input {
		width: 100%;
		height: 48px;
		border-radius: 10px;
		border: none;
		outline: 0;
		background: var(--bg-secondary);
		color: var(--text-primary);
		font: 500 15px/1.4 var(--font-sans);
		letter-spacing: -0.01em;
		padding: 0 16px 0 40px;
		box-sizing: border-box;
	}
	.cmd-color-row {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 6px 8px;
	}
	.cmd-color-label {
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
	}
	button {
		font: inherit;
		cursor: pointer;
	}
	.cmd-secondary,
	.cmd-action {
		border-radius: 980px;
		padding: 6px 14px;
		font-size: 13px;
		font-weight: 500;
		letter-spacing: -0.01em;
	}
	.cmd-secondary {
		border: 1px solid var(--border-primary);
		background: transparent;
		color: var(--text-secondary);
	}
	.cmd-action {
		margin-left: auto;
		border: 0;
		background: var(--accent);
		color: var(--text-on-color);
		padding: 6px 16px;
		font-weight: 600;
	}
	.cmd-action:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
