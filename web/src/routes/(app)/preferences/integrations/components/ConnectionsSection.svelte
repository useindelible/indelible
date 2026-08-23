<script lang="ts">
	import type { IntegrationConnectionDto } from '$lib/api';
	import IntegrationConnectionCard from '$lib/components/integrations/IntegrationConnectionCard.svelte';
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import type { HubConnectionStatus, StoreLink, SyncState } from '../integrations-hub-model';
	import { notionDatabaseLabel } from '../integrations-hub-model';
	import EmailForwardingCard from './EmailForwardingCard.svelte';
	import { t } from '$lib/i18n';
	import { relativeTime } from '$lib/utils/relative-time';

	interface Props {
		connectionsLoading: boolean;
		connectionsError: string | null;
		inboxAddress: string;
		feedAddress: string;
		copiedInbox: boolean;
		copiedFeed: boolean;
		extStore: StoreLink;
		notionConnection: IntegrationConnectionDto | undefined;
		obsidianConnection: IntegrationConnectionDto | undefined;
		notionStatus: HubConnectionStatus;
		obsidianStatus: HubConnectionStatus;
		syncStateByConnection: Record<string, SyncState>;
		syncErrorByConnection: Record<string, string>;
		notionConnectError: string | null;
		notionAvailable: boolean;
		onCopyAddress: (address: string, which: 'inbox' | 'feed') => void;
		onStartNotion: () => void;
		onOpenNotion: () => void;
		onOpenObsidian: () => void;
		onSync: (connectionId: string) => void;
		onDisconnect: (connection: IntegrationConnectionDto) => void;
	}

	let {
		connectionsLoading,
		connectionsError,
		inboxAddress,
		feedAddress,
		copiedInbox,
		copiedFeed,
		extStore,
		notionConnection,
		obsidianConnection,
		notionStatus,
		obsidianStatus,
		syncStateByConnection,
		syncErrorByConnection,
		notionConnectError,
		notionAvailable,
		onCopyAddress,
		onStartNotion,
		onOpenNotion,
		onOpenObsidian,
		onSync,
		onDisconnect
	}: Props = $props();
</script>

<SettingsGroup
	title={$t('integrations_hub_connections')}
	meta={$t('integrations_hub_connections_hint')}
>
	{#if connectionsLoading}
		<p class="zone-meta">{$t('integrations_hub_loading_connections')}</p>
	{:else if connectionsError}
		<p class="zone-meta error" role="alert">{connectionsError}</p>
	{:else}
		<div class="connections-stack">
			<EmailForwardingCard
				{inboxAddress}
				{feedAddress}
				{copiedInbox}
				{copiedFeed}
				onCopy={onCopyAddress}
			/>

			<div class="connections-grid">
				<IntegrationConnectionCard
					title={$t('integrations_hub_browser_extension')}
					tagline={$t('integrations_hub_browser_tagline')}
					statusLabel={$t('integrations_hub_installed')}
					statusVariant="active"
					statusCheck
					testId="browser-connection-card"
				>
					{#snippet body()}
						<div class="moment">
							<div class="moment-eyebrow">{$t('integrations_hub_capture_anywhere')}</div>
							<div class="shortcut-list">
								<div class="shortcut-item">
									<span class="shortcut-icon">
										<svg viewBox="0 0 24 24" aria-hidden="true">
											<rect x="4" y="4" width="16" height="16" rx="3" />
											<path d="M9 12l2 2 4-4" />
										</svg>
									</span>
									<span class="shortcut-meta">
										<span class="shortcut-title">{$t('integrations_hub_save_page')}</span>
										<span class="shortcut-sub">{$t('integrations_hub_save_page_hint')}</span>
									</span>
									<span class="shortcut-keys"><kbd>⌘</kbd><kbd>S</kbd></span>
								</div>
								<div class="shortcut-item">
									<span class="shortcut-icon">
										<svg viewBox="0 0 24 24" aria-hidden="true">
											<path d="M21 11.5V20a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h8.5" />
											<path d="M16 4l4 4-7 7H9v-4z" />
										</svg>
									</span>
									<span class="shortcut-meta">
										<span class="shortcut-title">{$t('integrations_hub_add_note')}</span>
										<span class="shortcut-sub">{$t('integrations_hub_add_note_hint')}</span>
									</span>
									<span class="shortcut-keys"><kbd>N</kbd></span>
								</div>
							</div>
							<div class="works-on">
								<span>{$t('integrations_hub_works_on')}</span>
								<span class="browsers">
									<span class="browser-glyph" aria-label="Chrome">
										<svg viewBox="0 0 24 24" aria-hidden="true">
											<circle cx="12" cy="12" r="10" fill="#4285F4" />
											<circle cx="12" cy="12" r="4" fill="#fff" />
											<path d="M21.5 9.5h-9l-3-5.2A10 10 0 0 1 21.5 9.5z" fill="#EA4335" />
											<path d="M2.5 14.5l4.5-2.6 4.5 7.8A10 10 0 0 1 2.5 14.5z" fill="#34A853" />
											<path d="M21.5 14.5l-4.5-2.6-4.5 7.8a10 10 0 0 0 9-5.2z" fill="#FBBC04" />
										</svg>
									</span>
									<span class="browser-glyph" aria-label="Firefox">
										<svg viewBox="0 0 24 24" aria-hidden="true">
											<circle cx="12" cy="12" r="10" fill="#FF7139" />
											<circle cx="12" cy="12" r="3" fill="#FF3E00" />
										</svg>
									</span>
									<span class="browser-glyph" aria-label="Safari">
										<svg viewBox="0 0 24 24" aria-hidden="true">
											<circle cx="12" cy="12" r="10" fill="#1A8FE3" />
											<circle cx="12" cy="12" r="7.5" fill="#fff" />
										</svg>
									</span>
									<span class="browser-glyph" aria-label="Edge">
										<svg viewBox="0 0 24 24" aria-hidden="true">
											<circle cx="12" cy="12" r="10" fill="#0078D4" />
										</svg>
									</span>
								</span>
								<span class="more-pill">{$t('integrations_hub_more')}</span>
							</div>
						</div>
					{/snippet}
					{#snippet actions()}
						<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- browser store URLs are external install links. -->
						<a class="btn ghost compact" href={extStore.href} target="_blank" rel="noopener">
							{$t(extStore.labelKey)}
						</a>
					{/snippet}
				</IntegrationConnectionCard>

				<IntegrationConnectionCard
					title="Notion"
					tagline={$t('integrations_hub_notion_tagline')}
					statusLabel={$t(notionStatus.labelKey)}
					statusVariant={notionStatus.variant}
					statusPulse={notionStatus.pulse}
					statusCheck={notionStatus.check}
					errorMessage={notionConnection?.last_error ??
						(notionConnection ? syncErrorByConnection[notionConnection.id] : notionConnectError)}
					testId="notion-connection-card"
				>
					{#snippet body()}
						<div class="moment">
							{#if notionConnection}
								{@const dbLabel = notionDatabaseLabel(notionConnection)}
								{#if dbLabel}<div class="moment-eyebrow">{dbLabel}</div>{/if}
								{#if notionConnection.last_sync_at}
									<div class="moment-stat">
										{$t('integrations_hub_last_sync_time', {
											values: { time: relativeTime(notionConnection.last_sync_at) ?? '' }
										})}
									</div>
								{:else}
									<div class="moment-muted">{$t('integrations_hub_no_sync_notion')}</div>
								{/if}
							{:else if notionAvailable}
								<div class="moment-muted">
									{$t('integrations_notion_empty_hint')}
								</div>
							{:else}
								<div class="moment-muted">
									{$t('integrations_notion_unavailable')}
								</div>
							{/if}
						</div>
					{/snippet}
					{#snippet actions()}
						{#if notionConnection}
							<button type="button" class="btn ghost compact" onclick={onOpenNotion}
								>{$t('integrations_hub_manage')}</button
							>
							<button
								type="button"
								class="btn ghost compact"
								onclick={() => onSync(notionConnection.id)}
								disabled={syncStateByConnection[notionConnection.id] === 'pending'}
							>
								{syncStateByConnection[notionConnection.id] === 'pending'
									? $t('integrations_notion_syncing')
									: $t('integrations_hub_force_resync')}
							</button>
							<button
								type="button"
								class="btn ghost compact danger"
								onclick={() => onDisconnect(notionConnection)}
							>
								{$t('integrations_disconnect')}
							</button>
						{:else}
							<button
								type="button"
								class="btn primary compact"
								disabled={!notionAvailable}
								onclick={onStartNotion}
							>
								{$t('integrations_notion_connect')}
							</button>
						{/if}
					{/snippet}
				</IntegrationConnectionCard>

				<IntegrationConnectionCard
					title="Obsidian"
					tagline={$t('integrations_hub_obsidian_tagline')}
					statusLabel={$t(obsidianStatus.labelKey)}
					statusVariant={obsidianStatus.variant}
					statusCheck={obsidianStatus.check}
					testId="obsidian-connection-card"
				>
					{#snippet body()}
						<div class="moment">
							{#if obsidianConnection?.last_sync_at}
								<div class="moment-stat">
									{$t('integrations_hub_last_sync_time', {
										values: { time: relativeTime(obsidianConnection.last_sync_at) ?? '' }
									})}
								</div>
							{:else if obsidianConnection}
								<div class="moment-muted">{$t('integrations_hub_no_sync_obsidian')}</div>
							{:else}
								<div class="moment-muted">
									{$t('integrations_hub_obsidian_setup_hint')}
								</div>
							{/if}
						</div>
					{/snippet}
					{#snippet actions()}
						<button type="button" class="btn ghost compact" onclick={onOpenObsidian}>
							{obsidianConnection
								? $t('integrations_hub_manage')
								: $t('integrations_hub_connect_obsidian')}
						</button>
						{#if obsidianConnection}
							<button
								type="button"
								class="btn ghost compact danger"
								onclick={() => onDisconnect(obsidianConnection)}
							>
								{$t('integrations_disconnect')}
							</button>
						{/if}
					{/snippet}
				</IntegrationConnectionCard>
			</div>

			<div class="coming-soon-strip">
				<span class="label">{$t('common_coming_soon')}</span>
				<span class="chip">{$t('integrations_hub_send_to_kindle')}</span>
			</div>
		</div>
	{/if}
</SettingsGroup>

<style>
	.zone-meta {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-tertiary);
		margin: 0;
	}

	.zone-meta.error {
		color: var(--destructive);
	}

	.connections-stack {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.connections-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 14px;
	}

	.moment {
		display: flex;
		flex-direction: column;
		gap: 8px;
		background: transparent;
		border-top: 0.5px solid var(--border-primary);
		padding: 10px 0 8px;
		flex: 1;
		min-height: 0;
	}

	.moment-eyebrow {
		font-size: 9.5px;
		font-weight: 600;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--text-tertiary);
	}

	.moment-stat,
	.moment-muted {
		font-size: 12px;
		color: var(--text-secondary);
	}

	.moment-muted {
		color: var(--text-tertiary);
	}

	.shortcut-list {
		display: flex;
		flex-direction: column;
	}

	.shortcut-item {
		display: grid;
		grid-template-columns: 22px 1fr auto;
		align-items: center;
		gap: 10px;
		padding: 4px 0;
		border-bottom: 0.5px solid var(--border-primary);
		background: transparent;
	}

	.shortcut-list .shortcut-item:last-child {
		border-bottom: none;
	}

	.shortcut-icon {
		width: 22px;
		height: 22px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		color: var(--text-tertiary);
		background: transparent;
		border-radius: 5px;
	}

	.shortcut-icon svg {
		width: 12px;
		height: 12px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.6;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.shortcut-meta {
		display: flex;
		flex-direction: column;
		gap: 0;
		min-width: 0;
	}

	.shortcut-title {
		font-size: 11.5px;
		font-weight: 600;
		color: var(--text-primary);
		letter-spacing: -0.01em;
		line-height: 1.2;
	}

	.shortcut-sub {
		font-size: 10px;
		color: var(--text-tertiary);
		letter-spacing: -0.005em;
		line-height: 1.25;
	}

	.shortcut-keys {
		display: inline-flex;
		gap: 2px;
	}

	.shortcut-keys kbd {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 20px;
		height: 20px;
		padding: 0 5px;
		border-radius: 4px;
		font-family: 'SF Mono', 'Fira Code', 'Menlo', ui-monospace, monospace;
		font-size: 10.5px;
		font-weight: 600;
		color: var(--text-primary);
		background: var(--bg-secondary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.works-on {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 2px 0 0;
		font-size: 10px;
		color: var(--text-tertiary);
	}

	.browsers {
		display: inline-flex;
		gap: 5px;
	}

	.browser-glyph {
		width: 14px;
		height: 14px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
	}

	.browser-glyph svg {
		width: 100%;
		height: 100%;
		display: block;
	}

	.more-pill {
		margin-left: auto;
		font-size: 10px;
		color: var(--text-secondary);
		font-weight: 500;
	}

	.coming-soon-strip {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 14px;
		background: var(--card-bg);
		border-radius: 12px;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		flex-wrap: wrap;
	}

	.label {
		font-size: 10px;
		font-weight: 700;
		letter-spacing: 0;
		text-transform: uppercase;
		color: var(--text-tertiary);
	}

	.chip {
		display: inline-flex;
		align-items: center;
		gap: 7px;
		padding: 5px 10px;
		border-radius: 8px;
		background: var(--bg-elevated);
		box-shadow: 0 0 0 0.5px var(--border-primary);
		font-size: 12px;
		font-weight: 500;
		color: var(--text-primary);
	}

	.btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 6px 12px;
		border-radius: 8px;
		font-size: 12.5px;
		font-weight: 500;
		letter-spacing: 0;
		cursor: pointer;
		border: none;
		font-family: inherit;
		white-space: nowrap;
		text-decoration: none;
	}

	.btn.ghost {
		background: transparent;
		color: var(--text-primary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.btn.primary {
		background: var(--int-ring-connected);
		color: var(--text-on-color);
	}

	.btn.danger {
		color: var(--destructive);
		box-shadow: inset 0 0 0 0.5px var(--destructive-border);
	}

	.btn.compact {
		padding: 5px 10px;
		font-size: 11.5px;
	}

	.btn:disabled {
		opacity: 0.45;
		cursor: default;
	}

	@media (max-width: 899px) {
		.connections-grid {
			grid-template-columns: 1fr;
		}
	}
</style>
