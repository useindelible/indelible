<script lang="ts">
	import { tick } from 'svelte';
	import type { Snippet } from 'svelte';
	import Button from '$lib/components/ui/Button.svelte';

	type Variant = 'primary' | 'destructive';

	interface Props {
		open: boolean;
		title: string;
		message?: string;
		confirmLabel?: string;
		cancelLabel?: string;
		variant?: Variant;
		busy?: boolean;
		errorMessage?: string | null;
		onConfirm: () => void;
		onCancel: () => void;
		children?: Snippet;
		actions?: Snippet;
	}

	let {
		open,
		title,
		message,
		confirmLabel = 'Confirm',
		cancelLabel = 'Cancel',
		variant = 'destructive',
		busy = false,
		errorMessage = null,
		onConfirm,
		onCancel,
		children,
		actions
	}: Props = $props();

	let dialogEl: HTMLDivElement | undefined = $state();

	$effect(() => {
		if (!open) return;
		void tick().then(() => dialogEl?.focus());
	});

	function handleBackdropClick(event: MouseEvent) {
		if (event.target === event.currentTarget && !busy) {
			onCancel();
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape' && !busy) {
			onCancel();
		}
	}

	function handleCancel() {
		if (!busy) {
			onCancel();
		}
	}

	function handleConfirm() {
		if (!busy) {
			onConfirm();
		}
	}
</script>

{#if open}
	<div
		class="confirm-backdrop"
		role="presentation"
		data-testid="confirm-dialog-backdrop"
		onclick={handleBackdropClick}
		onkeydown={handleKeydown}
	>
		<div
			bind:this={dialogEl}
			class="confirm-dialog"
			role="dialog"
			aria-modal="true"
			aria-labelledby="confirm-title"
			tabindex="-1"
		>
			<div class="confirm-copy">
				<h2 id="confirm-title">{title}</h2>
				{#if message}
					<p>{message}</p>
				{/if}
			</div>

			{@render children?.()}

			{#if errorMessage}
				<p class="confirm-error" role="alert">{errorMessage}</p>
			{/if}

			<div class="confirm-actions">
				{#if actions}
					{@render actions()}
				{:else}
					<Button variant="tertiary" size="sm" onclick={handleCancel} disabled={busy}>
						{cancelLabel}
					</Button>
					<Button {variant} size="sm" onclick={handleConfirm} loading={busy}>
						{confirmLabel}
					</Button>
				{/if}
			</div>
		</div>
	</div>
{/if}

<style>
	.confirm-backdrop {
		position: fixed;
		inset: 0;
		z-index: 300;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 16px;
		background: var(--overlay-backdrop);
	}

	.confirm-dialog {
		width: min(100%, 420px);
		display: flex;
		flex-direction: column;
		gap: 14px;
		padding: 20px;
		background: var(--bg-elevated);
		border: 0.5px solid var(--border-primary);
		border-radius: 8px;
		box-shadow: var(--shadow-3);
	}

	.confirm-copy {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	h2,
	p {
		margin: 0;
	}

	h2 {
		font-family: var(--font-sans);
		font-size: 16px;
		font-weight: 600;
		color: var(--text-primary);
		line-height: 1.3;
	}

	p {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-secondary);
		line-height: 1.45;
	}

	.confirm-error {
		color: var(--destructive);
	}

	.confirm-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 2px;
	}

	@media (max-width: 480px) {
		.confirm-dialog {
			padding: 18px;
		}
	}
</style>
