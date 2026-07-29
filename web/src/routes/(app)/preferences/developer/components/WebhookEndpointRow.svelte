<script lang="ts">
	import type { WebhookDelivery, WebhookEndpoint } from '$lib/api/webhooks';
	import { formatTime, lastStatusClass, lastStatusLabel, statusClassFor } from '../developer-model';

	interface Props {
		endpoint: WebhookEndpoint;
		expanded: boolean;
		deliveries: WebhookDelivery[];
		testEvent: string;
		onToggleExpanded: (id: string) => void;
		onRotateSecret: (id: string) => void;
		onSendTest: (id: string) => void;
		onToggleActive: (id: string, next: boolean) => void;
		onDelete: (id: string) => void;
		onSetTestEvent: (id: string, event: string) => void;
	}

	let {
		endpoint,
		expanded,
		deliveries,
		testEvent,
		onToggleExpanded,
		onRotateSecret,
		onSendTest,
		onToggleActive,
		onDelete,
		onSetTestEvent
	}: Props = $props();
</script>

<div class="endpoint" class:expanded>
	<button
		type="button"
		class="endpoint-row"
		aria-label={endpoint.url}
		onclick={() => onToggleExpanded(endpoint.id)}
	>
		<span class="chevron">
			<svg viewBox="0 0 24 24"><path d="M9 6l6 6-6 6" /></svg>
		</span>
		<div class="endpoint-url">
			<div class="url">{endpoint.url}</div>
			<div class="ep-name">{endpoint.name}</div>
		</div>
		<span class="endpoint-events-count">
			<strong>{endpoint.events.length}</strong>
			{endpoint.events.length === 1 ? 'event' : 'events'}
		</span>
		<span class="status-pill {lastStatusClass(endpoint)}">{lastStatusLabel(endpoint)}</span>
		<span class="delivery-rail" title="Last 8 deliveries">
			{#each endpoint.delivery_history as tick, index (`${tick}-${index}`)}
				<span class="tick {tick}"></span>
			{/each}
		</span>
	</button>

	<div class="endpoint-detail">
		<div class="endpoint-detail-inner">
			<div class="detail-block">
				<div class="detail-label">Subscribed events</div>
				<div class="pill-row">
					{#each endpoint.events as event (event)}
						<span class="event-pill">{event}</span>
					{/each}
				</div>
			</div>
			<div class="detail-block">
				<div class="detail-label">Signing secret</div>
				<div class="secret-row">
					<input class="input mono" type="text" value={endpoint.secret_preview} readonly />
					<button
						type="button"
						class="btn ghost compact"
						onclick={() => onRotateSecret(endpoint.id)}
					>
						<svg viewBox="0 0 24 24">
							<path d="M3 12a9 9 0 0 1 15-6.7L21 9" />
							<path d="M21 4v5h-5" />
						</svg>
						Rotate
					</button>
				</div>
			</div>
			<div class="detail-block">
				<div class="detail-label">Recent deliveries</div>
				{#if deliveries.length === 0}
					<div class="deliveries-empty">No deliveries yet. Use "Send test" to fire one.</div>
				{:else}
					<div class="deliveries-mini">
						{#each deliveries as delivery (delivery.id)}
							<div class="delivery">
								<span class="ts">{formatTime(delivery.attempted_at)}</span>
								<span class="ev">{delivery.event}</span>
								<span class="target">{delivery.target}</span>
								<span class="status {statusClassFor(delivery.status_code)}">
									<span>{delivery.outcome === 'delivered' ? 'Delivered' : 'Failed'}</span>
									{#if typeof delivery.status_code === 'number'}
										<span class="status-code">HTTP {delivery.status_code}</span>
									{/if}
								</span>
								<span class="latency">
									{delivery.latency_ms !== null ? `${delivery.latency_ms}ms` : '-'}
								</span>
								{#if delivery.error}
									<span class="delivery-error">{delivery.error}</span>
								{/if}
							</div>
						{/each}
					</div>
				{/if}
			</div>
			<div class="endpoint-actions-row">
				<select
					class="select test-select"
					aria-label="Test event"
					value={testEvent}
					onchange={(event) => onSetTestEvent(endpoint.id, event.currentTarget.value)}
				>
					{#each endpoint.events as event (event)}
						<option value={event}>Send test · {event}</option>
					{/each}
				</select>
				<button type="button" class="btn ghost compact" onclick={() => onSendTest(endpoint.id)}>
					Send test
				</button>
				<span class="spacer"></span>
				<span class="active-row">
					Active
					<button
						type="button"
						class="toggle"
						class:on={endpoint.is_active}
						aria-pressed={endpoint.is_active}
						aria-label="Toggle active"
						onclick={() => onToggleActive(endpoint.id, !endpoint.is_active)}
					></button>
				</span>
				<button type="button" class="btn danger compact" onclick={() => onDelete(endpoint.id)}>
					Delete
				</button>
			</div>
		</div>
	</div>
</div>

<style>
	.endpoint {
		border-top: 0.5px solid var(--border-hairline);
		transition: background 120ms ease;
	}

	.endpoint:first-child {
		border-top: none;
	}

	.endpoint.expanded {
		background: var(--fill-hover);
	}

	.endpoint-row {
		display: grid;
		grid-template-columns: 28px minmax(0, 1fr) auto auto auto;
		gap: 14px;
		align-items: center;
		padding: 14px 18px;
		cursor: pointer;
		width: 100%;
		text-align: left;
		background: none;
		border: none;
		color: inherit;
		font: inherit;
	}

	.chevron {
		width: 20px;
		height: 20px;
		color: var(--text-tertiary);
		display: flex;
		align-items: center;
		justify-content: center;
		transition: transform 200ms ease;
	}

	.chevron svg,
	.btn svg {
		stroke: currentColor;
		fill: none;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.chevron svg {
		width: 12px;
		height: 12px;
		stroke-width: 1.8;
	}

	.endpoint.expanded .chevron {
		transform: rotate(90deg);
		color: var(--dev-accent);
	}

	.endpoint-url {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.url {
		font-family: 'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace;
		font-size: 12.5px;
		color: var(--text-primary);
		letter-spacing: 0.02em;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.ep-name,
	.endpoint-events-count,
	.active-row {
		font-size: 11.5px;
		color: var(--text-secondary);
		letter-spacing: 0;
	}

	.endpoint-events-count,
	.active-row,
	.status-pill,
	.delivery-rail {
		display: inline-flex;
		align-items: center;
	}

	.endpoint-events-count {
		gap: 6px;
	}

	.endpoint-events-count strong {
		color: var(--text-primary);
		font-weight: 600;
	}

	.status-pill {
		gap: 5px;
		padding: 3px 9px;
		border-radius: 980px;
		font-size: 11px;
		font-weight: 500;
		letter-spacing: 0;
		line-height: 1.4;
	}

	.status-pill.healthy {
		background: var(--dev-success-soft);
		color: var(--dev-success-fg);
	}

	.status-pill.failing {
		background: var(--dev-destructive-soft);
		color: var(--destructive);
	}

	.status-pill.paused {
		background: var(--bg-tertiary);
		color: var(--text-secondary);
	}

	.status-pill::before {
		content: '';
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: currentColor;
	}

	.delivery-rail {
		gap: 3px;
	}

	.tick {
		width: 4px;
		height: 14px;
		border-radius: 1.5px;
		background: var(--dev-text-quaternary);
	}

	.tick.s2xx {
		background: var(--dev-term-status-2xx);
	}

	.tick.s4xx {
		background: var(--dev-term-status-4xx);
	}

	.tick.s5xx {
		background: var(--dev-term-status-5xx);
	}

	.tick.failed {
		background: var(--dev-term-status-5xx);
	}

	.tick.pending {
		background: var(--dev-text-quaternary);
		opacity: 0.55;
	}

	.endpoint-detail {
		overflow: hidden;
		max-height: 0;
		transition: max-height 360ms ease;
	}

	.endpoint.expanded .endpoint-detail {
		max-height: 980px;
	}

	.endpoint-detail-inner {
		padding: 4px 18px 22px 60px;
		display: grid;
		grid-template-columns: 1fr;
		gap: 18px;
	}

	.detail-block {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.detail-label {
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0;
		text-transform: uppercase;
		color: var(--text-tertiary);
	}

	.pill-row {
		display: inline-flex;
		flex-wrap: wrap;
		gap: 5px;
	}

	.event-pill {
		display: inline-flex;
		align-items: center;
		padding: 3px 9px;
		border-radius: 980px;
		background: var(--bg-secondary);
		color: var(--text-secondary);
		font-family: 'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace;
		font-size: 10.5px;
		letter-spacing: 0.02em;
		box-shadow:
			inset 0 0 0 0.5px var(--border-primary),
			0 1px 0 rgba(0, 0, 0, 0.02);
	}

	.secret-row {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 8px;
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

	.mono {
		font-family: 'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace;
		font-size: 12px;
	}

	.deliveries-empty {
		font-size: 12px;
		color: var(--text-tertiary);
		font-style: italic;
	}

	.deliveries-mini {
		display: flex;
		flex-direction: column;
	}

	.delivery {
		display: grid;
		grid-template-columns: 56px 60px minmax(0, 1fr) 80px 60px;
		gap: 10px;
		align-items: center;
		padding: 8px 0;
		border-top: 0.5px dashed var(--border-hairline);
		font-family: 'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace;
		font-size: 11.5px;
	}

	.delivery:first-child {
		border-top: none;
	}

	.ts,
	.latency {
		color: var(--text-tertiary);
	}

	.ev {
		color: var(--text-primary);
		font-weight: 600;
	}

	.target {
		color: var(--text-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.status,
	.latency {
		text-align: right;
	}

	.status {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 1px;
	}

	.status-code {
		font-size: 10px;
		color: var(--text-tertiary);
	}

	.status.s2xx {
		color: var(--dev-term-status-2xx);
	}

	.status.s4xx {
		color: var(--dev-term-status-4xx);
	}

	.status.s5xx {
		color: var(--dev-term-status-5xx);
	}

	.delivery-error {
		grid-column: 2 / -1;
		color: var(--destructive);
		overflow-wrap: anywhere;
	}

	.endpoint-actions-row {
		display: flex;
		gap: 8px;
		padding-top: 14px;
		border-top: 0.5px solid var(--border-hairline);
		align-items: center;
		flex-wrap: wrap;
	}

	.test-select {
		width: auto;
		min-width: 200px;
	}

	.spacer {
		flex: 1;
	}

	.active-row {
		gap: 8px;
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
		transition:
			background 120ms,
			transform 120ms;
		white-space: nowrap;
		border: none;
		background: none;
		color: inherit;
		font-family: inherit;
	}

	.btn.ghost {
		background: transparent;
		color: var(--text-primary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.btn.ghost:hover {
		background: var(--fill-hover);
	}

	.btn.danger {
		background: transparent;
		color: var(--destructive);
		box-shadow: inset 0 0 0 0.5px var(--dev-destructive-border);
	}

	.btn.danger:hover {
		background: var(--dev-destructive-soft);
	}

	.btn.compact {
		padding: 5px 10px;
		font-size: 11.5px;
	}

	.btn svg {
		width: 13px;
		height: 13px;
		stroke-width: 1.7;
	}

	.toggle {
		width: 40px;
		height: 24px;
		border-radius: 12px;
		background: var(--bg-tertiary);
		position: relative;
		cursor: pointer;
		flex-shrink: 0;
		transition: background 200ms ease;
		border: none;
		padding: 0;
	}

	.toggle::after {
		content: '';
		position: absolute;
		top: 3px;
		left: 3px;
		width: 18px;
		height: 18px;
		border-radius: 50%;
		background: var(--bg-elevated);
		box-shadow: var(--shadow-1);
		transition: left 200ms ease;
	}

	.toggle.on {
		background: var(--dev-accent);
	}

	.toggle.on::after {
		left: 19px;
	}

	@media (max-width: 720px) {
		.endpoint-row {
			grid-template-columns: 24px minmax(0, 1fr);
		}

		.endpoint-events-count,
		.status-pill,
		.delivery-rail {
			display: none;
		}

		.endpoint-detail-inner {
			padding-left: 20px;
		}
	}
</style>
