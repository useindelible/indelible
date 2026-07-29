<script lang="ts">
	import type { ApiTokenResponse } from '$lib/api';
	import type { ExpiryOption, ScopeDef, ScopeKey } from '../developer-model';
	import { formatDate, formatRelative, scopeClass } from '../developer-model';
	import TokenIssueDialog from './TokenIssueDialog.svelte';
	import TokenRevealCard from './TokenRevealCard.svelte';

	interface Props {
		tokens: ApiTokenResponse[];
		loading: boolean;
		error: string | null;
		activeTokens: number;
		issueOpen: boolean;
		issueName: string;
		issueScopes: Set<ScopeKey>;
		issueExpiry: ExpiryOption;
		creatingToken: boolean;
		issueError: string | null;
		revealToken: string | null;
		copied: boolean;
		scopeDefs: ScopeDef[];
		onOpenIssue: () => void;
		onCloseIssue: () => void;
		onIssueName: (name: string) => void;
		onToggleIssueScope: (scope: ScopeKey) => void;
		onIssueExpiry: (expiry: ExpiryOption) => void;
		onSubmitIssueToken: () => void;
		onCopyToken: () => void;
		onDismissToken: () => void;
		onRevokeToken: (id: string) => void;
	}

	let {
		tokens,
		loading,
		error,
		activeTokens,
		issueOpen,
		issueName,
		issueScopes,
		issueExpiry,
		creatingToken,
		issueError,
		revealToken,
		copied,
		scopeDefs,
		onOpenIssue,
		onCloseIssue,
		onIssueName,
		onToggleIssueScope,
		onIssueExpiry,
		onSubmitIssueToken,
		onCopyToken,
		onDismissToken,
		onRevokeToken
	}: Props = $props();
</script>

<section class="zone">
	<div class="zone-head">
		<div>
			<div class="zone-title">API Tokens</div>
			<div class="zone-desc">
				Personal access tokens scoped to read, write, or admin operations. Keep them secret — they
				grant the same access as your account.
			</div>
		</div>
		<div class="zone-actions">
			<button type="button" class="btn primary" onclick={onOpenIssue}>Issue token</button>
		</div>
	</div>

	<div class="group">
		<div class="group-label">
			<span>Active tokens</span>
			<span class="meta">{activeTokens} active</span>
		</div>

		<div class="group-card">
			{#if loading}
				<div class="empty">Loading tokens…</div>
			{:else if error}
				<div class="empty error" role="alert">{error}</div>
			{:else if tokens.length === 0}
				<div class="empty">
					No tokens yet. Issue one to call the Indelible API from your scripts, CLI, or another app.
				</div>
			{:else}
				<table class="table">
					<thead>
						<tr>
							<th>Name</th>
							<th>Scopes</th>
							<th>Last used</th>
							<th>Created / Expires</th>
							<th class="col-actions"></th>
						</tr>
					</thead>
					<tbody>
						{#each tokens as token (token.id)}
							<tr>
								<td>
									<div class="token-name">
										<div class="token-orb" aria-hidden="true">
											<svg viewBox="0 0 24 24">
												<path d="M21 2l-9 9" />
												<path d="M15 4l5 5" />
												<circle cx="7.5" cy="16.5" r="3.5" />
											</svg>
										</div>
										<div class="token-name-text">
											<div class="name">{token.name}</div>
											<div class="prefix">{token.prefix}</div>
										</div>
									</div>
								</td>
								<td>
									<span class="scopes">
										{#each token.scopes as scope (scope)}
											<span class="scope {scopeClass(scope)}">{scope}</span>
										{/each}
									</span>
								</td>
								<td>
									<span class="relative-time">{formatRelative(token.last_used_at)}</span>
								</td>
								<td>
									<span class="relative-time">
										{formatDate(token.created_at)}
										<span class="small">
											{token.expires_at ? `Expires ${formatDate(token.expires_at)}` : 'No expiry'}
										</span>
									</span>
								</td>
								<td class="col-actions">
									<button
										type="button"
										class="btn danger compact"
										onclick={() => onRevokeToken(token.id)}
									>
										Revoke
									</button>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			{/if}
		</div>

		<TokenIssueDialog
			open={issueOpen}
			name={issueName}
			scopes={issueScopes}
			expiry={issueExpiry}
			{scopeDefs}
			creating={creatingToken}
			error={issueError}
			onClose={onCloseIssue}
			onName={onIssueName}
			onToggleScope={onToggleIssueScope}
			onExpiry={onIssueExpiry}
			onSubmit={onSubmitIssueToken}
		/>

		<TokenRevealCard token={revealToken} {copied} onCopy={onCopyToken} onDismiss={onDismissToken} />
	</div>
</section>

<style>
	.zone {
		margin-bottom: 56px;
	}

	.zone-head {
		display: flex;
		align-items: flex-end;
		justify-content: space-between;
		gap: 24px;
		padding-bottom: 18px;
		margin-bottom: 18px;
		border-bottom: 0.5px solid var(--border-hairline);
	}

	.zone-title {
		font-family: 'New York', Georgia, serif;
		font-style: italic;
		font-size: 22px;
		letter-spacing: -0.02em;
		color: var(--text-primary);
		line-height: 1.2;
		margin-bottom: 4px;
	}

	.zone-desc {
		font-size: 13px;
		color: var(--text-secondary);
		line-height: 1.45;
		max-width: 500px;
	}

	.zone-actions {
		display: inline-flex;
		gap: 8px;
		flex-shrink: 0;
	}

	.group {
		margin-bottom: 24px;
	}

	.group-label {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		padding: 0 4px 8px;
		display: flex;
		justify-content: space-between;
		align-items: baseline;
	}

	.meta {
		font-size: 11px;
		font-weight: 400;
		color: var(--text-tertiary);
		text-transform: none;
		letter-spacing: -0.005em;
	}

	.group-card {
		background: var(--dev-card-bg);
		border-radius: 14px;
		/* The fixed-purpose token columns exceed narrow cards; the table pans
		   inside the card instead of clipping the Revoke column. */
		overflow: hidden;
		overflow-x: auto;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		container-type: inline-size;
		container-name: settings-card;
	}

	.empty {
		padding: 28px 22px;
		font-size: 13px;
		color: var(--text-secondary);
		text-align: center;
	}

	.empty.error {
		color: var(--destructive);
	}

	.table {
		width: 100%;
		min-width: 620px;
		border-collapse: collapse;
		background: var(--dev-card-strong);
	}

	th {
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		text-align: left;
		padding: 12px 18px;
		border-bottom: 0.5px solid var(--border-hairline);
		background: var(--bg-secondary);
	}

	td {
		padding: 14px 18px;
		border-bottom: 0.5px solid var(--border-hairline);
		font-size: 13px;
		color: var(--text-primary);
		letter-spacing: -0.005em;
		vertical-align: middle;
	}

	tr:last-child td {
		border-bottom: none;
	}

	.col-actions {
		text-align: right;
		width: 100px;
	}

	.token-name {
		display: flex;
		align-items: center;
		gap: 10px;
		min-width: 0;
	}

	.token-orb {
		width: 28px;
		height: 28px;
		border-radius: 50%;
		flex-shrink: 0;
		background: linear-gradient(135deg, var(--dev-accent), var(--dev-scope-ext-fg));
		box-shadow:
			0 2px 8px var(--dev-accent-glow),
			inset 0 0 0 1px rgba(255, 255, 255, 0.18);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-on-color);
	}

	.token-orb svg {
		width: 13px;
		height: 13px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.8;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.token-name-text {
		min-width: 0;
	}

	.name {
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		letter-spacing: -0.01em;
	}

	.prefix {
		font-family: 'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace;
		font-size: 11px;
		color: var(--text-tertiary);
		letter-spacing: 0.02em;
		margin-top: 1px;
	}

	.scopes {
		display: inline-flex;
		flex-wrap: wrap;
		gap: 4px;
	}

	.scope {
		display: inline-flex;
		align-items: center;
		padding: 2px 7px;
		border-radius: 5px;
		font-family: 'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace;
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: -0.01em;
		text-transform: lowercase;
	}

	.scope.read {
		background: var(--dev-scope-read-bg);
		color: var(--dev-scope-read-fg);
	}

	.scope.write {
		background: var(--dev-scope-write-bg);
		color: var(--dev-scope-write-fg);
	}

	.scope.admin {
		background: var(--dev-scope-admin-bg);
		color: var(--dev-scope-admin-fg);
	}

	.scope.cli {
		background: var(--dev-scope-cli-bg);
		color: var(--dev-scope-cli-fg);
	}

	.scope.ext,
	.scope.obsidian {
		background: var(--dev-scope-ext-bg);
		color: var(--dev-scope-ext-fg);
	}

	.relative-time {
		font-size: 12.5px;
		color: var(--text-secondary);
		letter-spacing: -0.01em;
	}

	.small {
		display: block;
		font-size: 11px;
		color: var(--text-tertiary);
		margin-top: 1px;
	}

	.btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 7px 14px;
		border-radius: 8px;
		font-size: 12.5px;
		font-weight: 500;
		letter-spacing: 0;
		cursor: pointer;
		white-space: nowrap;
		border: none;
		background: none;
		color: inherit;
		font-family: inherit;
	}

	.btn.primary {
		background: var(--dev-accent);
		color: var(--text-on-color);
	}

	.btn.danger {
		background: transparent;
		color: var(--destructive);
		box-shadow: inset 0 0 0 0.5px var(--dev-destructive-border);
	}

	.btn.compact {
		padding: 5px 10px;
		font-size: 11.5px;
	}

	@media (max-width: 720px) {
		.zone-head {
			flex-direction: column;
			align-items: flex-start;
		}
	}

	@container settings-card (max-width: 639px) {
		th {
			padding: 11px 14px;
		}

		td {
			padding: 12px 14px;
		}
	}
</style>
