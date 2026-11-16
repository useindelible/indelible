<script lang="ts">
	import { onMount } from 'svelte';
	import * as apiSdk from '$lib/api';
	import type { DocumentNoteResponse } from '$lib/api';
	import { date, t } from '$lib/i18n';
	import NoteEditor from './NoteEditor.svelte';
	import { noteConflictOf } from './note-conflict';

	interface Props {
		documentId: string;
	}

	let { documentId }: Props = $props();

	// A loaded "no note" is the epoch, so a note written elsewhere since the load is a conflict.
	const EPOCH = '1970-01-01T00:00:00Z';
	const NOT_FOUND = 404;

	// The stored row stays the base even with an empty body: the server keeps the row it emptied.
	let note = $state<DocumentNoteResponse | null>(null);
	let loading = $state(true);
	let editing = $state(false);
	let editText = $state('');
	let saving = $state(false);
	let error = $state('');
	let conflict = $state<DocumentNoteResponse | null>(null);
	let refusedBody = $state('');

	const hasNote = $derived(!!note?.body);
	const base = $derived(note?.updated_at ?? EPOCH);

	onMount(async () => {
		try {
			const {
				data,
				error: failure,
				response
			} = await apiSdk.getDocumentEntryNote({
				path: { document_id: documentId }
			});
			if (data) note = data;
			else if (failure && response?.status !== NOT_FOUND) throw failure;
		} catch {
			error = $t('library_notebook_error_load_note');
		} finally {
			loading = false;
		}
	});

	async function writeNote(body: string, against: string) {
		saving = true;
		error = '';
		try {
			const { data, error: failure } = await apiSdk.upsertDocumentEntryNote({
				path: { document_id: documentId },
				body: { body, base_updated_at: against }
			});
			const refused = noteConflictOf(failure);
			if (refused) {
				conflict = refused.note;
				refusedBody = body;
				return;
			}
			if (failure || !data) throw failure;
			note = data;
			conflict = null;
			editing = false;
		} catch {
			error = $t(body ? 'library_notebook_error_save_note' : 'library_notebook_error_delete_note');
		} finally {
			saving = false;
		}
	}

	function keepMine() {
		if (conflict) writeNote(refusedBody === '' ? '' : editText, conflict.updated_at);
	}

	function useTheirs() {
		if (!conflict) return;
		editText = conflict.body;
		note = conflict;
		conflict = null;
	}

	function cancel() {
		if (conflict) note = conflict;
		conflict = null;
		editing = false;
	}

	function startEdit() {
		editText = note?.body ?? '';
		error = '';
		editing = true;
	}
</script>

{#if loading}
	<div class="note-loading">{$t('common_loading')}</div>
{:else if editing}
	<NoteEditor
		bind:value={editText}
		placeholder={$t('library_notebook_write_note')}
		rows={4}
		focus
		{saving}
		{error}
		onSave={() => writeNote(editText, base)}
		onCancel={cancel}
		onDelete={hasNote ? () => writeNote('', base) : undefined}
	>
		{#if conflict}
			<div class="note-conflict" role="alert">
				<p class="note-conflict-title">{$t('library_notebook_conflict_title')}</p>
				<p class="note-conflict-body">
					{conflict.body || $t('library_notebook_conflict_empty')}
				</p>
				<div class="conflict-actions">
					<button type="button" class="conflict-theirs" onclick={useTheirs} disabled={saving}>
						{$t('library_notebook_conflict_use_theirs')}
					</button>
					<button
						type="button"
						class="conflict-mine"
						onclick={keepMine}
						disabled={saving || (refusedBody !== '' && !editText.trim())}
					>
						{$t('library_notebook_conflict_keep_mine')}
					</button>
				</div>
			</div>
		{/if}
	</NoteEditor>
{:else if note && hasNote}
	<div
		class="note-card"
		role="button"
		tabindex="0"
		onclick={startEdit}
		onkeydown={(e) => e.key === 'Enter' && startEdit()}
	>
		<p class="note-text">{note.body}</p>
		<span class="note-timestamp">
			{$date(new Date(note.updated_at), { month: 'short', day: 'numeric' })}
		</span>
	</div>
{:else}
	{#if error}
		<p class="note-error">{error}</p>
	{/if}
	<button type="button" class="add-note-row" onclick={startEdit}>
		<svg viewBox="0 0 24 24" aria-hidden="true">
			<line x1="12" y1="5" x2="12" y2="19" />
			<line x1="5" y1="12" x2="19" y2="12" />
		</svg>
		{$t('library_notebook_add_note')}
	</button>
{/if}

<style>
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

	.note-conflict {
		background: var(--fill-warning);
		border: 1px solid var(--border-primary);
		border-radius: 8px;
		padding: 8px 10px;
	}

	.note-conflict-title {
		font-size: 12px;
		font-weight: 500;
		font-family: var(--font-sans);
		color: var(--text-primary);
		margin: 0 0 4px;
	}

	.note-conflict-body {
		font-size: 13px;
		font-family: var(--font-sans);
		color: var(--text-primary);
		white-space: pre-wrap;
		margin: 0 0 8px;
	}

	.conflict-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
	}

	.conflict-theirs,
	.conflict-mine {
		font-size: 13px;
		font-family: var(--font-sans);
		border: none;
		cursor: pointer;
		padding: 4px 10px;
	}

	.conflict-theirs {
		color: var(--text-secondary);
		background: none;
	}

	.conflict-theirs:hover {
		color: var(--text-primary);
	}

	.conflict-mine {
		font-weight: 500;
		color: var(--text-on-color);
		background: var(--accent);
		border-radius: 6px;
	}

	.conflict-theirs:disabled,
	.conflict-mine:disabled {
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
</style>
