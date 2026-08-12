<script lang="ts">
	import type { DocumentListEntry } from '$lib/api';
	import SummarySection from '$lib/components/library/SummarySection.svelte';
	import type { BookMetadata } from './book-source';

	interface Props {
		item: DocumentListEntry;
		bookMetadata: BookMetadata;
		progress: number;
	}

	let { item, bookMetadata, progress }: Props = $props();

	const formattedDate = $derived(
		item.published_at
			? new Date(item.published_at).toLocaleDateString('en-US', {
					year: 'numeric',
					month: 'short',
					day: 'numeric'
				})
			: null
	);

	const savedAgo = $derived(
		(() => {
			const diff = Date.now() - new Date(item.saved_at).getTime();
			const days = Math.floor(diff / 86400000);
			if (days === 0) return 'Today';
			if (days === 1) return 'Yesterday';
			if (days < 7) return `${days} days ago`;
			const weeks = Math.floor(days / 7);
			if (weeks < 5) return `${weeks} week${weeks > 1 ? 's' : ''} ago`;
			const months = Math.floor(days / 30);
			return `${months} month${months > 1 ? 's' : ''} ago`;
		})()
	);

	const lastReadLabel = $derived(
		(() => {
			if (!item.last_read_at) return 'Never';
			const diff = Date.now() - new Date(item.last_read_at).getTime();
			const minutes = Math.floor(diff / 60000);
			if (minutes < 2) return 'Just now';
			if (minutes < 60) return `${minutes}m ago`;
			const hours = Math.floor(minutes / 60);
			if (hours < 24) return `${hours}h ago`;
			const days = Math.floor(hours / 24);
			if (days === 1) return 'Yesterday';
			return `${days} days ago`;
		})()
	);

	const totalPages = $derived(bookMetadata.estimatedPages ?? 0);
	const pagesRead = $derived(Math.round((progress / 100) * totalPages));
	const progressLabel = $derived(
		totalPages > 0
			? `${Math.round(progress)}% (${pagesRead} of ${totalPages} pages)`
			: `${Math.round(progress)}%`
	);

	const readingTimeEstimate = $derived(
		bookMetadata.totalWords ? `~${Math.round(bookMetadata.totalWords / 250 / 60)}h estimated` : null
	);

	const remainingEstimate = $derived(
		bookMetadata.totalWords && progress > 0
			? `~${Math.round(((100 - progress) / 100) * (bookMetadata.totalWords / 250 / 60))}h estimated`
			: readingTimeEstimate
	);

	const itemTypeLabel = $derived(
		item.item_type === 'book' ? 'Book (EPUB)' : item.item_type === 'pdf' ? 'PDF' : item.item_type
	);
</script>

<div class="detail-content">
	<div class="detail-title">{item.title}</div>
	<div class="detail-domain">{item.author ?? bookMetadata.author ?? ''}</div>

	<SummarySection summary={item.summary} excerpt={item.excerpt} />

	<div class="detail-section">
		<div class="section-heading">Metadata</div>
		<div class="metadata-table">
			<div class="metadata-row">
				<div class="metadata-label">Type</div>
				<div class="metadata-value">{itemTypeLabel}</div>
			</div>
			{#if formattedDate}
				<div class="metadata-row">
					<div class="metadata-label">Published</div>
					<div class="metadata-value">{formattedDate}</div>
				</div>
			{/if}
			{#if totalPages > 0}
				<div class="metadata-row">
					<div class="metadata-label">Length</div>
					<div class="metadata-value">{totalPages} pages</div>
				</div>
			{/if}
			{#if bookMetadata.publisher}
				<div class="metadata-row">
					<div class="metadata-label">Publisher</div>
					<div class="metadata-value">{bookMetadata.publisher}</div>
				</div>
			{/if}
			{#if bookMetadata.isbn}
				<div class="metadata-row">
					<div class="metadata-label">ISBN</div>
					<div class="metadata-value">{bookMetadata.isbn}</div>
				</div>
			{/if}
			<div class="metadata-row">
				<div class="metadata-label">Saved</div>
				<div class="metadata-value">{savedAgo}</div>
			</div>
			{#if bookMetadata.language}
				<div class="metadata-row">
					<div class="metadata-label">Language</div>
					<div class="metadata-value">{bookMetadata.language}</div>
				</div>
			{/if}
		</div>
	</div>

	<div class="section-divider"></div>

	<div class="detail-section">
		<div class="section-heading">Reading Progress</div>
		<div class="metadata-table">
			<div class="metadata-row">
				<div class="metadata-label">Progress</div>
				<div class="metadata-value">{progressLabel}</div>
			</div>
			{#if remainingEstimate}
				<div class="metadata-row">
					<div class="metadata-label">Remaining</div>
					<div class="metadata-value">{remainingEstimate}</div>
				</div>
			{/if}
			<div class="metadata-row">
				<div class="metadata-label">Last read</div>
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
