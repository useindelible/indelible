<script lang="ts">
	import {
		WEBHOOK_EVENT_GROUPS,
		type WebhookDelivery,
		type WebhookEndpoint
	} from '$lib/api/webhooks';
	import WebhookCreateDialog from './WebhookCreateDialog.svelte';
	import WebhookEndpointRow from './WebhookEndpointRow.svelte';
	import WebhookSecretRevealCard from './WebhookSecretRevealCard.svelte';

	interface WebhookSecret {
		name: string;
		raw_secret: string;
	}

	interface Props {
		endpoints: WebhookEndpoint[];
		endpointCount: number;
		expandedEndpoint: string | null;
		deliveriesByEndpoint: Record<string, WebhookDelivery[]>;
		testEventByEndpoint: Record<string, string>;
		addOpen: boolean;
		addName: string;
		addUrl: string;
		addEvents: Set<string>;
		addActive: boolean;
		creatingEndpoint: boolean;
		addError: string | null;
		revealWebhookSecret: WebhookSecret | null;
		webhookSecretCopied: boolean;
		onOpenAdd: () => void;
		onCloseAdd: () => void;
		onAddName: (name: string) => void;
		onAddUrl: (url: string) => void;
		onToggleEvent: (event: string) => void;
		onToggleGroup: (events: string[]) => void;
		onAddActive: (active: boolean) => void;
		onCreateEndpoint: () => void;
		onToggleExpanded: (id: string) => void;
		onRotateSecret: (id: string) => void;
		onSendTest: (id: string) => void;
		onToggleActive: (id: string, next: boolean) => void;
		onDelete: (id: string) => void;
		onSetTestEvent: (id: string, event: string) => void;
		onCopyWebhookSecret: () => void;
		onDismissWebhookSecret: () => void;
	}

	let {
		endpoints,
		endpointCount,
		expandedEndpoint,
		deliveriesByEndpoint,
		testEventByEndpoint,
		addOpen,
		addName,
		addUrl,
		addEvents,
		addActive,
		creatingEndpoint,
		addError,
		revealWebhookSecret,
		webhookSecretCopied,
		onOpenAdd,
		onCloseAdd,
		onAddName,
		onAddUrl,
		onToggleEvent,
		onToggleGroup,
		onAddActive,
		onCreateEndpoint,
		onToggleExpanded,
		onRotateSecret,
		onSendTest,
		onToggleActive,
		onDelete,
		onSetTestEvent,
		onCopyWebhookSecret,
		onDismissWebhookSecret
	}: Props = $props();
</script>

<section class="zone">
	<div class="zone-head">
		<div>
			<div class="zone-title">Webhooks</div>
			<div class="zone-desc">
				Push events from Indelible to your own services. Each endpoint gets an independent secret,
				scoped event subscriptions, and a delivery log with replay.
			</div>
		</div>
		<div class="zone-actions">
			<button type="button" class="btn primary" onclick={onOpenAdd}>Add endpoint</button>
		</div>
	</div>

	<WebhookSecretRevealCard
		secret={revealWebhookSecret}
		copied={webhookSecretCopied}
		onCopy={onCopyWebhookSecret}
		onDismiss={onDismissWebhookSecret}
	/>

	<div class="group">
		<div class="group-label">
			<span>Endpoints</span>
			<span class="meta">{endpointCount} endpoint{endpointCount === 1 ? '' : 's'}</span>
		</div>

		<div class="group-card">
			{#if endpoints.length === 0}
				<div class="empty">No webhook endpoints yet. Add one to receive events from Indelible.</div>
			{:else}
				<div class="endpoints-list">
					{#each endpoints as endpoint (endpoint.id)}
						<WebhookEndpointRow
							{endpoint}
							expanded={expandedEndpoint === endpoint.id}
							deliveries={deliveriesByEndpoint[endpoint.id] ?? []}
							testEvent={testEventByEndpoint[endpoint.id] ?? endpoint.events[0] ?? ''}
							{onToggleExpanded}
							{onRotateSecret}
							{onSendTest}
							{onToggleActive}
							{onDelete}
							{onSetTestEvent}
						/>
					{/each}
				</div>
			{/if}
		</div>

		<WebhookCreateDialog
			open={addOpen}
			name={addName}
			url={addUrl}
			events={addEvents}
			active={addActive}
			creating={creatingEndpoint}
			error={addError}
			eventGroups={WEBHOOK_EVENT_GROUPS}
			onClose={onCloseAdd}
			onName={onAddName}
			onUrl={onAddUrl}
			{onToggleEvent}
			{onToggleGroup}
			onActive={onAddActive}
			onSubmit={onCreateEndpoint}
		/>
	</div>
</section>

<style>
	.zone {
		margin-bottom: 0;
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
		letter-spacing: 0;
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
		letter-spacing: 0;
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
		letter-spacing: 0;
	}

	.group-card {
		background: var(--dev-card-bg);
		border-radius: 14px;
		overflow: hidden;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.endpoints-list {
		display: flex;
		flex-direction: column;
	}

	.empty {
		padding: 28px 22px;
		font-size: 13px;
		color: var(--text-secondary);
		text-align: center;
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

	@media (max-width: 720px) {
		.zone-head {
			flex-direction: column;
			align-items: flex-start;
		}
	}
</style>
