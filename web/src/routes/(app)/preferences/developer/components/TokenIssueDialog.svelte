<script lang="ts">
	import PermissionLedger from './PermissionLedger.svelte';
	import {
		formatDate,
		type ExpiryOption,
		type PermissionKey,
		type ResourceAccessLevel,
		type ResourcePermissionKey
	} from '../developer-model';
	import { t } from '$lib/i18n';

	interface Props {
		open: boolean;
		name: string;
		permissions: Set<PermissionKey>;
		expiry: ExpiryOption;
		allPermissionsSelected: boolean;
		creating: boolean;
		error: string | null;
		onClose: () => void;
		onName: (name: string) => void;
		onSetResourceAccess: (resource: ResourcePermissionKey, level: ResourceAccessLevel) => void;
		onTogglePermission: (permission: PermissionKey) => void;
		onToggleAllPermissions: () => void;
		onExpiry: (expiry: ExpiryOption) => void;
		onSubmit: () => void;
	}

	let {
		open,
		name,
		permissions,
		expiry,
		allPermissionsSelected,
		creating,
		error,
		onClose,
		onName,
		onSetResourceAccess,
		onTogglePermission,
		onToggleAllPermissions,
		onExpiry,
		onSubmit
	}: Props = $props();

	const revokesOn = $derived(
		expiry === 'never'
			? null
			: formatDate(new Date(Date.now() + Number(expiry) * 86_400_000).toISOString())
	);
</script>

<div class="issue-form" class:open inert={!open} aria-hidden={!open}>
	<div class="issue-form-inner">
		<div class="issue-form-head">
			<div class="ifh-title">{$t('prefs_developer_issue_new_token')}</div>
			<button type="button" class="close" onclick={onClose} aria-label={$t('common_close')}>
				<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6l-12 12" /></svg>
			</button>
		</div>

		<div class="form-row">
			<label class="lab" for="dev-token-name">
				{$t('prefs_developer_name')}<span class="help">{$t('prefs_developer_name_hint')}</span>
			</label>
			<input
				id="dev-token-name"
				class="input"
				type="text"
				placeholder={$t('prefs_developer_name_placeholder')}
				value={name}
				oninput={(event) => onName(event.currentTarget.value)}
			/>
		</div>

		<div class="form-row">
			<div class="lab">
				{$t('prefs_developer_permissions')}<span class="help"
					>{$t('prefs_developer_permissions_hint')}</span
				>
			</div>
			<PermissionLedger
				{permissions}
				{allPermissionsSelected}
				{onSetResourceAccess}
				{onTogglePermission}
				{onToggleAllPermissions}
			/>
		</div>

		<div class="form-row">
			<label class="lab" for="dev-token-expiry">
				{$t('prefs_developer_expiry')}<span class="help">{$t('prefs_developer_expiry_hint')}</span>
			</label>
			<div class="expiry">
				<select
					id="dev-token-expiry"
					class="select"
					value={expiry}
					onchange={(event) => onExpiry(event.currentTarget.value as ExpiryOption)}
				>
					<option value="30">{$t('prefs_developer_30_days')}</option>
					<option value="90">{$t('prefs_developer_90_days')}</option>
					<option value="365">{$t('prefs_developer_1_year')}</option>
					<option value="never">{$t('prefs_developer_no_expiry')}</option>
				</select>
				<span class="resolved">
					{#if revokesOn}
						{$t('prefs_developer_revokes_on', { values: { date: revokesOn } })}
					{:else}
						{$t('prefs_developer_valid_until_revoked')}
					{/if}
				</span>
			</div>
		</div>

		{#if error}
			<div class="form-error" role="alert">{error}</div>
		{/if}

		<div class="form-foot">
			<button type="button" class="btn ghost" onclick={onClose}>{$t('common_cancel')}</button>
			<button type="button" class="btn primary" disabled={creating} onclick={onSubmit}>
				{creating ? $t('prefs_developer_creating') : $t('prefs_developer_create_token')}
			</button>
		</div>
	</div>
</div>

<style>
	/* Expands on grid-template-rows, not max-height: the form grows past any
	   fixed ceiling once every permission row is on screen. */
	.issue-form {
		background: var(--dev-card-strong);
		border-radius: 14px;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		margin-top: 12px;
		display: grid;
		grid-template-rows: 0fr;
		opacity: 0;
		transition:
			grid-template-rows 340ms cubic-bezier(0.2, 0.7, 0.3, 1),
			opacity 240ms ease;
	}

	.issue-form.open {
		grid-template-rows: 1fr;
		opacity: 1;
	}

	.issue-form-inner {
		min-height: 0;
		overflow: hidden;
		padding: 22px 24px 24px;
		display: flex;
		flex-direction: column;
		gap: 22px;
	}

	.issue-form-head,
	.form-foot {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.ifh-title {
		font-size: 14px;
		font-weight: 600;
		color: var(--text-primary);
		letter-spacing: -0.015em;
	}

	.close {
		width: 24px;
		height: 24px;
		border-radius: 6px;
		color: var(--text-tertiary);
		background: none;
		border: none;
		cursor: pointer;
		display: inline-flex;
		align-items: center;
		justify-content: center;
	}

	.close:hover {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.close svg {
		width: 14px;
		height: 14px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.6;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.form-row {
		display: grid;
		grid-template-columns: 140px minmax(0, 1fr);
		gap: 18px;
		align-items: flex-start;
	}

	.lab {
		font-size: 12.5px;
		font-weight: 500;
		color: var(--text-primary);
		padding-top: 10px;
	}

	.help {
		display: block;
		font-size: 11.5px;
		color: var(--text-tertiary);
		margin-top: 2px;
		font-weight: 400;
		letter-spacing: -0.005em;
	}

	.input,
	.select {
		background: var(--bg-elevated);
		color: var(--text-primary);
		border: none;
		outline: none;
		border-radius: 8px;
		padding: 8px 12px;
		font-size: 13.5px;
		letter-spacing: -0.01em;
		box-shadow:
			inset 0 0 0 0.5px var(--border-primary),
			0 1px 0 rgba(0, 0, 0, 0.02);
		width: 100%;
		font-family: inherit;
	}

	.input:focus,
	.select:focus {
		box-shadow:
			inset 0 0 0 0.5px var(--border-primary),
			0 0 0 3px var(--dev-accent-soft);
	}

	.select {
		appearance: none;
		-webkit-appearance: none;
		padding-right: 32px;
		background-image: url("data:image/svg+xml,%3Csvg width='10' height='6' viewBox='0 0 10 6' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%237E8AA0' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
		background-repeat: no-repeat;
		background-position: right 10px center;
	}

	.expiry {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 12px;
	}

	.expiry .select {
		max-width: 200px;
	}

	.resolved {
		font-size: 11.5px;
		color: var(--text-tertiary);
		letter-spacing: -0.005em;
	}

	.form-error {
		font-size: 12.5px;
		color: var(--destructive);
		padding: 8px 12px;
		border-radius: 8px;
		background: var(--dev-destructive-soft);
	}

	.form-foot {
		justify-content: flex-end;
		gap: 8px;
		padding: 16px 0 0;
		border-top: 0.5px solid var(--border-hairline);
	}

	.btn {
		border: none;
		border-radius: 8px;
		padding: 7px 14px;
		font: inherit;
		font-size: 12.5px;
		font-weight: 500;
		cursor: pointer;
		letter-spacing: -0.01em;
	}

	.btn.ghost {
		background: transparent;
		color: var(--text-primary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.btn.primary {
		background: var(--dev-accent);
		color: var(--text-on-color);
	}

	@media (max-width: 720px) {
		.form-row {
			grid-template-columns: 1fr;
			gap: 6px;
		}

		.lab {
			padding-top: 0;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.issue-form {
			transition: opacity 240ms ease;
		}
	}
</style>
