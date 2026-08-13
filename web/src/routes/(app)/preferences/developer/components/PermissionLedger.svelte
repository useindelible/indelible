<script lang="ts">
	import PermissionChip from './PermissionChip.svelte';
	import {
		INDEPENDENT_PERMISSION_DEFS,
		PERMISSION_CATALOGUE,
		RESOURCE_PERMISSION_GROUPS,
		resourceAccessLevel,
		type PermissionKey,
		type ResourceAccessLevel,
		type ResourcePermissionKey
	} from '../developer-model';

	interface Props {
		permissions: Set<PermissionKey>;
		allPermissionsSelected: boolean;
		onSetResourceAccess: (resource: ResourcePermissionKey, level: ResourceAccessLevel) => void;
		onTogglePermission: (permission: PermissionKey) => void;
		onToggleAllPermissions: () => void;
	}

	let {
		permissions,
		allPermissionsSelected,
		onSetResourceAccess,
		onTogglePermission,
		onToggleAllPermissions
	}: Props = $props();

	const LEVELS: Array<{ value: ResourceAccessLevel; label: string }> = [
		{ value: 'none', label: 'None' },
		{ value: 'read', label: 'Read' },
		{ value: 'write', label: 'Read + write' }
	];

	const granted = $derived(PERMISSION_CATALOGUE.filter((permission) => permissions.has(permission)));
</script>

<div class="perm">
	<div class="perm-bar">
		<span class="summary">
			{#if granted.length}
				<strong>{granted.length}</strong>
				permission{granted.length === 1 ? '' : 's'} granted
			{:else}
				No permissions granted
			{/if}
		</span>
		<button type="button" class="perm-all" onclick={onToggleAllPermissions}>
			{allPermissionsSelected ? 'Clear all' : 'Grant everything'}
		</button>
	</div>

	<div class="ledger">
		<div class="ledger-head">
			<span>Resources</span>
			<span class="hint">Choose a level</span>
		</div>

		{#each RESOURCE_PERMISSION_GROUPS as group (group.key)}
			{@const level = resourceAccessLevel(permissions, group.key)}
			<div class="res" data-level={level}>
				<div class="res-copy">
					<div class="n">{group.label}</div>
					<div class="d">{group.desc}</div>
				</div>
				<div class="levels" role="radiogroup" aria-label={`${group.label} access`}>
					<span class="thumb" aria-hidden="true"></span>
					{#each LEVELS as option (option.value)}
						<label class="level">
							<input
								type="radio"
								name={`token-level-${group.key}`}
								value={option.value}
								checked={level === option.value}
								onchange={() => onSetResourceAccess(group.key, option.value)}
							/>
							<span>{option.label}</span>
						</label>
					{/each}
				</div>
			</div>
		{/each}

		<div class="ledger-head second">
			<span>Capabilities</span>
			<span class="hint">On or off</span>
		</div>

		{#each INDEPENDENT_PERMISSION_DEFS as permission (permission.key)}
			{@const implied = permission.key === 'ai:read' && permissions.has('ai:write')}
			<button
				type="button"
				class="cap"
				class:on={permissions.has(permission.key)}
				class:implied
				aria-pressed={permissions.has(permission.key)}
				onclick={() => onTogglePermission(permission.key)}
			>
				<span class="box">
					<svg viewBox="0 0 24 24" aria-hidden="true"><polyline points="20 6 9 17 4 12" /></svg>
				</span>
				<span class="cap-copy">
					<span class="n">{permission.label}<code>{permission.key}</code></span>
					<span class="d">{permission.desc}</span>
				</span>
				<span class="via">Included</span>
			</button>
		{/each}
	</div>

	<div class="manifest">
		<div class="manifest-bar">
			<span class="k">permissions</span>
			<span>granted to this token</span>
			<span class="n">{granted.length}</span>
		</div>
		<div class="manifest-body">
			{#each granted as permission (permission)}
				<PermissionChip {permission} />
			{:else}
				<span class="manifest-empty">Nothing granted yet — this token can't call the API.</span>
			{/each}
		</div>
	</div>
</div>

<style>
	.perm {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.perm-bar {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
	}

	.summary {
		font-size: 11.5px;
		color: var(--text-tertiary);
		letter-spacing: -0.005em;
	}

	.summary strong {
		color: var(--text-secondary);
		font-weight: 600;
	}

	.perm-all {
		border: none;
		background: none;
		font: inherit;
		font-size: 11.5px;
		font-weight: 600;
		letter-spacing: -0.005em;
		color: var(--dev-accent);
		cursor: pointer;
		padding: 2px 0;
	}

	.perm-all:hover {
		text-decoration: underline;
	}

	.ledger {
		border-radius: 10px;
		overflow: hidden;
		background: var(--bg-elevated);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.ledger-head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
		padding: 8px 12px 7px;
		background: var(--dev-card-bg);
		border-bottom: 0.5px solid var(--border-hairline);
		font-size: 10px;
		font-weight: 700;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--text-tertiary);
	}

	.ledger-head.second {
		border-top: 0.5px solid var(--border-primary);
	}

	.hint {
		font-size: 10.5px;
		font-weight: 500;
		letter-spacing: 0;
		text-transform: none;
		color: var(--dev-text-quaternary);
	}

	.res {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		gap: 16px;
		align-items: center;
		padding: 11px 12px;
		border-bottom: 0.5px solid var(--border-hairline);
	}

	.n {
		font-size: 12.5px;
		font-weight: 600;
		color: var(--text-primary);
		letter-spacing: -0.01em;
	}

	.d {
		font-size: 11.5px;
		color: var(--text-secondary);
		line-height: 1.4;
		margin-top: 1px;
	}

	.levels {
		position: relative;
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		width: 272px;
		padding: 2px;
		border-radius: 8px;
		background: var(--fill-hover);
	}

	.thumb {
		position: absolute;
		top: 2px;
		bottom: 2px;
		left: 2px;
		width: calc((100% - 4px) / 3);
		border-radius: 6px;
		background: var(--bg-elevated);
		box-shadow:
			0 1px 3px rgba(0, 0, 0, 0.1),
			0 0 0 0.5px var(--border-primary);
		pointer-events: none;
		transition:
			transform 240ms cubic-bezier(0.2, 0.7, 0.3, 1),
			background 200ms ease,
			box-shadow 200ms ease;
	}

	.res[data-level='read'] .thumb {
		transform: translateX(100%);
		background: var(--dev-scope-read-bg);
		box-shadow: inset 0 0 0 0.5px var(--dev-scope-read-fg);
	}

	.res[data-level='write'] .thumb {
		transform: translateX(200%);
		background: var(--dev-scope-write-bg);
		box-shadow: inset 0 0 0 0.5px var(--dev-scope-write-fg);
	}

	.level {
		position: relative;
		z-index: 1;
		display: block;
		padding: 5px 4px;
		border-radius: 6px;
		text-align: center;
		cursor: pointer;
	}

	.level input {
		position: absolute;
		inset: 0;
		opacity: 0;
		cursor: pointer;
		margin: 0;
	}

	.level span {
		display: block;
		border-radius: 6px;
		font-size: 11.5px;
		font-weight: 500;
		letter-spacing: -0.005em;
		white-space: nowrap;
		color: var(--text-tertiary);
		transition: color 160ms ease;
	}

	.level:hover span {
		color: var(--text-secondary);
	}

	.level input:focus-visible + span {
		box-shadow: 0 0 0 2px var(--dev-accent);
	}

	.res[data-level='none'] .level:first-of-type span {
		color: var(--text-secondary);
		font-weight: 600;
	}

	.res[data-level='read'] .level:nth-of-type(2) span {
		color: var(--dev-scope-read-fg);
		font-weight: 600;
	}

	.res[data-level='write'] .level:last-of-type span {
		color: var(--dev-scope-write-fg);
		font-weight: 600;
	}

	.cap {
		display: grid;
		grid-template-columns: 16px minmax(0, 1fr) auto;
		gap: 10px;
		align-items: start;
		width: 100%;
		text-align: left;
		padding: 11px 12px;
		border: none;
		border-bottom: 0.5px solid var(--border-hairline);
		background: none;
		font: inherit;
		color: inherit;
		cursor: pointer;
		transition: background 140ms ease;
	}

	.cap:last-child {
		border-bottom: none;
	}

	.cap:hover {
		background: var(--fill-hover);
	}

	.cap-copy {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}

	.box {
		margin-top: 1px;
		width: 16px;
		height: 16px;
		border-radius: 5px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		box-shadow: inset 0 0 0 1.5px var(--border-primary);
		transition:
			background 140ms ease,
			box-shadow 140ms ease;
	}

	.box svg {
		width: 10px;
		height: 10px;
		stroke: var(--text-on-color);
		fill: none;
		stroke-width: 2.6;
		stroke-linecap: round;
		stroke-linejoin: round;
		opacity: 0;
		transition: opacity 120ms ease;
	}

	.cap.on .box {
		background: var(--dev-accent);
		box-shadow: inset 0 0 0 1.5px var(--dev-accent);
	}

	.cap.on .box svg {
		opacity: 1;
	}

	.cap code {
		font-family: 'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace;
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0;
		color: var(--text-tertiary);
		margin-left: 6px;
	}

	/* ai:write carries ai:read with it — say so where it happens, not in prose */
	.via {
		align-self: center;
		font-size: 10px;
		font-weight: 700;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		color: var(--dev-scope-read-fg);
		background: var(--dev-scope-read-bg);
		padding: 3px 7px;
		border-radius: 5px;
		opacity: 0;
		transition: opacity 160ms ease;
	}

	.cap.implied .via {
		opacity: 1;
	}

	.cap.implied .box {
		background: var(--dev-scope-read-bg);
		box-shadow: inset 0 0 0 1.5px transparent;
	}

	.cap.implied .box svg {
		opacity: 1;
		stroke: var(--dev-scope-read-fg);
	}

	/* The exact permission array the request will carry, in the caller's
	   own vocabulary, so write-implies-read is visible rather than described. */
	.manifest {
		border-radius: 10px;
		overflow: hidden;
		background: var(--dev-card-bg);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.manifest-bar {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 7px 12px;
		background: var(--bg-secondary);
		border-bottom: 0.5px solid var(--border-hairline);
		font-family: 'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace;
		font-size: 10.5px;
		letter-spacing: 0.02em;
		color: var(--text-tertiary);
	}

	.manifest-bar .k {
		color: var(--dev-accent);
		font-weight: 700;
	}

	.manifest-bar .n {
		margin-left: auto;
		font-weight: 600;
		color: var(--text-primary);
	}

	.manifest-body {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 5px;
		padding: 11px 12px;
		min-height: 42px;
	}

	.manifest-empty {
		font-size: 11.5px;
		color: var(--text-tertiary);
		letter-spacing: -0.005em;
	}

	@media (max-width: 900px) {
		.res {
			grid-template-columns: 1fr;
			gap: 10px;
			align-items: start;
		}

		.levels {
			width: 100%;
		}

		.cap {
			grid-template-columns: 16px minmax(0, 1fr);
		}

		.via {
			grid-column: 2;
			justify-self: start;
			align-self: start;
			margin-top: 6px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.thumb,
		.box,
		.box svg,
		.via,
		.cap {
			transition: none;
		}
	}
</style>
