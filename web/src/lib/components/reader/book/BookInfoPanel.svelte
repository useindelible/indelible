<script lang="ts">
	import type { DocumentListEntry } from '$lib/api';
	import SummarySection from '$lib/components/library/SummarySection.svelte';
	import type { BookMetadata } from './book-source';
	import { date, locale, t } from '$lib/i18n';
	import { relativeTime } from '$lib/utils/relative-time';

	interface Props {
		item: DocumentListEntry;
		bookMetadata: BookMetadata;
		progress: number;
	}

	let { item, bookMetadata, progress }: Props = $props();

	const formattedDate = $derived(
		item.published_at
			? $date(new Date(item.published_at), {
					year: 'numeric',
					month: 'short',
					day: 'numeric'
				})
			: null
	);

	const savedAgo = $derived.by(() => {
		void $locale;
		return relativeTime(item.saved_at) ?? $date(new Date(item.saved_at));
	});

	const lastReadLabel = $derived.by(() => {
		if (!item.last_read_at) return $t('reader_never');
		void $locale;
		return relativeTime(item.last_read_at) ?? $date(new Date(item.last_read_at));
	});

	const totalPages = $derived(bookMetadata.estimatedPages ?? 0);
	const pagesRead = $derived(Math.round((progress / 100) * totalPages));
	const progressLabel = $derived(
		totalPages > 0
			? $t('reader_progress_pages', {
					values: { progress: Math.round(progress), current: pagesRead, total: totalPages }
				})
			: $t('reader_percent', { values: { progress: Math.round(progress) } })
	);

	const readingTimeEstimate = $derived(
		bookMetadata.totalWords
			? $t('reader_hours_estimated', {
					values: { hours: Math.round(bookMetadata.totalWords / 250 / 60) }
				})
			: null
	);

	const remainingEstimate = $derived(
		bookMetadata.totalWords && progress > 0
			? $t('reader_hours_estimated', {
					values: {
						hours: Math.round(((100 - progress) / 100) * (bookMetadata.totalWords / 250 / 60))
					}
				})
			: readingTimeEstimate
	);

	const itemTypeLabel = $derived(
		item.item_type === 'book'
			? $t('reader_type_epub')
			: item.item_type === 'pdf'
				? $t('reader_view_pdf')
				: item.item_type
	);

	const languageLabel = $derived.by(() => {
		if (!bookMetadata.language) return null;
		try {
			return (
				new Intl.DisplayNames($locale ?? 'en', { type: 'language' }).of(bookMetadata.language) ??
				bookMetadata.language
			);
		} catch {
			return bookMetadata.language;
		}
	});
</script>

<div class="detail-content">
	<div class="detail-title">{item.title}</div>
	<div class="detail-domain">{item.author ?? bookMetadata.author ?? ''}</div>

	<SummarySection summary={item.summary} excerpt={item.excerpt} />

	<div class="detail-section">
		<div class="section-heading">{$t('common_metadata')}</div>
		<div class="metadata-table">
			<div class="metadata-row">
				<div class="metadata-label">{$t('common_type')}</div>
				<div class="metadata-value">{itemTypeLabel}</div>
			</div>
			{#if formattedDate}
				<div class="metadata-row">
					<div class="metadata-label">{$t('common_published')}</div>
					<div class="metadata-value">{formattedDate}</div>
				</div>
			{/if}
			{#if totalPages > 0}
				<div class="metadata-row">
					<div class="metadata-label">{$t('reader_length')}</div>
					<div class="metadata-value">
						{$t('reader_page_count', { values: { count: totalPages } })}
					</div>
				</div>
			{/if}
			{#if bookMetadata.publisher}
				<div class="metadata-row">
					<div class="metadata-label">{$t('reader_publisher')}</div>
					<div class="metadata-value">{bookMetadata.publisher}</div>
				</div>
			{/if}
			{#if bookMetadata.isbn}
				<div class="metadata-row">
					<div class="metadata-label">{$t('reader_isbn')}</div>
					<div class="metadata-value">{bookMetadata.isbn}</div>
				</div>
			{/if}
			<div class="metadata-row">
				<div class="metadata-label">{$t('common_saved')}</div>
				<div class="metadata-value">{savedAgo}</div>
			</div>
			{#if bookMetadata.language}
				<div class="metadata-row">
					<div class="metadata-label">{$t('common_language')}</div>
					<div class="metadata-value">{languageLabel}</div>
				</div>
			{/if}
		</div>
	</div>

	<div class="section-divider"></div>

	<div class="detail-section">
		<div class="section-heading">{$t('reader_reading_progress')}</div>
		<div class="metadata-table">
			<div class="metadata-row">
				<div class="metadata-label">{$t('reader_progress')}</div>
				<div class="metadata-value">{progressLabel}</div>
			</div>
			{#if remainingEstimate}
				<div class="metadata-row">
					<div class="metadata-label">{$t('reader_remaining')}</div>
					<div class="metadata-value">{remainingEstimate}</div>
				</div>
			{/if}
			<div class="metadata-row">
				<div class="metadata-label">{$t('reader_last_read')}</div>
				<div class="metadata-value">{lastReadLabel}</div>
			</div>
		</div>
	</div>
</div>

<style>
	.detail-content {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.detail-title {
		font-size: 16px;
		font-weight: 600;
		color: var(--text-primary);
		line-height: 1.3;
		letter-spacing: -0.02em;
		font-family: var(--font-sans);
	}

	.detail-domain {
		font-size: 13px;
		font-weight: 400;
		color: var(--text-secondary);
		margin-top: -8px;
		font-family: var(--font-sans);
	}

	.detail-section {
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
		font-family: var(--font-sans);
	}

	.metadata-table {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.metadata-row {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
	}

	.metadata-label {
		font-size: 12px;
		font-weight: 400;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
	}

	.metadata-value {
		font-size: 12px;
		font-weight: 500;
		color: var(--text-primary);
		font-family: var(--font-sans);
	}

	.section-divider {
		height: 0.5px;
		background: var(--border-primary);
	}
</style>
