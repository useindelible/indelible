<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import { t } from '$lib/i18n';

	interface Props {
		open: boolean;
		busy?: boolean;
		errorMessage?: string | null;
		onConfirm: () => void;
		onCancel: () => void;
	}

	let { open, busy = false, errorMessage = null, onConfirm, onCancel }: Props = $props();

	function backdropClick(event: MouseEvent) {
		if (event.target === event.currentTarget && !busy) {
			onCancel();
		}
	}

	function onKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape' && !busy) {
			onCancel();
		}
	}
</script>

{#if open}
	<div
		class="backdrop"
		role="presentation"
		onclick={backdropClick}
		onkeydown={onKeyDown}
		data-testid="rollback-dialog"
	>
		<div class="dialog" role="dialog" aria-modal="true" aria-labelledby="rollback-dialog-title">
			<h2 id="rollback-dialog-title" class="title">{$t('imports_rollback_question')}</h2>
			<p class="body">{$t('imports_rollback_description')}</p>
			{#if errorMessage}
				<p class="error" role="alert">{errorMessage}</p>
			{/if}
			<div class="actions">
				<Button variant="tertiary" size="sm" onclick={onCancel} disabled={busy}
					>{$t('common_cancel')}</Button
				>
				<Button variant="destructive" size="sm" loading={busy} onclick={onConfirm}>
					{$t('imports_rollback')}
				</Button>
			</div>
		</div>
	</div>
{/if}

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		background: var(--overlay-backdrop);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 100;
		padding: 16px;
	}

	.dialog {
		background: var(--bg-elevated);
		border: 0.5px solid var(--border-primary);
		border-radius: 12px;
		padding: 20px;
		max-width: 420px;
		width: 100%;
		display: flex;
		flex-direction: column;
		gap: 12px;
		box-shadow: var(--shadow-3);
	}

	.title {
		font-family: var(--font-sans);
		font-size: 16px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		margin: 0;
	}

	.body {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-secondary);
		margin: 0;
		line-height: 1.4;
	}

	.error {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--destructive);
		margin: 0;
	}

	.actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 4px;
	}
</style>
