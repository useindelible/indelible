<script lang="ts">
	import * as apiSdk from '$lib/api';
	import type {
		DocumentListEntry,
		DocumentNoteResponse,
		HighlightWithNoteResponse
	} from '$lib/api';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { browser } from '$app/environment';

	interface Props {
		item: DocumentListEntry;
	}

	let { item }: Props = $props();

	let highlights = $state<HighlightWithNoteResponse[]>([]);
	let highlightsLoading = $state(false);
	let highlightsError = $state('');

	let note = $state<DocumentNoteResponse | null>(null);
	let noteLoading = $state(false);
	let editing = $state(false);
	let editText = $state('');
	let saving = $state(false);
	let noteError = $state('');
	let noteInputEl = $state<HTMLTextAreaElement | undefined>(undefined);

	let editingHighlightNoteId = $state<string | null>(null);
	let highlightNoteText = $state('');
	let highlightNoteSaving = $state(false);

	const HIGHLIGHT_COLORS: Record<string, string> = {
		yellow: 'var(--highlight-yellow-border)',
		blue: 'var(--highlight-blue-border)',
		green: 'var(--highlight-green-border)',
		pink: 'var(--highlight-pink-border)',
		purple: 'var(--highlight-purple-border)'
	};

	$effect(() => {
		const id = item.id;
		highlightsLoading = true;
		highlightsError = '';
		highlights = [];
		apiSdk
			.listHighlights({ path: { document_id: id } })
			.then(({ data }) => {
				if (data) {
					highlights = data.highlights;
				}
			})
			.catch(() => {
				highlightsError = 'Failed to load highlights.';
			})
			.finally(() => {
				highlightsLoading = false;
			});
	});

	$effect(() => {
		const id = item.id;
		note = null;
		noteLoading = true;
		noteError = '';
		editing = false;
		apiSdk
			.getDocumentEntryNote({ path: { document_id: id } })
			.then(({ data }) => {
				note = data ?? null;
			})
			.catch(() => {
				noteError = 'Failed to load note.';
			})
			.finally(() => {
				noteLoading = false;
			});
	});

	$effect(() => {
		if (editing && noteInputEl) noteInputEl.focus();
	});

	async function saveNote() {
		saving = true;
		noteError = '';
		const body = editText;
		try {
			const { data } = await apiSdk.upsertDocumentEntryNote({
				path: { document_id: item.id },
				body: { body }
			});
			note = data ?? null;
			editing = false;
		} catch {
			noteError = 'Failed to save note.';
		} finally {
			saving = false;
		}
	}

	async function deleteNote() {
		saving = true;
		noteError = '';
		try {
			await apiSdk.upsertDocumentEntryNote({
				path: { document_id: item.id },
				body: { body: '' }
			});
			note = null;
			editing = false;
		} catch {
			noteError = 'Failed to delete note.';
		} finally {
			saving = false;
		}
	}

	function startEdit() {
		editText = note?.body ?? '';
		editing = true;
	}

	async function deleteHighlight(highlightId: string) {
		try {
			await apiSdk.deleteHighlight({ path: { highlight_id: highlightId } });
			highlights = highlights.filter((h) => h.id !== highlightId);
		} catch {
			highlightsError = 'Failed to delete highlight.';
		}
	}

	function startHighlightNoteEdit(highlightId: string) {
		const hl = highlights.find((h) => h.id === highlightId);
		editingHighlightNoteId = highlightId;
		highlightNoteText = hl?.note?.body ?? '';
	}

	async function saveHighlightNote() {
		if (!editingHighlightNoteId) return;
		highlightNoteSaving = true;
		try {
			const { data: updatedNote } = await apiSdk.upsertNote({
				path: { highlight_id: editingHighlightNoteId },
				body: { body: highlightNoteText }
			});
			if (updatedNote) {
				highlights = highlights.map((h) =>
					h.id === editingHighlightNoteId ? { ...h, note: updatedNote } : h
				);
			}
			editingHighlightNoteId = null;
		} catch {
			highlightsError = 'Failed to save note.';
		} finally {
			highlightNoteSaving = false;
		}
	}

	async function deleteHighlightNote(highlightId: string) {
		try {
			await apiSdk.deleteNote({ path: { highlight_id: highlightId } });
			highlights = highlights.map((h) => (h.id === highlightId ? { ...h, note: null } : h));
			if (editingHighlightNoteId === highlightId) {
				editingHighlightNoteId = null;
			}
		} catch {
			highlightsError = 'Failed to delete note.';
		}
	}

	async function exportAllHighlights() {
		try {
			const { data } = await apiSdk.exportHighlights({
				path: { document_id: item.id }
			});
			if (data && browser) {
				const blob = new Blob([data as string], { type: 'text/markdown' });
				const url = URL.createObjectURL(blob);
				const a = document.createElement('a');
				a.href = url;
				a.download = `${item.title.replace(/[^a-z0-9]/gi, '_')}_highlights.md`;
				a.click();
				URL.revokeObjectURL(url);
			}
		} catch {
			highlightsError = 'Failed to export highlights.';
		}
	}

	function navigateToHighlight(highlightId: string) {
		const url = resolve('/(app)/reader/[documentId]', { documentId: item.id });
		// eslint-disable-next-line svelte/no-navigation-without-resolve -- resolve used above, query string appended
		goto(`${url}?highlight=${highlightId}`);
	}

	function copyToClipboard(text: string) {
		navigator.clipboard.writeText(text).catch(() => {});
	}

	function getLocationLabel(hl: HighlightWithNoteResponse): string {
		const loc = hl.locator;
		if (!loc) return '';
		if (loc.type === 'pdf') {
			return `p. ${loc.page}`;
		}
		if (loc.type === 'html') {
			const totalLen = item.word_count ? item.word_count * 5 : 10000;
			const pct = Math.round(((loc.start_offset ?? 0) / totalLen) * 100);
			return `${Math.min(pct, 100)}%`;
		}
		return '';
	}
</script>

<div class="notebook-content">
	<div class="content-header">
		<h2 class="item-title">{item.title}</h2>
		{#if item.domain}
			<p class="item-domain">{item.domain}</p>
		{/if}
	</div>

	<!-- Highlights -->
	<div class="section">
		<div class="section-header-row">
			<span class="section-heading">Highlights</span>
			<span class="section-count">{highlights.length} marked</span>
		</div>

		{#if highlightsLoading}
			<div class="note-loading">Loading...</div>
		{:else if highlightsError}
			<p class="note-error">{highlightsError}</p>
		{:else if highlights.length === 0}
			<div class="empty-state">No highlights yet. Open in Reader to start highlighting.</div>
		{:else}
			<div class="highlights-list">
				{#each highlights as hl (hl.id)}
					<div
						class="highlight-item"
						style:border-left-color={HIGHLIGHT_COLORS[hl.color] ?? HIGHLIGHT_COLORS.yellow}
						role="button"
						tabindex="0"
						onclick={() => navigateToHighlight(hl.id)}
						onkeydown={(e) => e.key === 'Enter' && navigateToHighlight(hl.id)}
					>
						<p class="highlight-text">{hl.text_content}</p>

						{#if hl.note}
							<p class="highlight-note">{hl.note.body}</p>
						{/if}

						{#if editingHighlightNoteId === hl.id}
							<div
								class="add-note-form"
								onclick={(e) => e.stopPropagation()}
								onkeydown={(e) => e.stopPropagation()}
								role="none"
							>
								<textarea
									bind:value={highlightNoteText}
									class="add-note-input"
									placeholder="Add a note..."
									rows={3}
									onkeydown={(e) => {
										if (e.key === 'Escape') {
											editingHighlightNoteId = null;
										}
									}}
								></textarea>
								<div class="add-note-actions">
									{#if hl.note}
										<button
											type="button"
											class="note-delete"
											onclick={() => deleteHighlightNote(hl.id)}
											disabled={highlightNoteSaving}
										>
											Delete
										</button>
									{/if}
									<button
										type="button"
										class="note-cancel"
										onclick={() => {
											editingHighlightNoteId = null;
										}}
										disabled={highlightNoteSaving}
									>
										Cancel
									</button>
									<button
										type="button"
										class="note-save"
										disabled={highlightNoteSaving || !highlightNoteText.trim()}
										onclick={saveHighlightNote}
									>
										{highlightNoteSaving ? 'Saving...' : 'Save'}
									</button>
								</div>
							</div>
						{/if}

						<div class="highlight-footer">
							<span class="hl-page">{getLocationLabel(hl)}</span>
							<button
								type="button"
								class="hl-action"
								aria-label="Add note to highlight"
								onclick={(e) => {
									e.stopPropagation();
									startHighlightNoteEdit(hl.id);
								}}
							>
								Note
							</button>
							<button
								type="button"
								class="hl-action"
								aria-label="Copy highlight"
								onclick={(e) => {
									e.stopPropagation();
									copyToClipboard(hl.text_content);
								}}
							>
								Copy
							</button>
							<button
								type="button"
								class="hl-action hl-action-delete"
								aria-label="Delete highlight"
								onclick={(e) => {
									e.stopPropagation();
									deleteHighlight(hl.id);
								}}
							>
								Delete
							</button>
						</div>
					</div>
				{/each}
			</div>

			<button type="button" class="export-btn" onclick={exportAllHighlights}>
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
					<polyline points="7 10 12 15 17 10" />
					<line x1="12" y1="15" x2="12" y2="3" />
				</svg>
				Export Highlights
			</button>
		{/if}
	</div>

	<!-- Note -->
	<div class="section">
		<span class="section-heading">Note</span>

		{#if noteLoading}
			<div class="note-loading">Loading...</div>
		{:else if editing}
			<div class="add-note-form">
				<textarea
					bind:this={noteInputEl}
					bind:value={editText}
					class="add-note-input"
					placeholder="Write a note..."
					rows={4}
					onkeydown={(e) => {
						if (e.key === 'Escape') {
							editing = false;
						}
					}}
				></textarea>
				{#if noteError}
					<p class="note-error">{noteError}</p>
				{/if}
				<div class="add-note-actions">
					{#if note}
						<button type="button" class="note-delete" onclick={deleteNote} disabled={saving}>
							Delete
						</button>
					{/if}
					<button
						type="button"
						class="note-cancel"
						onclick={() => {
							editing = false;
						}}
						disabled={saving}
					>
						Cancel
					</button>
					<button
						type="button"
						class="note-save"
						disabled={saving || !editText.trim()}
						onclick={saveNote}
					>
						{saving ? 'Saving...' : 'Save'}
					</button>
				</div>
			</div>
		{:else if note}
			<div
				class="note-card"
				role="button"
				tabindex="0"
				onclick={startEdit}
				onkeydown={(e) => e.key === 'Enter' && startEdit()}
			>
				<p class="note-text">{note.body}</p>
				<span class="note-timestamp">
					{new Date(note.updated_at).toLocaleDateString('en-US', {
						month: 'short',
						day: 'numeric'
					})}
				</span>
			</div>
		{:else}
			{#if noteError}
				<p class="note-error">{noteError}</p>
			{/if}
			<button type="button" class="add-note-row" onclick={startEdit}>
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<line x1="12" y1="5" x2="12" y2="19" />
					<line x1="5" y1="12" x2="19" y2="12" />
				</svg>
				Add a note
			</button>
		{/if}
	</div>
</div>

<style>
	.notebook-content {
		padding: 20px;
		display: flex;
		flex-direction: column;
		gap: 20px;
		flex: 1;
		overflow-y: auto;
	}

	.content-header {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.item-title {
		font-size: 20px;
		font-weight: 700;
		letter-spacing: -0.025em;
		line-height: 1.25;
		color: var(--text-primary);
		font-family: var(--font-sans);
		margin: 0;
	}

	.item-domain {
		font-size: 12.5px;
		font-weight: 400;
		color: var(--text-secondary);
		font-family: var(--font-sans);
		margin: 0;
	}

	.section {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.section-header-row {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
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

	.section-count {
		font-size: 11px;
		font-weight: 400;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
	}

	.highlights-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
		margin-top: 2px;
	}

	.highlight-item {
		padding: 10px 12px;
		border-left: 3px solid transparent;
		cursor: pointer;
		transition: background 120ms ease;
		border-radius: 0 8px 8px 0;
		background: var(--fill-hover);
		margin-bottom: 8px;
	}

	.highlight-item:hover {
		background: var(--fill-selected);
	}

	.highlight-text {
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.55;
		color: var(--text-primary);
		font-style: italic;
		font-family: var(--font-sans);
		margin: 0;
		display: -webkit-box;
		-webkit-line-clamp: 3;
		line-clamp: 3;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}

	.highlight-note {
		margin-top: 8px;
		font-size: 12px;
		font-weight: 400;
		font-style: italic;
		color: var(--text-secondary);
		letter-spacing: -0.005em;
		line-height: 1.4;
	}

	.highlight-footer {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-top: 4px;
		font-size: 11px;
		font-weight: 400;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
	}

	.hl-page {
		flex: 1;
	}

	.hl-action {
		background: none;
		border: none;
		padding: 0;
		font-size: 11px;
		font-family: var(--font-sans);
		color: var(--text-tertiary);
		cursor: pointer;
		transition: color 120ms ease;
	}

	.hl-action:hover {
		color: var(--accent);
	}

	.hl-action-delete:hover {
		color: var(--destructive);
	}

	.empty-state {
		font-size: 13px;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
		padding: 12px 0;
		line-height: 1.5;
	}

	.export-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		width: 100%;
		padding: 9px 16px;
		margin-top: 8px;
		border-radius: 980px;
		border: 1px solid var(--border-primary);
		background: transparent;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		letter-spacing: -0.01em;
		cursor: pointer;
		transition:
			background 120ms ease,
			border-color 120ms ease;
	}

	.export-btn:hover {
		background: var(--fill-hover);
		border-color: var(--border-secondary);
	}

	.export-btn :global(svg) {
		width: 14px;
		height: 14px;
	}

	.note-card {
		background: var(--fill-hover);
		border-radius: 10px;
		padding: 12px;
		cursor: pointer;
		transition: background 120ms ease;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.note-card:hover {
		background: var(--border-secondary);
	}

	.note-text {
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.55;
		color: var(--text-primary);
		font-family: var(--font-sans);
		margin: 0;
	}

	.note-timestamp {
		font-size: 11px;
		font-weight: 400;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
	}

	.note-loading {
		font-size: 13px;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
		padding: 4px 0;
	}

	.note-error {
		font-size: 12px;
		color: var(--destructive);
		font-family: var(--font-sans);
		margin: 0 0 4px;
	}

	.note-delete {
		font-size: 13px;
		font-family: var(--font-sans);
		color: var(--destructive);
		background: none;
		border: none;
		cursor: pointer;
		padding: 4px 10px;
		margin-right: auto;
	}

	.note-delete:hover:not(:disabled) {
		opacity: 0.8;
	}

	.note-delete:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.add-note-row {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 9px 12px;
		border-radius: 8px;
		border: 1px dashed var(--border-secondary);
		cursor: pointer;
		font-size: 13px;
		font-family: var(--font-sans);
		color: var(--text-tertiary);
		background: none;
		width: 100%;
		text-align: left;
		transition: all 120ms ease;
	}

	.add-note-row:hover {
		background: var(--fill-hover);
		color: var(--text-secondary);
	}

	.add-note-row :global(svg) {
		width: 14px;
		height: 14px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
		flex-shrink: 0;
	}

	.add-note-form {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.add-note-input {
		width: 100%;
		padding: 10px;
		border-radius: 8px;
		border: 1.5px solid var(--accent);
		background: var(--fill-hover);
		font-size: 13px;
		font-family: var(--font-sans);
		color: var(--text-primary);
		line-height: 1.55;
		resize: none;
		outline: none;
		box-sizing: border-box;
	}

	.add-note-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
	}

	.note-cancel {
		font-size: 13px;
		font-family: var(--font-sans);
		color: var(--text-secondary);
		background: none;
		border: none;
		cursor: pointer;
		padding: 4px 10px;
	}

	.note-cancel:hover {
		color: var(--text-primary);
	}

	.note-save {
		font-size: 13px;
		font-family: var(--font-sans);
		font-weight: 500;
		color: var(--text-on-color);
		background: var(--accent);
		border: none;
		border-radius: 6px;
		cursor: pointer;
		padding: 4px 14px;
		transition: opacity 120ms ease;
	}

	.note-save:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
</style>
