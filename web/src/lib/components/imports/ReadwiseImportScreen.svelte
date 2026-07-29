<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import ProviderIcon from './ProviderIcon.svelte';
	import ImportProgressCard from './ImportProgressCard.svelte';
	import ImportReport from './ImportReport.svelte';
	import {
		countCsvRows,
		countOpmlFeeds,
		validateReadwiseCsv,
		type ReadwiseImportFiles
	} from '$lib/api/imports';
	import { findProvider } from '$lib/integrations/providers';
	import type { ImportJobStatusResponse } from '$lib/api';

	interface Props {
		activeJob: ImportJobStatusResponse | null;
		isTerminal: boolean;
		busySlug: string | null;
		uploadError: string | null;
		pollError: string | null;
		rollbackNotice: string | null;
		onBack: () => void;
		onUpload: (files: ReadwiseImportFiles) => void;
		onRollback: () => void;
		onDismiss: () => void;
	}

	let {
		activeJob,
		isTerminal,
		busySlug,
		uploadError,
		pollError,
		rollbackNotice,
		onBack,
		onUpload,
		onRollback,
		onDismiss
	}: Props = $props();

	const provider = findProvider('readwise')!;

	// Staged files
	let libraryCsv = $state<File | null>(null);
	let archiveZip = $state<File | null>(null);
	let feedsOpml = $state<File | null>(null);

	// Client-side stats (computed asynchronously)
	let csvRowCount = $state<number | null>(null);
	let opmlFeedCount = $state<number | null>(null);
	let csvError = $state<string | null>(null);

	// Dropzone state
	let dragActive = $state(false);
	let dropError = $state<string | null>(null);
	let fileInputEl = $state<HTMLInputElement | undefined>(undefined);

	const isCommitting = $derived(busySlug === 'readwise');
	const hasFiles = $derived(libraryCsv !== null || archiveZip !== null || feedsOpml !== null);
	const zipWithoutCsv = $derived(archiveZip !== null && libraryCsv === null);

	const totalSize = $derived(
		(libraryCsv?.size ?? 0) + (archiveZip?.size ?? 0) + (feedsOpml?.size ?? 0)
	);

	const maxPerFile = provider.maxBytes ?? 50 * 1024 * 1024;

	function formatSize(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1_048_576) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / 1_048_576).toFixed(1)} MB`;
	}

	async function readText(file: File): Promise<string> {
		return new Promise((resolve, reject) => {
			const reader = new FileReader();
			reader.onload = () => resolve(reader.result as string);
			reader.onerror = () => reject(reader.error);
			reader.readAsText(file);
		});
	}

	async function stageCsv(file: File) {
		csvError = null;
		csvRowCount = null;
		try {
			const text = await readText(file);
			const err = validateReadwiseCsv(text);
			if (err) {
				csvError = err;
				return;
			}
			libraryCsv = file;
			csvRowCount = countCsvRows(text);
		} catch {
			csvError = 'Could not read the CSV file.';
		}
	}

	async function stageOpml(file: File) {
		feedsOpml = file;
		opmlFeedCount = null;
		try {
			const text = await readText(file);
			opmlFeedCount = countOpmlFeeds(text);
		} catch {
			opmlFeedCount = null;
		}
	}

	async function routeFiles(files: FileList | File[]) {
		dropError = null;
		const arr = Array.from(files);
		for (const file of arr) {
			if (file.size > maxPerFile) {
				dropError = `${file.name} exceeds the ${Math.round(maxPerFile / 1_048_576)} MB per-file limit.`;
				continue;
			}
			const lower = file.name.toLowerCase();
			if (lower.endsWith('.csv')) {
				await stageCsv(file);
			} else if (lower.endsWith('.zip')) {
				archiveZip = file;
			} else if (lower.endsWith('.opml') || lower.endsWith('.xml')) {
				await stageOpml(file);
			} else {
				dropError = `${file.name}: unsupported type. Accepted: .csv, .zip, .opml, .xml`;
			}
		}
	}

	function onDragOver(event: DragEvent) {
		event.preventDefault();
		dragActive = true;
	}

	function onDragLeave() {
		dragActive = false;
	}

	async function onDrop(event: DragEvent) {
		event.preventDefault();
		dragActive = false;
		const files = event.dataTransfer?.files;
		if (files && files.length > 0) {
			await routeFiles(files);
		}
	}

	async function onFileChange(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (input.files && input.files.length > 0) {
			await routeFiles(input.files);
		}
		input.value = '';
	}

	function removeFile(slot: 'csv' | 'zip' | 'opml') {
		if (slot === 'csv') {
			libraryCsv = null;
			csvRowCount = null;
			csvError = null;
		}
		if (slot === 'zip') {
			archiveZip = null;
		}
		if (slot === 'opml') {
			feedsOpml = null;
			opmlFeedCount = null;
		}
	}

	function commitImport() {
		if (!hasFiles || isCommitting) return;
		onUpload({ libraryCsv, archiveZip, feedsOpml });
	}

	// Readwise report panel data
	const readwiseReport = $derived(activeJob?.readwise_report ?? null);

	const unmatchedToShow = $derived(readwiseReport?.unmatched_zip_assets?.slice(0, 6) ?? []);
	const unmatchedExtra = $derived(
		Math.max(0, (readwiseReport?.unmatched_zip_assets?.length ?? 0) - 6)
	);
</script>

<div class="screen">
	<header class="screen-header">
		<Button variant="tertiary" size="sm" onclick={onBack}>← Back</Button>
		<div class="provider-identity">
			<ProviderIcon provider="readwise" size={32} />
			<div class="provider-text">
				<h2 class="provider-name">Import from Readwise Reader</h2>
				<p class="provider-desc">
					Drop any combination of CSV, ZIP archive, and OPML — we'll route each automatically.
				</p>
			</div>
		</div>
	</header>

	{#if !activeJob}
		<!-- Staging phase -->
		<section class="staging">
			<!-- Combined dropzone -->
			<div
				class="dropzone"
				class:drag-active={dragActive}
				class:has-error={dropError !== null}
				ondragover={onDragOver}
				ondragleave={onDragLeave}
				ondrop={onDrop}
				role="button"
				tabindex="0"
				onclick={() => fileInputEl?.click()}
				onkeydown={(e) => {
					if (e.key === 'Enter' || e.key === ' ') {
						e.preventDefault();
						fileInputEl?.click();
					}
				}}
			>
				<input
					type="file"
					multiple
					accept=".csv,.zip,.opml,.xml"
					class="hidden-input"
					bind:this={fileInputEl}
					onchange={onFileChange}
				/>
				<p class="dropzone-prompt">Drop files here or click to browse</p>
				<p class="dropzone-hint">
					Accepts .csv, .zip, .opml — up to {Math.round(maxPerFile / 1_048_576)} MB per file
				</p>
			</div>

			{#if dropError}
				<p class="drop-error" role="alert">{dropError}</p>
			{/if}

			{#if csvError}
				<p class="drop-error" role="alert">{csvError}</p>
			{/if}

			{#if uploadError}
				<p class="drop-error" role="alert">{uploadError}</p>
			{/if}

			<!-- Staged files list -->
			{#if hasFiles}
				<ul class="staged-list">
					{#if libraryCsv}
						<li class="staged-item">
							<span class="file-icon file-icon-csv">CSV</span>
							<div class="file-info">
								<span class="file-name">{libraryCsv.name}</span>
								<span class="file-meta">
									{formatSize(libraryCsv.size)}{csvRowCount !== null
										? ` · ${csvRowCount} rows`
										: ''} · CSV
								</span>
							</div>
							<Button variant="tertiary" size="sm" onclick={() => removeFile('csv')}>Remove</Button>
						</li>
					{/if}
					{#if archiveZip}
						<li class="staged-item">
							<span class="file-icon file-icon-zip">ZIP</span>
							<div class="file-info">
								<span class="file-name">{archiveZip.name}</span>
								<span class="file-meta">{formatSize(archiveZip.size)} · ZIP archive</span>
							</div>
							<Button variant="tertiary" size="sm" onclick={() => removeFile('zip')}>Remove</Button>
						</li>
					{/if}
					{#if feedsOpml}
						<li class="staged-item">
							<span class="file-icon file-icon-opml">OPML</span>
							<div class="file-info">
								<span class="file-name">{feedsOpml.name}</span>
								<span class="file-meta">
									{formatSize(feedsOpml.size)}{opmlFeedCount !== null
										? ` · ${opmlFeedCount} feeds`
										: ''} · OPML
								</span>
							</div>
							<Button variant="tertiary" size="sm" onclick={() => removeFile('opml')}>Remove</Button
							>
						</li>
					{/if}
				</ul>

				<!-- Pre-commit preview -->
				<section class="preview">
					<h3 class="preview-title">Pre-commit preview</h3>
					<p class="preview-note">
						{csvRowCount !== null ? `${csvRowCount} CSV rows staged` : 'No CSV staged'}
						{opmlFeedCount !== null ? ` · ${opmlFeedCount} OPML feeds detected` : ''}. Matched /
						unmatched archive counts will appear once the import runs.
					</p>
					<dl class="stats-grid">
						<div class="stat-card">
							<dt>CSV rows</dt>
							<dd>{csvRowCount !== null ? csvRowCount : '—'}</dd>
						</div>
						<div class="stat-card">
							<dt>Archive ZIP</dt>
							<dd>{archiveZip ? formatSize(archiveZip.size) : '—'}</dd>
						</div>
						<div class="stat-card">
							<dt>OPML feeds</dt>
							<dd>{opmlFeedCount !== null ? opmlFeedCount : '—'}</dd>
						</div>
						<div class="stat-card">
							<dt>Total size</dt>
							<dd>{totalSize > 0 ? formatSize(totalSize) : '—'}</dd>
						</div>
					</dl>
				</section>
			{/if}

			{#if zipWithoutCsv}
				<div class="zip-warning" role="note">
					<strong>No CSV staged</strong> — archive documents will import without metadata (URL, tags,
					reading progress). Add your Readwise library CSV to preserve this.
				</div>
			{/if}

			<div class="staging-actions">
				<Button variant="secondary" size="sm" onclick={onBack}>Cancel</Button>
				<Button
					variant="primary"
					size="sm"
					disabled={!hasFiles}
					loading={isCommitting}
					onclick={commitImport}
				>
					Commit import
				</Button>
			</div>
		</section>
	{:else if !isTerminal}
		<!-- Progress phase -->
		<section class="progress-section">
			<ImportProgressCard job={activeJob} />

			{#if pollError}
				<p class="poll-error" role="alert">{pollError}</p>
			{/if}

			{#if readwiseReport && (readwiseReport.opml_feeds_created ?? 0) > 0}
				<div class="opml-callout">
					<p class="opml-text">
						OPML routed to Feeds. We added {readwiseReport.opml_feeds_created} RSS subscription{readwiseReport.opml_feeds_created ===
						1
							? ''
							: 's'} to your Feeds manager.
					</p>
				</div>
			{/if}

			{#if readwiseReport && ((readwiseReport.zip_files_matched ?? 0) > 0 || (readwiseReport.zip_files_unmatched ?? 0) > 0)}
				<div class="match-grid">
					<div class="match-panel match-panel-matched">
						<header class="match-header">
							<span class="match-icon match-icon-success" aria-hidden="true">✓</span>
							<h4 class="match-title">Matched</h4>
						</header>
						<p class="match-count">{readwiseReport.zip_files_matched ?? 0} archive files</p>
						<p class="match-sub">matched to CSV rows · highlights anchored to existing items</p>
					</div>
					<div class="match-panel match-panel-unmatched">
						<header class="match-header">
							<span class="match-icon match-icon-warning" aria-hidden="true">!</span>
							<h4 class="match-title">Unmatched</h4>
						</header>
						{#if unmatchedToShow.length > 0}
							<ul class="unmatched-list">
								{#each unmatchedToShow as asset (asset)}
									<li class="unmatched-item">
										<span class="unmatched-name">{asset}</span>
										<span class="unmatched-meta">URL-only · re-archive pending</span>
									</li>
								{/each}
							</ul>
							{#if unmatchedExtra > 0}
								<p class="unmatched-more">+ {unmatchedExtra} more</p>
							{/if}
						{:else}
							<p class="match-count">{readwiseReport.zip_files_unmatched ?? 0} unmatched</p>
						{/if}
					</div>
				</div>
			{/if}

			<button type="button" class="dismiss-btn" onclick={onDismiss}>Stop tracking</button>
		</section>
	{:else}
		<!-- Report phase -->
		<section class="report-section">
			<ImportReport job={activeJob} canRollback={true} {onRollback} />
			{#if rollbackNotice}
				<p class="rollback-notice" role="status">{rollbackNotice}</p>
			{/if}
			<Button variant="secondary" size="sm" onclick={onBack}>Back to imports</Button>
		</section>
	{/if}
</div>

<style>
	.screen {
		display: flex;
		flex-direction: column;
		gap: 24px;
	}

	.screen-header {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.provider-identity {
		display: flex;
		align-items: flex-start;
		gap: 12px;
	}

	.provider-text {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.provider-name {
		font-family: var(--font-sans);
		font-size: 22px;
		font-weight: 700;
		letter-spacing: -0.02em;
		color: var(--text-primary);
		margin: 0;
	}

	.provider-desc {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-secondary);
		margin: 0;
		line-height: 1.4;
	}

	/* Staging */
	.staging {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.dropzone {
		border: 1.5px dashed var(--border-secondary);
		border-radius: var(--radius-lg);
		padding: 32px 24px;
		text-align: center;
		cursor: pointer;
		background: transparent;
		transition:
			border-color 120ms ease,
			background 120ms ease;
	}

	.dropzone:hover,
	.dropzone:focus-visible,
	.dropzone.drag-active {
		border-color: var(--accent);
		background: var(--fill-selected);
		outline: none;
	}

	.dropzone.has-error {
		border-color: var(--destructive);
		background: var(--fill-danger);
	}

	.hidden-input {
		display: none;
	}

	.dropzone-prompt {
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--text-primary);
		margin: 0 0 6px;
	}

	.dropzone-hint {
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--text-tertiary);
		margin: 0;
	}

	.drop-error {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--destructive);
		margin: 0;
	}

	/* Staged list */
	.staged-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.staged-item {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 14px;
		border-radius: var(--radius-md);
		background: var(--bg-secondary);
		border: 0.5px solid var(--border-primary);
	}

	.file-icon {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		border-radius: var(--radius-sm);
		font-family: var(--font-sans);
		font-size: 9px;
		font-weight: 700;
		letter-spacing: 0.04em;
		flex-shrink: 0;
	}

	.file-icon-csv {
		background: var(--fill-success);
		color: var(--success);
	}

	.file-icon-zip {
		background: var(--fill-selected);
		color: var(--accent);
	}

	.file-icon-opml {
		background: var(--fill-warning);
		color: var(--warning);
	}

	.file-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		flex: 1;
		min-width: 0;
	}

	.file-name {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.file-meta {
		font-family: var(--font-sans);
		font-size: 11px;
		color: var(--text-tertiary);
	}

	/* Preview panel */
	.preview {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 14px 16px;
		border-radius: var(--radius-lg);
		background: var(--bg-secondary);
		border: 0.5px solid var(--border-primary);
	}

	.preview-title {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0;
	}

	.preview-note {
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--text-secondary);
		margin: 0;
		line-height: 1.4;
	}

	.stats-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 8px;
		margin: 0;
	}

	@media (max-width: 599px) {
		.stats-grid {
			grid-template-columns: repeat(2, 1fr);
		}
	}

	.stat-card {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 8px 10px;
		border-radius: var(--radius-sm);
		background: var(--bg-elevated);
		border: 0.5px solid var(--border-primary);
	}

	.stat-card dt {
		font-family: var(--font-sans);
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-tertiary);
	}

	.stat-card dd {
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0;
	}

	.zip-warning {
		padding: 10px 14px;
		border-radius: var(--radius-md);
		background: var(--fill-warning);
		border: 0.5px solid var(--border-primary);
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-primary);
		line-height: 1.4;
	}

	.zip-warning strong {
		font-weight: 600;
	}

	.staging-actions {
		display: flex;
		gap: 8px;
		justify-content: flex-end;
	}

	/* Progress */
	.progress-section {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.poll-error {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--destructive);
		margin: 0;
	}

	.opml-callout {
		padding: 12px 14px;
		border-radius: var(--radius-md);
		background: var(--fill-selected);
		border: 0.5px solid var(--border-primary);
	}

	.opml-text {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-primary);
		margin: 0;
		line-height: 1.4;
	}

	.match-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
	}

	@media (max-width: 599px) {
		.match-grid {
			grid-template-columns: 1fr;
		}
	}

	.match-panel {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 14px;
		border-radius: var(--radius-lg);
		border: 0.5px solid var(--border-primary);
	}

	.match-panel-matched {
		background: var(--fill-success);
	}

	.match-panel-unmatched {
		background: var(--fill-warning);
	}

	.match-header {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.match-icon {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		border-radius: var(--radius-circle);
		font-size: 11px;
		font-weight: 700;
		flex-shrink: 0;
	}

	.match-icon-success {
		background: var(--success);
		color: var(--text-on-color);
	}

	.match-icon-warning {
		background: var(--warning);
		color: var(--text-on-color);
	}

	.match-title {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0;
	}

	.match-count {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0;
	}

	.match-sub {
		font-family: var(--font-sans);
		font-size: 11px;
		color: var(--text-secondary);
		margin: 0;
		line-height: 1.4;
	}

	.unmatched-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.unmatched-item {
		display: flex;
		flex-direction: column;
		gap: 1px;
	}

	.unmatched-name {
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 500;
		color: var(--text-primary);
		word-break: break-all;
	}

	.unmatched-meta {
		font-family: var(--font-sans);
		font-size: 11px;
		color: var(--text-secondary);
	}

	.unmatched-more {
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--text-secondary);
		margin: 0;
	}

	.dismiss-btn {
		align-self: flex-end;
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--text-tertiary);
		background: transparent;
		border: none;
		cursor: pointer;
		padding: 4px 0;
	}

	.dismiss-btn:hover {
		color: var(--text-primary);
	}

	/* Report */
	.report-section {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.rollback-notice {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--success);
		margin: 0;
	}
</style>
