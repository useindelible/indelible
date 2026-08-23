<script lang="ts">
	import type { DocumentListEntry } from '$lib/api';
	import { formatReadingTime } from '$lib/utils/format';
	import { date, locale, t, type MessageKey } from '$lib/i18n';
	import { relativeTime } from '$lib/utils/relative-time';

	interface Props {
		item: DocumentListEntry;
	}

	let { item }: Props = $props();

	function formatItemType(raw: string): string {
		const keys: Partial<Record<string, MessageKey>> = {
			article: 'library_filter_value_article',
			book: 'library_filter_value_book',
			email: 'library_filter_value_email',
			pdf: 'library_filter_value_pdf',
			podcast: 'library_filter_value_podcast',
			tweet: 'library_filter_value_tweet',
			video: 'library_filter_value_video'
		};
		const key = keys[raw];
		if (key) return $t(key);
		return raw.charAt(0).toUpperCase() + raw.slice(1).toLowerCase();
	}

	function formatDate(iso: string | null | undefined): string {
		if (!iso) return '—';
		const d = new Date(iso);
		return $date(d, { month: 'short', day: 'numeric', year: 'numeric' });
	}

	function formatLength(minutes: number | null | undefined): string {
		if (!minutes) return '—';
		return $t('library_reading_length', { values: { time: formatReadingTime(minutes) } });
	}

	function formatDuration(seconds: number): string {
		const h = Math.floor(seconds / 3600);
		const m = Math.floor((seconds % 3600) / 60);
		const s = seconds % 60;
		if (h > 0) {
			return $t('library_duration_hours_minutes_seconds', {
				values: { hours: h, minutes: m, seconds: s }
			});
		}
		if (m > 0) {
			return $t('library_duration_minutes_seconds', { values: { minutes: m, seconds: s } });
		}
		return $t('library_duration_seconds', { values: { seconds: s } });
	}

	const isVideo = $derived(item.item_type === 'video');

	function formatLanguage(code: string | null | undefined): string {
		if (!code) return '—';
		try {
			return new Intl.DisplayNames([$locale ?? 'en'], { type: 'language' }).of(code) ?? code;
		} catch {
			return code;
		}
	}

	const progressPercent = $derived(Math.round(item.progress_percent ?? 0));
</script>

<div class="metadata-section">
	<div class="section-heading">{$t('common_metadata')}</div>
	<div class="metadata-table">
		<div class="metadata-row">
			<span class="metadata-label">{$t('common_type')}</span>
			<span class="metadata-value">{formatItemType(item.item_type)}</span>
		</div>
		<div class="metadata-row">
			<span class="metadata-label">{$t('common_domain')}</span>
			<span class="metadata-value">{item.domain ?? '—'}</span>
		</div>
		<div class="metadata-row">
			<span class="metadata-label">{$t('common_published')}</span>
			<span class="metadata-value">{formatDate(item.published_at)}</span>
		</div>
		{#if isVideo && item.video_duration_seconds}
			<div class="metadata-row">
				<span class="metadata-label">{$t('library_metadata_duration')}</span>
				<span class="metadata-value">{formatDuration(item.video_duration_seconds)}</span>
			</div>
		{:else}
			<div class="metadata-row">
				<span class="metadata-label">{$t('library_metadata_length')}</span>
				<span class="metadata-value">{formatLength(item.reading_time_minutes)}</span>
			</div>
			<div class="metadata-row">
				<span class="metadata-label">{$t('library_metadata_words')}</span>
				<span class="metadata-value">
					{item.word_count ? $t('library_word_count', { values: { count: item.word_count } }) : '—'}
				</span>
			</div>
		{/if}
		<div class="metadata-row">
			<span class="metadata-label">{$t('library_metadata_saved')}</span>
			<span class="metadata-value">{relativeTime(item.saved_at) ?? formatDate(item.saved_at)}</span>
		</div>
		<div class="metadata-row">
			<span class="metadata-label">{$t('library_metadata_progress')}</span>
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
			<span class="metadata-label">{$t('library_metadata_last_read')}</span>
			<span class="metadata-value"
				>{item.last_read_at
					? (relativeTime(item.last_read_at) ?? formatDate(item.last_read_at))
					: '—'}</span
			>
		</div>
		<div class="metadata-row">
			<span class="metadata-label">{$t('common_language')}</span>
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
