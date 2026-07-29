<script lang="ts">
	import type { EmailSenderResponse, RenderDefaultDto } from '$lib/api';
	import { formatRelative, routingValue, senderInitial } from '../email-model';

	interface Props {
		senders: EmailSenderResponse[];
		totalSenders: number;
		updatingSender: string | null;
		unsubscribingSender: string | null;
		onRenderChange: (sender: EmailSenderResponse, value: RenderDefaultDto) => void;
		onRoutingChange: (sender: EmailSenderResponse, value: string) => void;
		onToggleBlock: (sender: EmailSenderResponse) => void;
		onUnsubscribe: (sender: EmailSenderResponse) => void;
	}

	let {
		senders,
		totalSenders,
		updatingSender,
		unsubscribingSender,
		onRenderChange,
		onRoutingChange,
		onToggleBlock,
		onUnsubscribe
	}: Props = $props();
</script>

<div class="ledger-wrap">
	{#if senders.length === 0}
		<p class="muted">No senders match this filter.</p>
	{:else}
		<div class="ledger-scroll">
			<table class="ledger">
				<thead>
					<tr>
						<th>Sender</th>
						<th>List-ID</th>
						<th>Activity</th>
						<th>Render</th>
						<th>Routing</th>
						<th class="center">Block</th>
						<th class="right">Action</th>
					</tr>
				</thead>
				<tbody>
					{#each senders as sender (sender.id)}
						<tr class="ledger-row" data-blocked={sender.blocked}>
							<td>
								<div class="sender-cell">
									<div class="sender-mark">{senderInitial(sender)}</div>
									<div class="sender-meta">
										<div class="sender-name">
											{sender.display_name ??
												sender.canonical_addr.split('@')[1] ??
												sender.canonical_addr}
										</div>
										<div class="sender-addr">{sender.canonical_addr}</div>
									</div>
								</div>
							</td>
							<td>
								{#if sender.list_id}
									<span class="listid">{sender.list_id}</span>
								{:else}
									<span class="listid empty">no list-id provided</span>
								{/if}
							</td>
							<td>
								<div class="activity-cell">
									<div class="last">{formatRelative(sender.last_seen_at)}</div>
									<div class="count">
										<strong>{sender.delivery_count.toLocaleString()}</strong> received
									</div>
								</div>
							</td>
							<td>
								<select
									class="inline-select"
									aria-label="Render mode for {sender.canonical_addr}"
									value={sender.render_default}
									disabled={sender.blocked || updatingSender === sender.id}
									onchange={(event) =>
										onRenderChange(sender, event.currentTarget.value as RenderDefaultDto)}
								>
									<option value="reader">Reader</option>
									<option value="original">Original</option>
								</select>
							</td>
							<td>
								<select
									class="inline-select"
									class:routing-feed={!sender.blocked &&
										(sender.routing_default === 'feed' || sender.routing_default == null)}
									aria-label="Routing for {sender.canonical_addr}"
									value={routingValue(sender)}
									disabled={sender.blocked || updatingSender === sender.id}
									onchange={(event) => onRoutingChange(sender, event.currentTarget.value)}
								>
									<option value="default">Default</option>
									<option value="feed">Feed</option>
									<option value="library">Library</option>
								</select>
							</td>
							<td class="center">
								<label class="toggle">
									<input
										type="checkbox"
										checked={sender.blocked}
										disabled={updatingSender === sender.id}
										aria-label="Block {sender.canonical_addr}"
										onchange={() => onToggleBlock(sender)}
									/>
									<span class="toggle-track"></span>
									<span class="toggle-label">{sender.blocked ? 'Blocked' : 'Allow'}</span>
								</label>
							</td>
							<td class="right">
								<button
									class="unsub-btn"
									type="button"
									disabled={unsubscribingSender === sender.id || sender.blocked}
									aria-label="Unsubscribe from {sender.display_name ?? sender.canonical_addr}"
									onclick={() => onUnsubscribe(sender)}
								>
									<svg viewBox="0 0 24 24" aria-hidden="true">
										<path d="M3 6l3 14h12l3-14" />
										<path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
										<path d="M10 11v6M14 11v6" />
									</svg>
									{unsubscribingSender === sender.id ? 'Sending…' : 'Unsubscribe'}
								</button>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
	<div class="ledger-foot">
		<div class="pagination">
			<strong>{senders.length}</strong> of <strong>{totalSenders}</strong>
		</div>
		<div class="legend"><em>Default</em> respects the inbox the message arrived on.</div>
	</div>
</div>

<style>
	.ledger-wrap {
		position: relative;
		border-radius: var(--radius-lg);
		background: var(--paper);
		box-shadow: var(--envelope-shadow);
		overflow: hidden;
	}

	.ledger-wrap::before {
		content: '';
		position: absolute;
		left: 36px;
		top: 44px;
		bottom: 0;
		width: 1px;
		background: var(--accent-line);
		opacity: 0.2;
		z-index: 0;
		pointer-events: none;
	}

	/* On narrow cards the fixed-purpose columns exceed the card; the table
	   pans inside it while the footer stays pinned. */
	.ledger-scroll {
		overflow-x: auto;
	}

	.ledger {
		position: relative;
		width: 100%;
		min-width: 560px;
		border-collapse: collapse;
		font-size: 13px;
		z-index: 1;
	}

	th {
		text-align: left;
		padding: 12px;
		font-family: var(--font-mono);
		font-size: 9.5px;
		font-weight: 600;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		background: var(--table-head-bg);
		border-bottom: 0.5px solid var(--border-primary);
		white-space: nowrap;
	}

	td {
		padding: 14px 12px;
		border-bottom: 0.5px solid var(--border-hairline);
		color: var(--text-primary);
		letter-spacing: -0.005em;
		vertical-align: middle;
	}

	td:first-child {
		padding-left: 18px;
	}

	td:last-child {
		padding-right: 18px;
	}

	tr:last-child td {
		border-bottom: none;
	}

	tr:hover {
		background: var(--table-row-hover);
	}

	.ledger-row[data-blocked='true'] {
		background: var(--table-row-blocked);
	}

	.ledger-row[data-blocked='true'] .sender-name,
	.ledger-row[data-blocked='true'] .sender-addr {
		opacity: 0.55;
		text-decoration: line-through;
		text-decoration-color: var(--text-tertiary);
		text-decoration-thickness: 0.5px;
	}

	.ledger-row[data-blocked='true'] .listid,
	.ledger-row[data-blocked='true'] .activity-cell,
	.ledger-row[data-blocked='true'] .inline-select {
		opacity: 0.6;
	}

	.sender-cell {
		display: inline-flex;
		align-items: flex-start;
		gap: 12px;
		min-width: 0;
	}

	.sender-mark {
		position: relative;
		width: 36px;
		height: 36px;
		border-radius: 4px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		font-family: var(--font-display);
		font-style: italic;
		font-size: 17px;
		font-weight: 500;
		letter-spacing: -0.02em;
		background: var(--stamp-fill);
		color: var(--stamp-ink);
		flex-shrink: 0;
	}

	.sender-mark::after {
		content: '';
		position: absolute;
		inset: 2px;
		border: 0.5px dashed var(--stamp-line);
		opacity: 0.6;
		border-radius: 2px;
		pointer-events: none;
	}

	.sender-meta {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
		flex: 1;
	}

	.sender-name {
		font-size: 13.5px;
		font-weight: 600;
		color: var(--text-primary);
		letter-spacing: -0.01em;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 240px;
	}

	.sender-addr,
	.listid,
	.count,
	.last,
	.legend,
	.pagination,
	.muted {
		color: var(--text-secondary);
		font-size: 12px;
	}

	.sender-addr,
	.listid {
		font-family: var(--font-mono);
		letter-spacing: -0.005em;
	}

	.sender-addr {
		font-size: 11.5px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 280px;
	}

	.listid {
		font-size: 11px;
		letter-spacing: 0.01em;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 200px;
		display: inline-block;
		padding: 3px 7px;
		background: var(--code-bg);
		border-radius: 4px;
	}

	.listid.empty,
	.muted,
	.legend {
		color: var(--text-tertiary);
	}

	.listid.empty {
		font-family: var(--font-display);
		font-style: italic;
		background: transparent;
		padding: 0;
		font-size: 12px;
	}

	.activity-cell {
		display: flex;
		flex-direction: column;
		gap: 2px;
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-secondary);
		letter-spacing: 0.01em;
		line-height: 1.4;
		white-space: nowrap;
	}

	.last {
		font-size: 12px;
		color: var(--text-primary);
		font-weight: 500;
	}

	.count {
		color: var(--text-tertiary);
		font-size: 10.5px;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		margin-top: 2px;
	}

	.count strong {
		font-family: var(--font-display);
		font-style: italic;
		color: var(--text-secondary);
		font-weight: 600;
		font-size: 12px;
		letter-spacing: -0.01em;
		text-transform: none;
	}

	.inline-select {
		appearance: none;
		-webkit-appearance: none;
		background: var(--bg-secondary);
		color: var(--text-primary);
		border: none;
		border-radius: 6px;
		padding: 5px 10px;
		font-family: inherit;
		font-size: 12px;
		font-weight: 500;
		letter-spacing: -0.005em;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		outline: none;
		cursor: pointer;
		transition: box-shadow 150ms ease;
		max-width: 130px;
	}

	.inline-select:focus {
		box-shadow:
			inset 0 0 0 1.5px var(--accent),
			0 0 0 3px var(--accent-soft);
	}

	.inline-select:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}

	.inline-select.routing-feed {
		color: var(--accent-strong);
	}

	.center {
		text-align: center;
	}

	.right {
		text-align: right;
	}

	.toggle {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		cursor: pointer;
		user-select: none;
	}

	.toggle input {
		position: absolute;
		opacity: 0;
		pointer-events: none;
	}

	.toggle-track {
		width: 32px;
		height: 19px;
		border-radius: var(--radius-full);
		background: var(--bg-tertiary);
		position: relative;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.toggle-track::after {
		content: '';
		position: absolute;
		width: 15px;
		height: 15px;
		border-radius: 50%;
		top: 2px;
		left: 2px;
		background: var(--switch-thumb);
		box-shadow:
			0 1px 2px rgba(26, 22, 18, 0.2),
			0 0 0 0.5px rgba(26, 22, 18, 0.05);
		transition: left 180ms ease;
	}

	input:checked + .toggle-track {
		background: var(--accent);
	}

	input:checked + .toggle-track::after {
		left: 15px;
	}

	.toggle-label {
		font-size: 11px;
		color: var(--text-tertiary);
		font-family: var(--font-mono);
		text-transform: uppercase;
		letter-spacing: 0.08em;
	}

	.unsub-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		border: none;
		border-radius: 6px;
		padding: 6px 11px;
		background: transparent;
		color: var(--text-secondary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		font-size: 11.5px;
		font-weight: 500;
		cursor: pointer;
		letter-spacing: -0.005em;
		white-space: nowrap;
		transition:
			background 140ms ease,
			color 140ms ease,
			box-shadow 140ms ease;
	}

	.unsub-btn:hover:not(:disabled) {
		color: var(--accent-strong);
		box-shadow: inset 0 0 0 0.5px var(--accent-line);
		background: var(--accent-tint);
	}

	.unsub-btn:disabled {
		opacity: 0.45;
		cursor: default;
	}

	.unsub-btn svg {
		width: 11px;
		height: 11px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.8;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.ledger-foot,
	.muted {
		padding: 14px 18px;
	}

	.ledger-foot {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		border-top: 0.5px solid var(--border-hairline);
		background: var(--table-head-bg);
		font-size: 11.5px;
		color: var(--text-tertiary);
	}

	.pagination {
		font-family: var(--font-mono);
		font-size: 10.5px;
		letter-spacing: 0.12em;
		text-transform: uppercase;
	}

	.ledger-foot strong {
		color: var(--text-primary);
		font-weight: 600;
	}

	.legend em {
		font-family: var(--font-display);
		font-style: italic;
		color: var(--text-secondary);
	}
</style>
