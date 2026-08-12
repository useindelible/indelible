<script lang="ts">
	import {
		INDEPENDENT_PERMISSION_DEFS,
		RESOURCE_PERMISSION_GROUPS,
		resourceAccessLevel,
		type ExpiryOption,
		type PermissionKey,
		type ResourceAccessLevel,
		type ResourcePermissionKey
	} from '../developer-model';

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
</script>

<div class="issue-form" class:open inert={!open} aria-hidden={!open}>
	<div class="issue-form-inner">
		<div class="issue-form-head">
			<div class="ifh-title">Issue a new token</div>
			<button type="button" class="close" onclick={onClose} aria-label="Close">
				<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6l-12 12" /></svg>
			</button>
		</div>

		<div class="form-row">
			<label class="lab" for="dev-token-name">
				Name<span class="help">A descriptive label so you know what it's for.</span>
			</label>
			<input
				id="dev-token-name"
				class="input"
				type="text"
				placeholder="e.g. Personal MacBook"
				value={name}
				oninput={(event) => onName(event.currentTarget.value)}
			/>
		</div>

		<div class="form-row">
			<div class="lab">
				Permissions<span class="help">Start with none, then grant only what this token needs.</span>
			</div>
			<div class="permission-picker">
				<div class="permission-toolbar">
					<span>{permissions.size} selected</span>
					<button type="button" class="select-all" onclick={onToggleAllPermissions}>
						{allPermissionsSelected ? 'Clear all' : 'Select all'}
					</button>
				</div>

				<div class="resource-list">
					{#each RESOURCE_PERMISSION_GROUPS as group (group.key)}
						<div class="resource-row">
							<div class="resource-copy">
								<div class="resource-name">{group.label}</div>
								<div class="desc">{group.desc}</div>
							</div>
							<div class="level-picker" role="group" aria-label={`${group.label} access`}>
								<button
									type="button"
									class:selected={resourceAccessLevel(permissions, group.key) === 'read'}
									aria-pressed={resourceAccessLevel(permissions, group.key) === 'read'}
									onclick={() =>
										onSetResourceAccess(
											group.key,
											resourceAccessLevel(permissions, group.key) === 'read' ? 'none' : 'read'
										)}
								>
									Read
								</button>
								<button
									type="button"
									class:selected={resourceAccessLevel(permissions, group.key) === 'write'}
									aria-pressed={resourceAccessLevel(permissions, group.key) === 'write'}
									onclick={() =>
										onSetResourceAccess(
											group.key,
											resourceAccessLevel(permissions, group.key) === 'write' ? 'none' : 'write'
										)}
								>
									Read + write
								</button>
							</div>
						</div>
					{/each}
				</div>

				<div class="permission-grid">
					{#each INDEPENDENT_PERMISSION_DEFS as permission (permission.key)}
						<button
							type="button"
							class="permission-card"
							class:selected={permissions.has(permission.key)}
							aria-pressed={permissions.has(permission.key)}
							onclick={() => onTogglePermission(permission.key)}
						>
							<div class="permission-head">
								<span class="permission-label">{permission.label}</span>
								<span class="check">
									<svg viewBox="0 0 24 24" aria-hidden="true">
										<polyline points="20 6 9 17 4 12" />
									</svg>
								</span>
							</div>
							<span class="key">{permission.key}</span>
							<div class="desc">{permission.desc}</div>
						</button>
					{/each}
				</div>
			</div>
		</div>

		<div class="form-row">
			<label class="lab" for="dev-token-expiry">
				Expiry<span class="help">Token auto-revokes at this time.</span>
			</label>
			<select
				id="dev-token-expiry"
				class="select expiry-select"
				value={expiry}
				onchange={(event) => onExpiry(event.currentTarget.value as ExpiryOption)}
			>
				<option value="30">30 days</option>
				<option value="90">90 days</option>
				<option value="365">1 year</option>
				<option value="never">No expiry</option>
			</select>
		</div>

		{#if error}
			<div class="form-error" role="alert">{error}</div>
		{/if}

		<div class="form-foot">
			<button type="button" class="btn ghost" onclick={onClose}>Cancel</button>
			<button type="button" class="btn primary" disabled={creating} onclick={onSubmit}>
				{creating ? 'Creating…' : 'Create token'}
			</button>
		</div>
	</div>
</div>

<style>
	.issue-form {
		background: var(--dev-card-strong);
		border-radius: 14px;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		margin-top: 12px;
		overflow: hidden;
		max-height: 0;
		opacity: 0;
		transition:
			max-height 320ms ease,
			opacity 240ms ease;
	}

	.issue-form.open {
		max-height: 1000px;
		opacity: 1;
	}

	.issue-form-inner {
		padding: 22px 24px 24px;
		display: flex;
		flex-direction: column;
		gap: 22px;
	}

	.issue-form-head,
	.form-foot,
	.permission-head {
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

	.expiry-select {
		max-width: 240px;
	}

	.permission-picker {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.permission-toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		font-size: 11.5px;
		color: var(--text-tertiary);
	}

	.select-all {
		border: none;
		background: none;
		color: var(--dev-accent);
		font: inherit;
		font-weight: 600;
		cursor: pointer;
		padding: 3px 0;
	}

	.resource-list {
		border-radius: 10px;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		overflow: hidden;
	}

	.resource-row {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		gap: 16px;
		align-items: center;
		padding: 11px 12px;
		background: var(--bg-elevated);
		border-bottom: 0.5px solid var(--border-hairline);
	}

	.resource-row:last-child {
		border-bottom: none;
	}

	.resource-name,
	.permission-label {
		font-size: 12.5px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.level-picker {
		display: inline-flex;
		padding: 2px;
		border-radius: 8px;
		background: var(--fill-hover);
	}

	.level-picker button {
		border: none;
		background: transparent;
		color: var(--text-secondary);
		font: inherit;
		font-size: 11.5px;
		font-weight: 500;
		padding: 5px 8px;
		border-radius: 6px;
		cursor: pointer;
	}

	.level-picker button.selected {
		background: var(--bg-elevated);
		color: var(--dev-accent);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
	}

	.permission-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 12px;
	}

	.permission-card {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 10px 12px;
		background: var(--bg-elevated);
		border-radius: 10px;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		cursor: pointer;
		text-align: left;
		font: inherit;
		color: inherit;
		border: none;
	}

	.permission-card.selected {
		box-shadow:
			inset 0 0 0 1.5px var(--dev-accent),
			0 0 0 4px var(--dev-accent-soft);
		background: var(--dev-accent-soft);
	}

	.key {
		font-family: 'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace;
		font-size: 12px;
		font-weight: 700;
		letter-spacing: 0;
		color: var(--text-primary);
	}

	.check {
		width: 16px;
		height: 16px;
		border-radius: 50%;
		background: var(--dev-accent);
		color: var(--text-on-color);
		display: none;
		align-items: center;
		justify-content: center;
	}

	.check svg {
		width: 9px;
		height: 9px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.permission-card.selected .check {
		display: inline-flex;
	}

	.desc {
		font-size: 11.5px;
		color: var(--text-secondary);
		line-height: 1.4;
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
		.form-row,
		.permission-grid,
		.resource-row {
			grid-template-columns: 1fr;
		}

		.level-picker {
			width: 100%;
		}

		.level-picker button {
			flex: 1;
		}
	}
</style>
