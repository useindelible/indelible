<script lang="ts">
	import type { DocumentListEntry } from '$lib/api';
	import { t } from '$lib/i18n';
	import AuthorCard from './AuthorCard.svelte';
	import SummarySection from './SummarySection.svelte';
	import MetadataTable from './MetadataTable.svelte';
	import EntitiesSection from './EntitiesSection.svelte';

	interface Props {
		item: DocumentListEntry;
		onEditMetadata: () => void;
	}

	let { item, onEditMetadata }: Props = $props();

	// Parse "Name @handle" pattern from free-text author field.
	const parsedAuthor = $derived(
		(() => {
			const raw = item.author;
			if (!raw) return null;
			const match = raw.match(/^(.*?)\s*(@\S+)\s*$/);
			if (match && match[1] !== undefined && match[2] !== undefined) {
				return { name: match[1].trim() || match[2], handle: match[2] };
			}
			return { name: raw, handle: null };
		})()
	);

	function domainFromUrl(url: string | null | undefined): string | null {
		if (!url) return null;
		try {
			return new URL(url).hostname;
		} catch {
			return null;
		}
	}

	const displayDomain = $derived(
		item.domain ?? domainFromUrl(item.url) ?? domainFromUrl(item.canonical_url)
	);
</script>

<div class="detail-content">
	<div class="detail-header">
		<h2 class="detail-title">{item.title}</h2>
		{#if displayDomain}
			<p class="detail-domain">{displayDomain}</p>
		{/if}
	</div>

	{#if parsedAuthor}
		<AuthorCard name={parsedAuthor.name} handle={parsedAuthor.handle} currentItemId={item.id} />
	{/if}

	<SummarySection summary={item.summary} excerpt={item.excerpt} />

	<MetadataTable {item} />

	<button class="edit-metadata-link" type="button" onclick={onEditMetadata}
		>{$t('library_edit_metadata')}</button
	>

	<EntitiesSection itemId={item.id} />
</div>

<style>
	.detail-content {
		padding: 20px;
		display: flex;
		flex-direction: column;
		gap: 20px;
		flex: 1;
		overflow-y: auto;
	}

	.detail-header {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.detail-title {
		font-size: 20px;
		font-weight: 700;
		letter-spacing: -0.025em;
		line-height: 1.25;
		color: var(--text-primary);
		margin: 0;
	}

	.detail-domain {
		font-size: 12.5px;
		font-weight: 400;
		color: var(--text-secondary);
		margin: 0;
	}

	.edit-metadata-link {
		font-size: 13px;
		font-weight: 400;
		color: var(--accent);
		background: none;
		border: none;
		cursor: pointer;
		text-align: center;
		padding: 8px 0;
		font-family: var(--font-sans);
		width: 100%;
	}

	.edit-metadata-link:hover {
		text-decoration: underline;
	}
</style>
