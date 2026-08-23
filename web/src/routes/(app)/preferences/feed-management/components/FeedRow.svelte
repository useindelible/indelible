<script lang="ts">
	import FeedRowErrorDetail from './FeedRowErrorDetail.svelte';
	import { formatSchedule, isFresh, type Feed } from '../feed-model';
	import { t } from '$lib/i18n';

	interface Props {
		feed: Feed;
		menuOpen: boolean;
		onToggleAutoSave: (id: string) => void;
		onToggleFeed: (id: string) => void;
		onToggleMenu: (id: string, event: MouseEvent) => void;
		onCloseMenu: () => void;
		onEdit: (id: string) => void;
		onRetry: (id: string) => void;
		onDelete: (id: string) => void;
	}

	let {
		feed,
		menuOpen,
		onToggleAutoSave,
		onToggleFeed,
		onToggleMenu,
		onCloseMenu,
		onEdit,
		onRetry,
		onDelete
	}: Props = $props();

	function menuAction(callback: (id: string) => void) {
		onCloseMenu();
		callback(feed.id);
	}
</script>

<tr data-feed-status={feed.status} class:error-row={feed.status === 'error'}>
	<td>
		<div class="feed-cell">
			<div class="feed-icon" data-icon-key={feed.iconKey}>{feed.initials}</div>
			<div class="feed-meta">
				<div class="feed-name">{feed.name}</div>
				<div class="feed-secondary">
					<span class="feed-domain">{feed.domain}</span>
				</div>
			</div>
		</div>
	</td>
	<td>
		{#if feed.status === 'active'}
			<span class="status-pill active">
				<svg viewBox="0 0 24 24"><path d="M5 12l4 4 10-10" /></svg>
				{$t('feed_management_active')}
			</span>
		{:else if feed.status === 'paused'}
			<span class="status-pill paused">
				<svg viewBox="0 0 24 24">
					<rect x="6" y="5" width="4" height="14" />
					<rect x="14" y="5" width="4" height="14" />
				</svg>
				{$t('feed_management_paused')}
			</span>
		{:else}
			<span class="status-pill error">
				<span class="err-dot"></span>
				{$t('feed_management_error')}
			</span>
		{/if}
	</td>
	<td>
		<button
			type="button"
			class="toggle"
			class:on={feed.autoSave}
			role="switch"
			aria-checked={feed.autoSave}
			aria-label={feed.autoSave
				? $t('feed_management_disable_auto_save')
				: $t('feed_management_enable_auto_save')}
			onclick={() => onToggleAutoSave(feed.id)}
		>
			<span class="toggle-track"></span>
		</button>
	</td>
	<td class="schedule-cell">
		{#if feed.status === 'paused'}
			<span>-</span>
		{:else}
			<strong>{formatSchedule(feed.pollIntervalOverride)}</strong>
		{/if}
	</td>
	<td class="when" class:fresh={isFresh(feed.lastFetched)}>
		{feed.lastFetched}
	</td>
	<td class="right">
		<div class="kebab-wrap">
			<button
				type="button"
				class="kebab"
				class:open={menuOpen}
				aria-label={$t('feed_management_actions')}
				aria-haspopup="menu"
				aria-expanded={menuOpen}
				onclick={(event) => onToggleMenu(feed.id, event)}
			>
				<svg viewBox="0 0 24 24">
					<circle cx="12" cy="5" r="1.4" />
					<circle cx="12" cy="12" r="1.4" />
					<circle cx="12" cy="19" r="1.4" />
				</svg>
			</button>
			{#if menuOpen}
				<div
					class="kebab-menu"
					role="menu"
					onclick={(event) => event.stopPropagation()}
					onkeydown={() => {}}
					tabindex="-1"
				>
					<button
						type="button"
						class="kebab-item"
						role="menuitem"
						onclick={() => menuAction(onEdit)}
					>
						<svg viewBox="0 0 24 24">
							<path d="M12 20h9" />
							<path d="M16.5 3.5a2.121 2.121 0 1 1 3 3L7 19l-4 1 1-4 12.5-12.5z" />
						</svg>
						{$t('feed_management_edit_details')}
					</button>
					{#if feed.status === 'paused'}
						<button
							type="button"
							class="kebab-item"
							role="menuitem"
							onclick={() => menuAction(onToggleFeed)}
						>
							<svg viewBox="0 0 24 24"><polygon points="5 3 19 12 5 21 5 3" /></svg>
							{$t('feed_management_resume_polling')}
						</button>
					{:else}
						<button
							type="button"
							class="kebab-item"
							role="menuitem"
							onclick={() => menuAction(onToggleFeed)}
						>
							<svg viewBox="0 0 24 24">
								<rect x="6" y="5" width="4" height="14" />
								<rect x="14" y="5" width="4" height="14" />
							</svg>
							{$t('feed_management_pause_feed')}
						</button>
					{/if}
					{#if feed.status === 'error'}
						<button
							type="button"
							class="kebab-item"
							role="menuitem"
							onclick={() => menuAction(onRetry)}
						>
							<svg viewBox="0 0 24 24">
								<polyline points="23 4 23 10 17 10" />
								<path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
							</svg>
							{$t('feed_management_retry_now')}
						</button>
					{/if}
					<div class="kebab-divider"></div>
					<button
						type="button"
						class="kebab-item danger"
						role="menuitem"
						onclick={() => menuAction(onDelete)}
					>
						<svg viewBox="0 0 24 24">
							<path d="M3 6h18" />
							<path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
							<path d="M6 6l1 14a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-14" />
						</svg>
						{$t('email_unsubscribe')}
					</button>
				</div>
			{/if}
		</div>
	</td>
</tr>

{#if feed.status === 'error' && feed.errorMessage}
	<FeedRowErrorDetail {feed} {onRetry} {onToggleFeed} />
{/if}

<style>
	tr {
		border-bottom: 0.5px solid var(--border-hairline);
		transition: background 120ms;
	}

	tr:hover {
		background: var(--feed-table-row-hover);
	}

	tr.error-row {
		background: var(--feed-table-error-bg);
	}

	td {
		padding: 12px 14px;
		color: var(--text-primary);
		vertical-align: middle;
	}

	td.right {
		text-align: right;
	}

	tr[data-feed-status='paused'] .feed-name {
		opacity: 0.78;
	}

	.feed-cell {
		display: flex;
		align-items: center;
		gap: 11px;
		min-width: 0;
	}

	.feed-icon {
		width: 30px;
		height: 30px;
		border-radius: 8px;
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 12px;
		font-weight: 700;
		letter-spacing: -0.02em;
	}

	.feed-icon[data-icon-key='blue'] {
		background: var(--feed-icon-blue-bg);
		color: var(--feed-icon-blue-fg);
	}
	.feed-icon[data-icon-key='green'] {
		background: var(--feed-icon-green-bg);
		color: var(--feed-icon-green-fg);
	}
	.feed-icon[data-icon-key='orange'] {
		background: var(--feed-icon-orange-bg);
		color: var(--feed-icon-orange-fg);
	}
	.feed-icon[data-icon-key='rose'] {
		background: var(--feed-icon-rose-bg);
		color: var(--feed-icon-rose-fg);
	}
	.feed-icon[data-icon-key='purple'] {
		background: var(--feed-icon-purple-bg);
		color: var(--feed-icon-purple-fg);
	}
	.feed-icon[data-icon-key='cyan'] {
		background: var(--feed-icon-cyan-bg);
		color: var(--feed-icon-cyan-fg);
	}
	.feed-icon[data-icon-key='red'] {
		background: var(--feed-icon-red-bg);
		color: var(--feed-icon-red-fg);
	}
	.feed-icon[data-icon-key='teal'] {
		background: var(--feed-icon-teal-bg);
		color: var(--feed-icon-teal-fg);
	}

	.feed-meta {
		min-width: 0;
	}

	.feed-name {
		font-weight: 500;
		color: var(--text-primary);
		letter-spacing: -0.01em;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 240px;
	}

	.feed-secondary {
		display: flex;
		align-items: center;
		gap: 6px;
		max-width: 260px;
		min-width: 0;
	}

	.feed-domain {
		font-size: 11.5px;
		color: var(--text-tertiary);
		letter-spacing: -0.005em;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.status-pill {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 3px 9px;
		border-radius: 980px;
		font-size: 11px;
		font-weight: 600;
		letter-spacing: -0.005em;
		font-family: var(--font-sans);
		white-space: nowrap;
	}

	.status-pill svg {
		width: 10px;
		height: 10px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2.2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.status-pill.active {
		background: var(--feed-status-active-bg);
		color: var(--feed-status-active-text);
	}

	.status-pill.paused {
		background: var(--feed-status-paused-bg);
		color: var(--feed-status-paused-text);
	}

	.status-pill.error {
		background: var(--feed-status-error-bg);
		color: var(--feed-status-error-text);
	}

	.err-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: currentColor;
		animation: live-pulse 1.6s ease-out infinite;
	}

	@keyframes live-pulse {
		0% {
			box-shadow: 0 0 0 0 var(--feed-pulse-color);
		}
		70% {
			box-shadow: 0 0 0 8px var(--feed-pulse-fade);
		}
		100% {
			box-shadow: 0 0 0 0 var(--feed-pulse-fade);
		}
	}

	.toggle {
		display: inline-flex;
		align-items: center;
		cursor: pointer;
		flex-shrink: 0;
		background: none;
		border: 0;
		padding: 0;
	}

	.toggle-track {
		width: 32px;
		height: 19px;
		border-radius: 980px;
		background: var(--bg-tertiary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		position: relative;
		transition: background 160ms;
		display: block;
	}

	.toggle-track::after {
		content: '';
		position: absolute;
		left: 2px;
		top: 2px;
		width: 15px;
		height: 15px;
		border-radius: 50%;
		background: var(--text-on-color);
		box-shadow: var(--feed-toggle-thumb-shadow);
		transition: left 180ms;
	}

	.toggle.on .toggle-track {
		background: var(--feed-amber);
	}

	.toggle.on .toggle-track::after {
		left: 15px;
	}

	.schedule-cell,
	.when {
		color: var(--text-secondary);
		font-size: 12.5px;
		white-space: nowrap;
		font-variant-numeric: tabular-nums;
		letter-spacing: -0.005em;
	}

	.schedule-cell strong {
		color: var(--text-primary);
		font-weight: 500;
	}

	.when.fresh {
		color: var(--feed-status-active-text);
	}

	.kebab-wrap {
		position: relative;
		display: inline-block;
	}

	.kebab {
		width: 26px;
		height: 26px;
		border-radius: 7px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		color: var(--text-tertiary);
		background: transparent;
		border: 0;
		cursor: pointer;
		transition:
			background 120ms,
			color 120ms;
	}

	.kebab:hover,
	.kebab.open {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.kebab.open {
		background: var(--feed-amber-soft);
		color: var(--feed-amber);
	}

	.kebab svg,
	.kebab-item svg {
		stroke: currentColor;
		fill: none;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.kebab svg {
		width: 14px;
		height: 14px;
		stroke-width: 1.8;
	}

	.kebab-menu {
		position: absolute;
		top: calc(100% + 6px);
		right: 0;
		min-width: 200px;
		background: var(--bg-elevated);
		border-radius: 12px;
		box-shadow:
			0 12px 32px rgba(0, 0, 0, 0.16),
			0 0 0 0.5px var(--border-primary);
		padding: 5px;
		z-index: 50;
	}

	.kebab-item {
		display: flex;
		align-items: center;
		gap: 9px;
		width: 100%;
		padding: 8px 10px;
		background: transparent;
		border: 0;
		border-radius: 8px;
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-primary);
		letter-spacing: -0.005em;
		cursor: pointer;
		text-align: left;
		transition: background 120ms;
	}

	.kebab-item:hover {
		background: var(--fill-hover);
	}

	.kebab-item svg {
		width: 14px;
		height: 14px;
		stroke: var(--text-tertiary);
		stroke-width: 1.8;
		flex-shrink: 0;
	}

	.kebab-item:hover svg {
		stroke: var(--text-primary);
	}

	.kebab-item.danger {
		color: var(--feed-status-error-text);
	}

	.kebab-item.danger svg {
		stroke: var(--feed-status-error-text);
	}

	.kebab-item.danger:hover {
		background: var(--feed-status-error-bg);
	}

	.kebab-divider {
		height: 0.5px;
		background: var(--border-primary);
		margin: 4px 6px;
	}

	/* Mirrors the header fold in FeedSubscriptionsTable: Schedule and Last
	   updated drop first, Auto-save follows on phones. */
	@container feeds-card (max-width: 739px) {
		td {
			padding: 10px 8px;
		}

		td:first-child {
			padding-left: 14px;
		}

		td:last-child {
			padding-right: 14px;
		}

		td.schedule-cell,
		td.when {
			display: none;
		}
	}

	@container feeds-card (max-width: 439px) {
		td:nth-child(3) {
			display: none;
		}

		.feed-name,
		.feed-secondary {
			max-width: 140px;
		}
	}
</style>
