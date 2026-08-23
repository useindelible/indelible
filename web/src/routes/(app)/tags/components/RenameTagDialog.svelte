<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';

	interface Props {
		value: string;
		onClose: () => void;
		onSubmit: () => void;
		onValueChange: (value: string) => void;
	}

	let { value, onClose, onSubmit, onValueChange }: Props = $props();
	let nameInputEl: HTMLInputElement | undefined = $state();

	onMount(() => {
		nameInputEl?.focus();
		nameInputEl?.select();
	});
</script>

<div
	class="cmd-backdrop"
	role="dialog"
	aria-modal="true"
	aria-label={$t('tag_rename_dialog')}
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
			<input
				bind:this={nameInputEl}
				class="cmd-input"
				type="text"
				{value}
				placeholder={$t('tag_rename_placeholder')}
				oninput={(event) => onValueChange(event.currentTarget.value)}
				onkeydown={(event) => {
					if (event.key === 'Enter' && value.trim()) onSubmit();
				}}
			/>
		</div>
		<div class="cmd-controls">
			<button type="button" class="cmd-secondary" onclick={onClose}>{$t('common_cancel')}</button>
			<button type="button" class="cmd-action" disabled={!value.trim()} onclick={onSubmit}
				>{$t('common_save')}</button
			>
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
		padding: 0 16px;
		box-sizing: border-box;
	}
	.cmd-controls {
		padding: 10px 16px 14px;
		display: flex;
		align-items: center;
		gap: 6px;
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
