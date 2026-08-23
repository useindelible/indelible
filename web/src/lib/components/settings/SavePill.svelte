<script lang="ts">
	import { t } from '$lib/i18n';

	interface Props {
		isDirty: boolean;
		saving: boolean;
		showSaved: boolean;
		saveDisabled?: boolean;
		onSave: () => void;
		onDiscard: () => void;
	}

	let { isDirty, saving, showSaved, saveDisabled = false, onSave, onDiscard }: Props = $props();
</script>

<div class="save-pill-anchor">
	<div class="save-pill" class:visible={isDirty || showSaved}>
		{#if showSaved && !isDirty}
			<span class="pill-saved">
				<svg width="13" height="13" viewBox="0 0 14 14" fill="none" aria-hidden="true">
					<path
						d="M2.5 7.5L5.5 10.5L11.5 3.5"
						stroke="currentColor"
						stroke-width="1.75"
						stroke-linecap="round"
						stroke-linejoin="round"
					/>
				</svg>
				{$t('common_saved')}
			</span>
		{:else}
			<button type="button" class="pill-btn discard" onclick={onDiscard} disabled={saving}
				>{$t('common_discard')}</button
			>
			<div class="pill-divider"></div>
			<button
				type="button"
				class="pill-btn save"
				onclick={onSave}
				disabled={saving || saveDisabled}
			>
				{saving ? $t('common_saving') : $t('common_save')}
			</button>
		{/if}
	</div>
</div>

<style>
	.save-pill-anchor {
		position: fixed;
		right: 0;
		bottom: 20px;
		width: 100cqw;
		display: flex;
		justify-content: center;
		pointer-events: none;
		z-index: 10;
	}

	.save-pill {
		display: inline-flex;
		align-items: center;
		gap: 0;
		padding: 4px;
		border-radius: 980px;
		background: var(--bg-primary);
		box-shadow:
			0 2px 12px rgba(0, 0, 0, 0.08),
			0 0 0 0.5px rgba(0, 0, 0, 0.06);
		pointer-events: none;
		visibility: hidden;
		opacity: 0;
		transform: translateY(8px) scale(0.96);
		transition:
			opacity 250ms ease,
			transform 250ms ease;
	}

	:global([data-theme='dark']) .save-pill {
		box-shadow:
			0 2px 16px rgba(0, 0, 0, 0.3),
			0 0 0 0.5px rgba(255, 255, 255, 0.06);
	}

	.save-pill.visible {
		pointer-events: auto;
		visibility: visible;
		opacity: 1;
		transform: translateY(0) scale(1);
	}

	.pill-btn {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		letter-spacing: -0.01em;
		border: none;
		border-radius: 980px;
		cursor: pointer;
		padding: 6px 16px;
		transition:
			background 120ms ease,
			opacity 120ms ease;
	}

	.pill-btn:active:not(:disabled) {
		opacity: 0.7;
	}

	.pill-btn:disabled {
		opacity: 0.45;
		cursor: default;
	}

	.pill-btn.discard {
		background: transparent;
		color: var(--text-secondary);
	}

	.pill-btn.discard:hover:not(:disabled) {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.pill-btn.save {
		background: var(--accent);
		color: var(--text-on-color);
	}

	.pill-btn.save:hover:not(:disabled) {
		opacity: 0.9;
	}

	.pill-divider {
		width: 0.5px;
		height: 16px;
		background: var(--border-primary);
		flex-shrink: 0;
	}

	.pill-saved {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 6px 18px;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		letter-spacing: -0.01em;
		color: var(--success);
	}
</style>
