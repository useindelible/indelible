import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { DocumentNoteResponse } from '$lib/api';

const getNote = vi.fn();
const upsertNote = vi.fn();

vi.mock('$lib/api', () => ({
	getDocumentEntryNote: (...args: unknown[]) => getNote(...args),
	upsertDocumentEntryNote: (...args: unknown[]) => upsertNote(...args)
}));

import DocumentNoteSection from '../../src/lib/components/library/DocumentNoteSection.svelte';

const T1 = '2026-11-12T22:20:24.000Z';
const T2 = '2026-11-12T22:22:32.000Z';
const EPOCH = '1970-01-01T00:00:00Z';

function stored(body: string, updatedAt: string): DocumentNoteResponse {
	return { id: 'inote_1', body, created_at: T1, updated_at: updatedAt };
}

function conflict(body: string, updatedAt: string) {
	return {
		data: undefined,
		error: {
			type: 'https://indelible.app/problems/note-conflict',
			title: 'Conflict',
			status: 409,
			detail: 'the note changed since base_updated_at',
			code: 'note_conflict',
			note: stored(body, updatedAt)
		}
	};
}

const written = (note: DocumentNoteResponse) => ({ data: note, error: undefined });
const sentBodies = () => upsertNote.mock.calls.map(([options]) => options.body);

async function openEditor(text: string) {
	await fireEvent.click(await screen.findByText(text));
	return screen.getByRole('textbox') as HTMLTextAreaElement;
}

async function type(input: HTMLTextAreaElement, value: string) {
	await fireEvent.input(input, { target: { value } });
}

describe('DocumentNoteSection', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		getNote.mockResolvedValue({ data: stored('first', T1), error: undefined });
	});

	it('shows the other version on a conflict and keeps mine on that version', async () => {
		upsertNote
			.mockResolvedValueOnce(conflict('theirs', T2))
			.mockResolvedValueOnce(written(stored('mine', '2026-11-12T22:30:00.000Z')));
		render(DocumentNoteSection, { documentId: 'doc_a' });
		await type(await openEditor('first'), 'mine');

		await fireEvent.click(screen.getByText('Save'));
		expect((await screen.findByRole('alert')).textContent).toContain('theirs');
		await fireEvent.click(screen.getByText('Keep mine'));

		await waitFor(() => expect(screen.queryByRole('textbox')).toBeNull());
		expect(sentBodies()).toEqual([
			{ body: 'mine', base_updated_at: T1 },
			{ body: 'mine', base_updated_at: T2 }
		]);
	});

	it('puts the other version in the editor on use theirs', async () => {
		upsertNote.mockResolvedValueOnce(conflict('theirs', T2));
		render(DocumentNoteSection, { documentId: 'doc_a' });
		const input = await openEditor('first');
		await type(input, 'mine');

		await fireEvent.click(screen.getByText('Save'));
		await fireEvent.click(await screen.findByText('Use theirs'));

		expect(input.value).toBe('theirs');
		expect(screen.queryByRole('alert')).toBeNull();
	});

	it('keeps the other version when the conflict is cancelled', async () => {
		upsertNote.mockResolvedValueOnce(conflict('theirs', T2));
		render(DocumentNoteSection, { documentId: 'doc_a' });
		await type(await openEditor('first'), 'mine');

		await fireEvent.click(screen.getByText('Save'));
		await screen.findByRole('alert');
		await fireEvent.click(screen.getByText('Cancel'));

		expect(await screen.findByText('theirs')).toBeTruthy();
	});

	it('bases the next note on the stamp of a deleted one', async () => {
		upsertNote
			.mockResolvedValueOnce(written(stored('', T2)))
			.mockResolvedValueOnce(written(stored('again', '2026-11-12T22:40:00.000Z')));
		render(DocumentNoteSection, { documentId: 'doc_a' });
		await openEditor('first');

		await fireEvent.click(screen.getByText('Delete'));
		await type(await openEditor('Add a note…'), 'again');
		await fireEvent.click(screen.getByText('Save'));

		await waitFor(() => expect(upsertNote).toHaveBeenCalledTimes(2));
		expect(sentBodies()[1]).toEqual({ body: 'again', base_updated_at: T2 });
	});

	it('writes a first note on the epoch', async () => {
		getNote.mockResolvedValue({ data: undefined, error: {}, response: { status: 404 } });
		upsertNote.mockResolvedValueOnce(written(stored('new', T1)));
		render(DocumentNoteSection, { documentId: 'doc_a' });

		await type(await openEditor('Add a note…'), 'new');
		await fireEvent.click(screen.getByText('Save'));

		await waitFor(() => expect(sentBodies()).toEqual([{ body: 'new', base_updated_at: EPOCH }]));
	});

	it('keeps the draft open when the server refuses for another reason', async () => {
		upsertNote.mockResolvedValueOnce({ data: undefined, error: { code: 'validation_error' } });
		render(DocumentNoteSection, { documentId: 'doc_a' });
		const input = await openEditor('first');
		await type(input, 'draft');

		await fireEvent.click(screen.getByText('Save'));

		expect(await screen.findByText('Failed to save note.')).toBeTruthy();
		expect(input.value).toBe('draft');
		expect(screen.queryByRole('alert')).toBeNull();
	});

	it('does not let keep mine turn an emptied draft into a deletion', async () => {
		upsertNote.mockResolvedValueOnce(conflict('theirs', T2));
		render(DocumentNoteSection, { documentId: 'doc_a' });
		const input = await openEditor('first');
		await type(input, 'mine');

		await fireEvent.click(screen.getByText('Save'));
		await screen.findByRole('alert');
		await type(input, '  ');

		expect((screen.getByText('Keep mine') as HTMLButtonElement).disabled).toBe(true);
	});

	it('reports a note that could not be loaded', async () => {
		getNote.mockResolvedValue({ data: undefined, error: {}, response: { status: 503 } });
		render(DocumentNoteSection, { documentId: 'doc_a' });

		expect(await screen.findByText('Failed to load note.')).toBeTruthy();
	});
});
