<script lang="ts">
	import type { DocumentListEntry } from '$lib/api';
	import { formatReadingTime } from '$lib/utils/format';

	interface Props {
		item: DocumentListEntry;
	}

	let { item }: Props = $props();

	function formatItemType(raw: string): string {
		return raw.charAt(0).toUpperCase() + raw.slice(1).toLowerCase();
	}

	function formatDate(iso: string | null | undefined): string {
		if (!iso) return '—';
		const d = new Date(iso);
		return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
	}

	function formatRelativeDate(iso: string): string {
		const diff = Math.max(0, Date.now() - new Date(iso).getTime());
		const minutes = Math.floor(diff / 60_000);
		if (minutes < 60) return minutes <= 1 ? 'just now' : `${minutes} minutes ago`;
		const hours = Math.floor(minutes / 60);
		if (hours < 24) return hours === 1 ? '1 hour ago' : `${hours} hours ago`;
		const days = Math.floor(hours / 24);
		if (days < 30) return days === 1 ? '1 day ago' : `${days} days ago`;
		return formatDate(iso);
	}

	function formatLength(minutes: number | null | undefined): string {
		if (!minutes) return '—';
		return `${formatReadingTime(minutes)} read`;
	}

	function formatDuration(seconds: number): string {
		const h = Math.floor(seconds / 3600);
		const m = Math.floor((seconds % 3600) / 60);
		const s = seconds % 60;
		if (h > 0) return `${h}h ${m}m ${s}s`;
		if (m > 0) return `${m} min ${s} sec`;
		return `${s} sec`;
	}

	const isVideo = $derived(item.item_type === 'video');

	function formatLanguage(code: string | null | undefined): string {
		if (!code) return '—';
		try {
			return new Intl.DisplayNames(['en'], { type: 'language' }).of(code) ?? code;
		} catch {
			return code;
		}
	}

	const progressPercent = $derived(Math.round(item.progress_percent ?? 0));
</script>

<div class="metadata-section">
	<div class="section-heading">Metadata</div>
	<div class="metadata-table">
		<div class="metadata-row">
			<span class="metadata-label">Type</span>
			<span class="metadata-value">{formatItemType(item.item_type)}</span>
		</div>
		<div class="metadata-row">
			<span class="metadata-label">Domain</span>
			<span class="metadata-value">{item.domain ?? '—'}</span>
		</div>
		<div class="metadata-row">
			<span class="metadata-label">Published</span>
			<span class="metadata-value">{formatDate(item.published_at)}</span>
		</div>
		{#if isVideo && item.video_duration_seconds}
			<div class="metadata-row">
				<span class="metadata-label">Duration</span>
				<span class="metadata-value">{formatDuration(item.video_duration_seconds)}</span>
			</div>
		{:else}
			<div class="metadata-row">
				<span class="metadata-label">Length</span>
				<span class="metadata-value">{formatLength(item.reading_time_minutes)}</span>
			</div>
			<div class="metadata-row">
				<span class="metadata-label">Words</span>
				<span class="metadata-value"
					>{item.word_count ? item.word_count.toLocaleString() + ' words' : '—'}</span
				>
			</div>
		{/if}
		<div class="metadata-row">
			<span class="metadata-label">Saved</span>
			<span class="metadata-value">{formatRelativeDate(item.saved_at)}</span>
		</div>
		<div class="metadata-row">
			<span class="metadata-label">Progress</span>
			<span class="metadata-value">
				<span class="progress-inline">
					{progressPercent}%
					<span
						class="progress-bar-sm"
						role="progressbar"
						aria-valuenow={progressPercent}
						aria-valuemin={0}
						aria-valuemax={100}
					>
						<span class="progress-fill-sm" style="width:{progressPercent}%"></span>
					</span>
				</span>
			</span>
		</div>
		<div class="metadata-row">
			<span class="metadata-label">Last read</span>
			<span class="metadata-value"
				>{item.last_read_at ? formatRelativeDate(item.last_read_at) : '—'}</span
			>
		</div>
		<div class="metadata-row">
			<span class="metadata-label">Language</span>
			<span class="metadata-value">{formatLanguage(item.language)}</span>
		</div>
	</div>
</div>

<style>
	.metadata-section {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.section-heading {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		line-height: 1.2;
	}

	.metadata-table {
		display: flex;
		flex-direction: column;
	}

	.metadata-row {
		display: flex;
		align-items: baseline;
		padding: 9px 0;
		border-bottom: 0.5px solid var(--border-primary);
	}

	.metadata-row:last-child {
		border-bottom: none;
	}

	.metadata-label {
		width: 90px;
		flex-shrink: 0;
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
	}

	.metadata-value {
		font-size: 13px;
		font-weight: 400;
		color: var(--text-primary);
		flex: 1;
		min-width: 0;
		word-break: break-word;
	}

	.progress-inline {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}

	.progress-bar-sm {
		width: 60px;
		height: 3px;
		border-radius: 2px;
		background: var(--fill-hover);
		display: inline-block;
		overflow: hidden;
	}

	.progress-fill-sm {
		height: 100%;
		border-radius: 2px;
		background: var(--accent);
		display: block;
	}
</style>
