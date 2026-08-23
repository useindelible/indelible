<script lang="ts">
	import type { DocumentListEntry } from '$lib/api';
	import { date, t } from '$lib/i18n';

	type TrashItem = DocumentListEntry;

	interface Props {
		item: TrashItem;
		onRestore: (id: string) => void;
		onDeleteClick: (item: TrashItem) => void;
		restoring: boolean;
	}

	let { item, onRestore, onDeleteClick, restoring }: Props = $props();

	function gradientClass(itemType: string): string {
		switch (itemType) {
			case 'video':
				return 'red-gradient';
			case 'podcast':
				return 'purple-gradient';
			case 'email':
				return 'orange-gradient';
			case 'pdf':
				return 'teal-gradient';
			case 'tweet':
				return 'teal-gradient';
			case 'book':
				return 'green-gradient';
			default:
				return 'blue-gradient';
		}
	}

	function thumbEmoji(itemType: string): string {
		switch (itemType) {
			case 'video':
				return '\u{1F3AC}';
			case 'podcast':
				return '\u{1F3A7}';
			case 'email':
				return '\u{2709}\u{FE0F}';
			case 'pdf':
				return '\u{1F4C4}';
			case 'tweet':
				return '\u{1F426}';
			case 'book':
				return '\u{1F4D6}';
			default:
				return '\u{1F4F0}';
		}
	}

	function faviconUrl(domain: string): string {
		return `https://www.google.com/s2/favicons?domain=${domain}&sz=32`;
	}

	function relativeDeletedDate(iso: string | null | undefined): string {
		if (!iso) return '';
		const deleted = new Date(iso);
		const now = new Date();
		const diffMs = now.getTime() - deleted.getTime();
		const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
		if (diffDays === 0) return $t('trash_deleted_today');
		if (diffDays <= 30) return $t('trash_deleted_days_ago', { values: { count: diffDays } });
		return $t('trash_deleted_date', {
			values: { date: $date(deleted, { month: 'short', day: 'numeric', year: 'numeric' }) }
		});
	}

	function formatReadTime(minutes: number | null | undefined): string | null {
		if (minutes == null || minutes === 0) return null;
		return $t('common_reading_time_minutes', { values: { minutes } });
	}
</script>

<div class="trash-row">
	<div class="trash-thumb {gradientClass(item.item_type)}">
		{#if item.thumbnail_url}
			<img src={item.thumbnail_url} alt="" class="thumb-img" />
		{:else}
			<span class="thumb-emoji">{thumbEmoji(item.item_type)}</span>
		{/if}
	</div>
	<div class="trash-item-content">
		<div class="trash-item-title">{item.title}</div>
		{#if item.summary ?? item.excerpt}
			<div class="trash-item-excerpt">{item.summary ?? item.excerpt}</div>
		{/if}
		<div class="trash-source-row">
			{#if item.domain}
				<img
					src={faviconUrl(item.domain)}
					alt=""
					class="favicon"
					width="14"
					height="14"
					onerror={(e) => {
						const el = e.currentTarget as HTMLImageElement;
						el.style.display = 'none';
					}}
				/>
				<span class="trash-source">{item.domain}</span>
			{/if}
			{#if formatReadTime(item.reading_time_minutes)}
				<span class="trash-meta" class:has-source={!!item.domain}
					>{formatReadTime(item.reading_time_minutes)}</span
				>
			{/if}
		</div>
	</div>
	<div class="trash-right">
		<span class="trash-deleted-date">{relativeDeletedDate(item.deleted_at)}</span>
		<div class="trash-actions">
			<button class="restore-btn" onclick={() => onRestore(item.id)} disabled={restoring}>
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<polyline points="1 4 1 10 7 10" />
					<path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
				</svg>
				{restoring ? $t('trash_restoring') : $t('trash_restore')}
			</button>
			<button class="delete-perm-btn" onclick={() => onDeleteClick(item)}
				>{$t('common_delete')}</button
			>
		</div>
	</div>
</div>

<style>
	.trash-row {
		display: flex;
		align-items: flex-start;
		gap: 16px;
		padding: 14px 20px;
		border-bottom: 0.5px solid var(--border-primary);
		position: relative;
		transition: background 120ms ease;
	}
	.trash-row:hover {
		background: var(--fill-hover);
	}

	.trash-thumb {
		width: 56px;
		height: 56px;
		border-radius: var(--radius-lg);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		overflow: hidden;
	}
	.thumb-img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		border-radius: inherit;
	}
	.thumb-emoji {
		font-size: 26px;
		line-height: 1;
	}

	.trash-thumb.blue-gradient {
		background: linear-gradient(135deg, rgba(0, 113, 227, 0.12), rgba(0, 113, 227, 0.26));
		border: 0.5px solid rgba(0, 113, 227, 0.18);
	}
	:global([data-theme='dark']) .trash-thumb.blue-gradient {
		background: linear-gradient(135deg, rgba(10, 132, 255, 0.2), rgba(10, 132, 255, 0.38));
		border-color: rgba(10, 132, 255, 0.3);
	}
	.trash-thumb.purple-gradient {
		background: linear-gradient(135deg, rgba(175, 82, 222, 0.12), rgba(175, 82, 222, 0.26));
		border: 0.5px solid rgba(175, 82, 222, 0.18);
	}
	:global([data-theme='dark']) .trash-thumb.purple-gradient {
		background: linear-gradient(135deg, rgba(175, 82, 222, 0.2), rgba(175, 82, 222, 0.38));
		border-color: rgba(175, 82, 222, 0.3);
	}
	.trash-thumb.green-gradient {
		background: linear-gradient(135deg, rgba(52, 199, 89, 0.12), rgba(52, 199, 89, 0.26));
		border: 0.5px solid rgba(52, 199, 89, 0.18);
	}
	:global([data-theme='dark']) .trash-thumb.green-gradient {
		background: linear-gradient(135deg, rgba(52, 199, 89, 0.2), rgba(52, 199, 89, 0.38));
		border-color: rgba(52, 199, 89, 0.3);
	}
	.trash-thumb.orange-gradient {
		background: linear-gradient(135deg, rgba(255, 149, 0, 0.12), rgba(255, 149, 0, 0.26));
		border: 0.5px solid rgba(255, 149, 0, 0.18);
	}
	:global([data-theme='dark']) .trash-thumb.orange-gradient {
		background: linear-gradient(135deg, rgba(255, 149, 0, 0.2), rgba(255, 149, 0, 0.38));
		border-color: rgba(255, 149, 0, 0.3);
	}
	.trash-thumb.red-gradient {
		background: linear-gradient(135deg, rgba(255, 59, 48, 0.12), rgba(255, 59, 48, 0.26));
		border: 0.5px solid rgba(255, 59, 48, 0.18);
	}
	:global([data-theme='dark']) .trash-thumb.red-gradient {
		background: linear-gradient(135deg, rgba(255, 69, 58, 0.2), rgba(255, 69, 58, 0.38));
		border-color: rgba(255, 69, 58, 0.3);
	}
	.trash-thumb.teal-gradient {
		background: linear-gradient(135deg, rgba(90, 200, 250, 0.12), rgba(90, 200, 250, 0.26));
		border: 0.5px solid rgba(90, 200, 250, 0.18);
	}
	:global([data-theme='dark']) .trash-thumb.teal-gradient {
		background: linear-gradient(135deg, rgba(90, 200, 250, 0.2), rgba(90, 200, 250, 0.38));
		border-color: rgba(90, 200, 250, 0.3);
	}

	.trash-item-content {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 3px;
	}
	.trash-item-title {
		font-size: 14px;
		font-weight: 600;
		letter-spacing: -0.01em;
		line-height: 1.4;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.trash-item-excerpt {
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.45;
		color: var(--text-secondary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.trash-source-row {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-top: 2px;
	}
	.favicon {
		width: 14px;
		height: 14px;
		border-radius: 3px;
		flex-shrink: 0;
	}
	.trash-source {
		font-size: 11.5px;
		font-weight: 500;
		color: var(--text-secondary);
	}
	.trash-meta {
		font-size: 11.5px;
		font-weight: 400;
		color: var(--text-tertiary);
	}
	.trash-meta.has-source::before {
		content: '\00b7';
		margin: 0 4px;
	}

	.trash-right {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 8px;
		flex-shrink: 0;
		align-self: center;
	}
	.trash-deleted-date {
		font-size: 11px;
		font-weight: 400;
		color: var(--text-tertiary);
		white-space: nowrap;
	}
	.trash-actions {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.restore-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 5px;
		padding: 5px 12px;
		border-radius: 6px;
		border: 1px solid var(--accent);
		background: transparent;
		color: var(--accent);
		font-family: inherit;
		font-size: 12px;
		font-weight: 500;
		letter-spacing: -0.01em;
		min-height: 28px;
		cursor: pointer;
		transition: background 120ms ease;
		white-space: nowrap;
		flex-shrink: 0;
	}
	.restore-btn:hover:not(:disabled) {
		background: var(--fill-selected);
	}
	.restore-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
	.restore-btn svg {
		width: 13px;
		height: 13px;
		min-width: 13px;
		min-height: 13px;
		flex-shrink: 0;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
	.delete-perm-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		font-family: inherit;
		font-size: 12px;
		font-weight: 500;
		letter-spacing: -0.01em;
		min-height: 28px;
		color: var(--destructive);
		cursor: pointer;
		background: transparent;
		border: 1px solid var(--destructive);
		padding: 5px 12px;
		border-radius: 6px;
		white-space: nowrap;
		transition: background 120ms ease;
		flex-shrink: 0;
	}
	.delete-perm-btn:hover {
		background: var(--fill-danger);
	}

	/* ---- Responsive: mobile reflow ---- */

	@media (max-width: 599px) {
		.trash-row {
			padding: 14px 16px;
			gap: 12px;
			flex-wrap: wrap;
		}

		.trash-thumb {
			width: 44px;
			height: 44px;
		}

		.thumb-emoji {
			font-size: 20px;
		}

		/* Date + actions move to their own row under the text column instead of
		   squeezing the title against the right edge. */
		.trash-right {
			flex-direction: row;
			align-items: center;
			justify-content: space-between;
			width: 100%;
			margin-left: 56px;
		}
	}
</style>
