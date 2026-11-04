<script lang="ts">
	import { tick } from 'svelte';
	import { t } from '$lib/i18n';
	import { registerShortcuts } from '$lib/shortcuts/registry.svelte';
	import ShortcutTable from './ShortcutTable.svelte';

	let { onClose }: { onClose: () => void } = $props();

	let dialogEl: HTMLDivElement | undefined = $state();

	const FOCUSABLE = 'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])';

	// Mounted only while open, so the layer lives exactly as long as the overlay.
	// Escape reaches it only once focus has left the dialog; the backdrop handles the rest.
	registerShortcuts(() => ({ scope: 'modal', handlers: { dismiss: onClose }, exclusive: true }));

	$effect(() => {
		const opener = document.activeElement;
		void tick().then(() => dialogEl?.focus());
		return () => {
			if (opener instanceof HTMLElement && opener.isConnected) opener.focus();
		};
	});

	function handleBackdropClick(event: MouseEvent) {
		if (event.target === event.currentTarget) onClose();
	}

	function containTab(event: KeyboardEvent) {
		if (!dialogEl) return;
		const focusable = [...dialogEl.querySelectorAll<HTMLElement>(FOCUSABLE)];
		const first = focusable[0];
		const last = focusable[focusable.length - 1];
		if (!first || !last) {
			event.preventDefault();
			return;
		}
		const active = document.activeElement;
		if (event.shiftKey && (active === first || active === dialogEl)) {
			event.preventDefault();
			last.focus();
		} else if (!event.shiftKey && (active === last || active === dialogEl)) {
			event.preventDefault();
			first.focus();
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') onClose();
		else if (event.key === 'Tab') containTab(event);
	}
</script>

<div
	class="help-backdrop"
	role="presentation"
	onclick={handleBackdropClick}
	onkeydown={handleKeydown}
>
	<div
		bind:this={dialogEl}
		class="help-dialog"
		role="dialog"
		aria-modal="true"
		aria-labelledby="shortcut-help-title"
		tabindex="-1"
	>
		<div class="help-header">
			<h2 id="shortcut-help-title">{$t('prefs_reading_shortcuts')}</h2>
			<button type="button" class="help-close" onclick={onClose} aria-label={$t('common_close')}>
				<svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
					<path
						d="M3 3l8 8M11 3l-8 8"
						stroke="currentColor"
						stroke-width="1.5"
						stroke-linecap="round"
					/>
				</svg>
			</button>
		</div>
		<ShortcutTable />
	</div>
</div>

<style>
	.help-backdrop {
		position: fixed;
		inset: 0;
		z-index: 300;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 16px;
		background: var(--overlay-backdrop);
	}

	.help-dialog {
		width: min(100%, 760px);
		max-height: calc(100vh - 32px);
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 14px;
		padding: 20px;
		background: var(--bg-elevated);
		border: 0.5px solid var(--border-primary);
		border-radius: 8px;
		box-shadow: var(--shadow-3);
	}

	.help-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	h2 {
		margin: 0;
		font-family: var(--font-sans);
		font-size: 16px;
		font-weight: 600;
		color: var(--text-primary);
		line-height: 1.3;
	}

	.help-close {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
	}

	.help-close:hover {
		background: var(--fill-hover);
		color: var(--text-primary);
	}
</style>
