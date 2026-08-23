<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import { t } from '$lib/i18n';

	interface Props {
		passwordOpen: boolean;
		currentPassword: string;
		newPassword: string;
		confirmPassword: string;
		passwordMismatch: boolean;
		canSubmitPassword: boolean;
		passwordSaving: boolean;
		passwordError: string;
		passwordSuccess: boolean;
		onOpenPassword: () => void;
		onCancelPassword: () => void;
		onCurrentPasswordChange: (value: string) => void;
		onNewPasswordChange: (value: string) => void;
		onConfirmPasswordChange: (value: string) => void;
		onChangePassword: () => void;
	}

	let {
		passwordOpen,
		currentPassword,
		newPassword,
		confirmPassword,
		passwordMismatch,
		canSubmitPassword,
		passwordSaving,
		passwordError,
		passwordSuccess,
		onOpenPassword,
		onCancelPassword,
		onCurrentPasswordChange,
		onNewPasswordChange,
		onConfirmPasswordChange,
		onChangePassword
	}: Props = $props();
</script>

<SettingsGroup title={$t('account_security')}>
	<div class="group-card">
		<div class="row">
			<div class="label-block">
				<div class="label">{$t('auth_password')}</div>
				<div class="hint">{$t('account_password_hint')}</div>
			</div>
			<div>
				<button type="button" class="btn ghost" disabled={passwordOpen} onclick={onOpenPassword}>
					{$t('account_change_password')}
				</button>
			</div>
		</div>
		{#if passwordOpen}
			<div class="row with-stack">
				<div class="label-block">
					<div class="label">{$t('account_set_password')}</div>
					<div class="hint">{$t('account_set_password_hint')}</div>
				</div>
				<div class="input-group input-group--stacked">
					<input
						class="input"
						type="password"
						value={currentPassword}
						placeholder={$t('account_current_password')}
						autocomplete="current-password"
						oninput={(event) =>
							onCurrentPasswordChange((event.currentTarget as HTMLInputElement).value)}
					/>
					<input
						class="input"
						type="password"
						value={newPassword}
						placeholder={$t('auth_new_password')}
						autocomplete="new-password"
						oninput={(event) =>
							onNewPasswordChange((event.currentTarget as HTMLInputElement).value)}
					/>
					<input
						class="input"
						type="password"
						value={confirmPassword}
						placeholder={$t('auth_confirm_password_placeholder')}
						autocomplete="new-password"
						oninput={(event) =>
							onConfirmPasswordChange((event.currentTarget as HTMLInputElement).value)}
					/>
					{#if passwordMismatch}
						<p class="reveal-error" role="alert">{$t('account_passwords_mismatch')}</p>
					{:else if passwordError}
						<p class="reveal-error" role="alert">{passwordError}</p>
					{:else if passwordSuccess}
						<p class="reveal-success" role="status">{$t('account_password_changed')}</p>
					{/if}
					<div class="reveal-actions">
						<button
							type="button"
							class="btn ghost"
							onclick={onCancelPassword}
							disabled={passwordSaving}>{$t('common_cancel')}</button
						>
						<button
							type="button"
							class="btn primary"
							disabled={!canSubmitPassword}
							onclick={onChangePassword}
						>
							{passwordSaving ? $t('account_updating') : $t('account_update_password')}
						</button>
					</div>
				</div>
			</div>
		{/if}
	</div>

	<div class="group-card">
		<div class="session-row">
			<div class="session-icon">
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<rect x="2" y="4" width="20" height="14" rx="2" />
					<path d="M2 18l4 3M22 18l-4 3" />
				</svg>
			</div>
			<div class="session-meta">
				<div class="device">
					{$t('account_this_browser')}
					<span class="current-tag">
						<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12l4 4 10-10" /></svg>
						{$t('account_current')}
					</span>
				</div>
				<div class="where">{$t('account_active_now')}</div>
			</div>
			<button type="button" class="btn ghost" disabled>{$t('account_this_session')}</button>
		</div>
	</div>
</SettingsGroup>
