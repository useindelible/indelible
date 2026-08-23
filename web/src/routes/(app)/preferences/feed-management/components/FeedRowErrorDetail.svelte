<script lang="ts">
	import type { Feed } from '../feed-model';
	import { t } from '$lib/i18n';

	interface Props {
		feed: Feed;
		onRetry: (id: string) => void;
		onToggleFeed: (id: string) => void;
	}

	let { feed, onRetry, onToggleFeed }: Props = $props();
</script>

<tr class="error-detail-row">
	<td colspan="6">
		<div class="error-detail">
			<div class="error-icon">
				<svg viewBox="0 0 24 24">
					<path d="M12 9v4" />
					<circle cx="12" cy="17" r="0.5" />
					<path
						d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"
					/>
				</svg>
			</div>
			<div class="error-body">
				<div class="error-title">{$t('feed_management_unreachable')}</div>
				<div class="error-msg">{feed.errorMessage}</div>
			</div>
			<div class="error-actions">
				<button type="button" class="btn amber-soft compact" onclick={() => onRetry(feed.id)}>
					<svg viewBox="0 0 24 24">
						<path d="M3 7v6h6" />
						<path d="M21 17a9 9 0 0 0-15-6.7L3 13" />
					</svg>
					{$t('feed_management_retry_now')}
				</button>
				<button type="button" class="btn ghost compact" onclick={() => onToggleFeed(feed.id)}>
					{$t('feed_management_pause')}
				</button>
			</div>
		</div>
	</td>
</tr>

<style>
	.error-detail-row td {
		padding: 0;
		background: var(--feed-table-error-bg);
		border-top: 0.5px solid var(--feed-table-error-border);
	}

	.error-detail {
		padding: 12px 14px 14px 53px;
		display: flex;
		align-items: flex-start;
		gap: 12px;
	}

	.error-icon {
		width: 22px;
		height: 22px;
		border-radius: 50%;
		background: var(--feed-status-error-bg);
		color: var(--feed-status-error-text);
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		margin-top: 1px;
	}

	.error-icon svg,
	.btn svg {
		stroke: currentColor;
		fill: none;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.error-icon svg {
		width: 12px;
		height: 12px;
		stroke-width: 2;
	}

	.error-body {
		flex: 1;
		min-width: 0;
	}

	.error-title {
		font-size: 12.5px;
		font-weight: 600;
		color: var(--feed-status-error-text);
		margin-bottom: 3px;
		letter-spacing: 0;
		font-family: var(--font-sans);
	}

	.error-msg {
		font-size: 12px;
		color: var(--feed-code-text);
		font-family: ui-monospace, 'SF Mono', Menlo, monospace;
		background: var(--feed-code-bg);
		padding: 6px 9px;
		border-radius: 6px;
		word-break: break-word;
		line-height: 1.4;
	}

	.error-actions {
		display: flex;
		gap: 6px;
		flex-shrink: 0;
	}

	.btn {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 6px 12px;
		border-radius: 8px;
		font-family: var(--font-sans);
		font-size: 12.5px;
		font-weight: 500;
		border: 0;
		cursor: pointer;
		transition:
			background 140ms,
			transform 140ms;
		white-space: nowrap;
		letter-spacing: 0;
	}

	.btn.compact {
		padding: 4px 9px;
		font-size: 11.5px;
	}

	.btn.ghost {
		background: transparent;
		color: var(--text-primary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.btn.ghost:hover {
		background: var(--fill-hover);
	}

	.btn.amber-soft {
		background: var(--feed-amber-soft);
		color: var(--feed-amber);
	}

	.btn.amber-soft:hover {
		background: var(--feed-chip-active-bg);
	}

	.btn svg {
		width: 11px;
		height: 11px;
		stroke-width: 1.8;
	}

	/* The wide left inset aligns with the feed column on desktop; on narrow
	   cards it wastes the row, so the detail hugs the card edge and the
	   actions drop below the message. */
	@container feeds-card (max-width: 439px) {
		.error-detail {
			padding: 12px 14px;
			flex-wrap: wrap;
		}

		.error-actions {
			width: 100%;
			justify-content: flex-end;
		}
	}
</style>
