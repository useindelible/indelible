<script lang="ts">
	import type { DocumentListEntry } from '$lib/api';
	import * as apiSdk from '$lib/api';
	import { getLibrary } from '$lib/stores/library.svelte';
	import TagInput from './TagInput.svelte';
	import { date, t } from '$lib/i18n';

	interface Props {
		item: DocumentListEntry;
		onClose: () => void;
	}

	let { item, onClose }: Props = $props();

	const lib = getLibrary();

	function toDisplayDate(iso: string | null | undefined): string {
		if (!iso) return '';
		const d = new Date(iso);
		if (isNaN(d.getTime())) return '';
		return $date(d, { month: 'short', day: 'numeric', year: 'numeric' });
	}

	function toDateInputValue(iso: string | null | undefined): string {
		if (!iso) return '';
		const d = new Date(iso);
		if (isNaN(d.getTime())) return '';
		return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
	}

	// Snapshot captures the prop at mount time — intentional for form field initialization.
	const snap = $state.snapshot(item);
	const libraryEntryId = snap.library_entry_id;
	let title = $state(snap.title);
	let author = $state(snap.author ?? '');
	let publishedAt = $state(toDateInputValue(snap.published_at));
	let excerpt = $state(snap.excerpt ?? '');

	let tags = $state<string[]>([]);
	let tagsLoaded = $state(false);

	$effect(() => {
		if (!libraryEntryId) {
			tagsLoaded = true;
			return;
		}

		apiSdk.getLibraryEntryTags({ path: { library_entry_id: libraryEntryId } }).then(({ data }) => {
			tags = data?.tags ?? [];
			tagsLoaded = true;
		});
	});

	let saving = $state(false);
	let error = $state('');
	let titleEl = $state<HTMLTextAreaElement | undefined>(undefined);

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

	const lengthLabel = $derived(() => {
		const mins = item.reading_time_minutes;
		const words = item.word_count;
		if (mins && words) {
			return $t('library_length_minutes_words', { values: { minutes: mins, words } });
		}
		if (mins) return $t('library_length_minutes', { values: { minutes: mins } });
		if (words) return $t('library_word_count', { values: { count: words } });
		return '—';
	});

	const progressLabel = $derived(
		item.progress_percent != null ? `${Math.round(item.progress_percent)}%` : '0%'
	);

	const savedLabel = $derived(toDisplayDate(item.saved_at));

	function autoResize(el: HTMLTextAreaElement) {
		el.style.height = 'auto';
		el.style.height = el.scrollHeight + 'px';
	}

	$effect(() => {
		if (titleEl) autoResize(titleEl);
	});

	async function handleSave() {
		error = '';
		saving = true;
		try {
			const saveTags = libraryEntryId
				? apiSdk.replaceLibraryEntryTags({
						path: { library_entry_id: libraryEntryId },
						body: { tags }
					})
				: Promise.resolve();

			const [itemResult] = await Promise.all([
				apiSdk.updateDocumentEntry({
					path: { document_id: item.id },
					body: {
						title: title || undefined,
						author: author || undefined,
						published_at: publishedAt ? new Date(publishedAt + 'T00:00:00').toISOString() : null,
						excerpt: excerpt || undefined
					}
				}),
				saveTags
			]);

			if (itemResult.data) {
				lib.updateItemInList(itemResult.data as DocumentListEntry);
				onClose();
				return;
			}

			error = $t('library_error_save_changes');
		} catch {
			error = $t('library_error_unexpected');
		} finally {
			saving = false;
		}
	}
</script>

<div class="edit-panel">
	<!-- Cover hero -->
	<div class="cover-wrap">
		<div class="cover-img">
			<span class="cover-emoji" aria-hidden="true">📄</span>
			<button
				type="button"
				class="cover-change-btn"
				aria-label={$t('library_edit_change_cover_image')}
			>
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path
						d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"
					/>
					<circle cx="12" cy="13" r="4" />
				</svg>
				{$t('library_edit_change_cover')}
			</button>
		</div>
	</div>

	<form
		class="edit-form"
		onsubmit={(e) => {
			e.preventDefault();
			handleSave();
		}}
	>
		<!-- Title -->
		<textarea
			class="title-input"
			bind:this={titleEl}
			bind:value={title}
			oninput={(e) => autoResize(e.currentTarget)}
			rows={2}
			aria-label={$t('common_title')}
		></textarea>

		<!-- Byline row -->
		<div class="byline-row">
			<span class="byline-by">{$t('library_edit_by')}</span>
			<input
				class="byline-input"
				type="text"
				bind:value={author}
				placeholder={$t('library_edit_author')}
				style="width: 110px;"
				aria-label={$t('library_edit_author')}
			/>
			<span class="byline-sep">&middot;</span>
			<select
				class="byline-input byline-select"
				aria-label={$t('library_filter_field_content_type')}
				style="cursor: pointer; width: 80px;"
			>
				<option value="article" selected={item.item_type === 'article'}
					>{$t('library_filter_value_article')}</option
				>
				<option value="book" selected={item.item_type === 'book'}
					>{$t('library_filter_value_book')}</option
				>
				<option value="pdf" selected={item.item_type === 'pdf'}
					>{$t('library_filter_value_pdf')}</option
				>
				<option value="video" selected={item.item_type === 'video'}
					>{$t('library_filter_value_video')}</option
				>
				{#if item.item_type === 'podcast'}
					<option value="podcast" selected>{$t('library_filter_value_podcast')}</option>
				{/if}
				<option value="tweet" selected={item.item_type === 'tweet'}
					>{$t('library_filter_value_tweet')}</option
				>
				<option value="email" selected={item.item_type === 'email'}
					>{$t('library_filter_value_email')}</option
				>
			</select>
		</div>

		{#if displayDomain}
			<div class="domain-display">{displayDomain}</div>
		{/if}

		<!-- Summary -->
		<div class="edit-section">
			<div class="section-heading">{$t('library_edit_summary')}</div>
			<textarea
				class="summary-input"
				bind:value={excerpt}
				rows={4}
				placeholder={$t('library_edit_add_summary')}
				aria-label={$t('library_edit_summary')}
			></textarea>
		</div>

		<!-- Tags -->
		<div class="edit-section">
			<div class="section-heading">{$t('common_tags')}</div>
			{#if tagsLoaded}
				<TagInput bind:tags />
			{/if}
		</div>

		<!-- Metadata -->
		<div class="edit-section">
			<div class="section-heading">{$t('common_metadata')}</div>
			<div class="metadata-table">
				<div class="metadata-row">
					<span class="metadata-label">{$t('common_published')}</span>
					<div class="metadata-value">
						<input
							class="meta-input"
							type="date"
							bind:value={publishedAt}
							aria-label={$t('library_filter_field_published_date')}
						/>
					</div>
				</div>
				<div class="metadata-row">
					<span class="metadata-label">{$t('library_metadata_length')}</span>
					<span class="metadata-static">{lengthLabel()}</span>
				</div>
				<div class="metadata-row">
					<span class="metadata-label">{$t('library_metadata_progress')}</span>
					<span class="metadata-static">{progressLabel}</span>
				</div>
				<div class="metadata-row">
					<span class="metadata-label">{$t('library_metadata_saved')}</span>
					<span class="metadata-static">{savedLabel}</span>
				</div>
			</div>
		</div>

		{#if error}
			<p class="save-error" role="alert">{error}</p>
		{/if}

		<div class="form-actions">
			<button type="button" class="btn-secondary" onclick={onClose} disabled={saving}
				>{$t('common_cancel')}</button
			>
			<button type="submit" class="btn-primary" disabled={saving} aria-busy={saving}>
				{#if saving}
					<span class="spinner" aria-hidden="true"></span>
					{$t('common_saving')}
				{:else}
					{$t('common_save')}
				{/if}
			</button>
		</div>
	</form>
</div>

<style>
	.edit-panel {
		display: flex;
		flex-direction: column;
		flex: 1;
		overflow-y: auto;
	}

	/* Cover hero */
	.cover-wrap {
		margin: 0;
		position: relative;
		overflow: hidden;
	}

	.cover-img {
		width: 100%;
		height: 120px;
		background: linear-gradient(135deg, rgba(0, 113, 227, 0.22), rgba(0, 113, 227, 0.46));
		display: flex;
		align-items: center;
		justify-content: center;
		position: relative;
		user-select: none;
	}

	.cover-emoji {
		position: absolute;
		font-size: 38px;
		opacity: 0.5;
	}

	.cover-change-btn {
		position: relative;
		z-index: 1;
		display: inline-flex;
		align-items: center;
		gap: 5px;
		background: rgba(0, 0, 0, 0.42);
		backdrop-filter: blur(12px);
		-webkit-backdrop-filter: blur(12px);
		color: rgba(255, 255, 255, 0.92);
		font-size: 12px;
		font-weight: 500;
		letter-spacing: -0.01em;
		padding: 5px 11px;
		border-radius: 20px;
		cursor: pointer;
		border: 0.5px solid rgba(255, 255, 255, 0.22);
		font-family: var(--font-sans);
		outline: none;
	}

	.cover-change-btn svg {
		width: 11px;
		height: 11px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
		flex-shrink: 0;
	}

	/* Form */
	.edit-form {
		padding: 16px 20px 20px;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.title-input {
		font-size: 20px;
		font-weight: 700;
		letter-spacing: -0.025em;
		line-height: 1.25;
		color: var(--text-primary);
		background: transparent;
		border: none;
		border-bottom: 1.5px solid transparent;
		outline: none;
		width: 100%;
		resize: none;
		font-family: var(--font-sans);
		padding: 0 0 2px;
		box-sizing: border-box;
		overflow: hidden;
	}

	.title-input:focus {
		border-bottom-color: var(--accent);
	}

	.byline-row {
		display: flex;
		align-items: center;
		gap: 5px;
		font-size: 12.5px;
		color: var(--text-secondary);
		margin-top: -8px;
		flex-wrap: wrap;
	}

	.byline-by {
		color: var(--text-tertiary);
		font-size: 11.5px;
		font-family: var(--font-sans);
	}

	.byline-sep {
		color: var(--text-quaternary, var(--text-tertiary));
		font-family: var(--font-sans);
	}

	.byline-input {
		background: transparent;
		border: none;
		border-bottom: 1px solid transparent;
		outline: none;
		font-family: var(--font-sans);
		font-size: 12.5px;
		color: var(--text-secondary);
		padding: 0 0 1px;
		min-width: 40px;
	}

	.byline-input:hover {
		border-bottom-color: var(--border-secondary);
	}

	.byline-input:focus {
		border-bottom-color: var(--accent);
		color: var(--text-primary);
	}

	.byline-select {
		appearance: none;
		-webkit-appearance: none;
		cursor: pointer;
	}

	.domain-display {
		font-size: 12px;
		font-weight: 400;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
		margin-top: -6px;
	}

	.edit-section {
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
		line-height: 1.2;
	}

	.summary-input {
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.65;
		color: var(--text-primary);
		background: var(--fill-hover);
		border: 1px solid transparent;
		border-radius: 8px;
		padding: 8px 10px;
		outline: none;
		width: 100%;
		resize: none;
		font-family: var(--font-sans);
		box-sizing: border-box;
	}

	.summary-input:focus {
		border-color: var(--accent);
		box-shadow: 0 0 0 3px rgba(0, 113, 227, 0.12);
	}

	.metadata-table {
		display: flex;
		flex-direction: column;
		gap: 0;
	}

	.metadata-row {
		display: flex;
		align-items: center;
		padding: 8px 0;
		border-bottom: 0.5px solid var(--border-primary);
	}

	.metadata-row:last-child {
		border-bottom: none;
	}

	.metadata-label {
		flex: 1;
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
	}

	.metadata-value {
		flex: 1;
		display: flex;
		justify-content: flex-end;
	}

	.meta-input {
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		background: transparent;
		border: none;
		border-bottom: 1px solid transparent;
		outline: none;
		text-align: right;
		width: 100%;
		padding: 0;
		font-family: var(--font-sans);
	}

	.meta-input:hover {
		border-bottom-color: var(--border-secondary);
	}

	.meta-input:focus {
		border-bottom-color: var(--accent);
	}

	.metadata-static {
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--text-secondary);
		font-family: var(--font-sans);
		text-align: right;
		flex: 1;
	}

	.save-error {
		font-size: 12px;
		color: var(--destructive);
		margin: 0;
		font-family: var(--font-sans);
	}

	.form-actions {
		display: flex;
		gap: 8px;
		justify-content: flex-end;
		padding-top: 12px;
		border-top: 0.5px solid var(--border-primary);
	}

	.btn-primary {
		display: flex;
		align-items: center;
		gap: 6px;
		height: 30px;
		padding: 0 14px;
		border-radius: 7px;
		font-size: 13px;
		font-weight: 500;
		letter-spacing: -0.01em;
		font-family: var(--font-sans);
		cursor: pointer;
		border: none;
		background: var(--accent);
		color: var(--text-on-color);
		transition: opacity 120ms ease;
	}

	.btn-primary:hover:not(:disabled) {
		opacity: 0.9;
	}

	.btn-primary:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.btn-secondary {
		height: 30px;
		padding: 0 14px;
		border-radius: 7px;
		font-size: 13px;
		font-weight: 500;
		letter-spacing: -0.01em;
		font-family: var(--font-sans);
		cursor: pointer;
		background: transparent;
		border: 1px solid var(--border-secondary);
		color: var(--text-primary);
		transition: background 120ms ease;
	}

	.btn-secondary:hover:not(:disabled) {
		background: var(--fill-hover);
	}

	.btn-secondary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.spinner {
		display: inline-block;
		width: 12px;
		height: 12px;
		border: 2px solid rgba(255, 255, 255, 0.35);
		border-top-color: var(--text-on-color);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
		flex-shrink: 0;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
