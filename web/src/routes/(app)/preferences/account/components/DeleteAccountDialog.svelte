<script lang="ts">
	import { t } from '$lib/i18n';

	interface Props {
		email: string;
		confirmEmail: string;
		deleteEmailMatches: boolean;
		deleting: boolean;
		error: string;
		onClose: () => void;
		onConfirmEmailChange: (value: string) => void;
		onDelete: () => void;
	}

	let {
		email,
		confirmEmail,
		deleteEmailMatches,
		deleting,
		error,
		onClose,
		onConfirmEmailChange,
		onDelete
	}: Props = $props();
</script>

<button
	type="button"
	class="modal-backdrop"
	aria-label={$t('common_close')}
	onclick={(event) => {
		if (event.currentTarget === event.target) onClose();
	}}
></button>
<div class="modal-wrap" role="dialog" aria-modal="true" aria-labelledby="delete-title">
	<div class="modal">
		<div class="modal-icon">
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<path d="M3 6h18" />
				<path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
				<path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
			</svg>
		</div>
		<h2 id="delete-title">{$t('account_delete_title')}</h2>
		<p>
			{$t('account_delete_dialog_body', {
				values: { account: email || $t('account_your_account_lower') }
			})}
		</p>
		<p>{$t('account_delete_confirm_hint')}</p>
		<input
			class="input modal-input"
			type="email"
			value={confirmEmail}
			placeholder={email || $t('account_email_placeholder')}
			autocomplete="off"
			aria-label={$t('account_delete_confirm_aria')}
			oninput={(event) => onConfirmEmailChange((event.currentTarget as HTMLInputElement).value)}
		/>
		{#if error}
			<p class="modal-error">{error}</p>
		{/if}
		<div class="modal-actions">
			<button type="button" class="btn ghost" onclick={onClose} disabled={deleting}
				>{$t('common_cancel')}</button
			>
			<button
				type="button"
				class="btn danger"
				onclick={onDelete}
				disabled={!deleteEmailMatches || deleting}
			>
				{deleting ? $t('account_deleting') : $t('account_delete_forever')}
			</button>
		</div>
	</div>

	<style>
		.modal-backdrop {
			position: fixed;
			inset: 0;
			background: var(--overlay-backdrop);
			backdrop-filter: blur(6px);
			-webkit-backdrop-filter: blur(6px);
			border: none;
			padding: 0;
			cursor: pointer;
			z-index: 100;
		}

		.modal-wrap {
			position: fixed;
			inset: 0;
			display: flex;
			align-items: center;
			justify-content: center;
			z-index: 101;
			pointer-events: none;
			padding: 16px;
		}

		.modal {
			pointer-events: auto;
			background: var(--bg-primary);
			color: var(--text-primary);
			border-radius: 16px;
			box-shadow:
				0 24px 60px rgba(0, 0, 0, 0.3),
				0 0 0 0.5px var(--border-primary);
			width: 460px;
			max-width: 100%;
			padding: 24px;
			box-sizing: border-box;
		}

		.modal-icon {
			width: 44px;
			height: 44px;
			border-radius: 12px;
			background: var(--destructive-soft);
			color: var(--destructive);
			display: flex;
			align-items: center;
			justify-content: center;
			margin-bottom: 16px;
			box-shadow: inset 0 0 0 0.5px var(--destructive-border);
		}

		.modal-icon svg {
			width: 22px;
			height: 22px;
			stroke: currentColor;
			fill: none;
			stroke-width: 1.7;
			stroke-linecap: round;
			stroke-linejoin: round;
		}

		.modal h2 {
			font-family: var(--font-sans);
			font-size: 19px;
			font-weight: 700;
			letter-spacing: -0.025em;
			color: var(--text-primary);
			margin: 0 0 8px;
		}

		.modal p {
			font-family: var(--font-sans);
			font-size: 13.5px;
			color: var(--text-secondary);
			line-height: 1.5;
			margin: 0 0 14px;
		}

		.modal p strong {
			color: var(--text-primary);
			font-weight: 600;
		}

		.input {
			background: var(--input-bg);
			color: var(--text-primary);
			border: none;
			outline: none;
			border-radius: 8px;
			padding: 8px 12px;
			font-family: var(--font-sans);
			font-size: 13.5px;
			letter-spacing: -0.01em;
			box-shadow:
				var(--input-shadow),
				0 1px 0 rgba(0, 0, 0, 0.02);
			transition: box-shadow 120ms ease;
			width: 100%;
			box-sizing: border-box;
		}

		.input:focus {
			box-shadow:
				var(--input-shadow),
				0 0 0 3px var(--accent-soft);
		}

		.input::placeholder {
			color: var(--text-tertiary);
		}

		.modal-input {
			margin-bottom: 16px;
		}

		.modal-error {
			font-family: var(--font-sans);
			font-size: 13px;
			color: var(--destructive);
			margin: 0 0 12px;
		}

		.modal-actions {
			display: flex;
			justify-content: flex-end;
			gap: 8px;
		}

		.btn {
			display: inline-flex;
			align-items: center;
			justify-content: center;
			gap: 6px;
			padding: 7px 14px;
			border-radius: 8px;
			border: none;
			font-family: var(--font-sans);
			font-size: 13px;
			font-weight: 500;
			letter-spacing: -0.01em;
			cursor: pointer;
			transition:
				background 120ms ease,
				transform 120ms ease,
				opacity 120ms ease;
			white-space: nowrap;
		}

		.btn:active:not(:disabled) {
			opacity: 0.7;
		}

		.btn.ghost {
			background: transparent;
			color: var(--text-primary);
			box-shadow: inset 0 0 0 0.5px var(--border-primary);
		}

		.btn.ghost:hover:not(:disabled) {
			background: var(--fill-hover);
		}

		.btn.danger {
			background: var(--destructive);
			color: var(--text-on-color);
		}

		.btn.danger:hover:not(:disabled) {
			opacity: 0.92;
		}

		.btn:disabled {
			opacity: 0.45;
			cursor: default;
		}
	</style>
</div>
