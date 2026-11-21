import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { DocumentListEntry } from '$lib/api';

const getNote = vi.fn();
const upsertNote = vi.fn();

vi.mock('$lib/api', () => ({
	listHighlights: () => Promise.resolve({ data: { highlights: [] } }),
	getDocumentEntryNote: (...args: unknown[]) => getNote(...args),
	upsertDocumentEntryNote: (...args: unknown[]) => upsertNote(...args)
}));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/paths', () => ({ resolve: (path: string) => path }));
vi.mock('$app/environment', () => ({ browser: true }));

import NotebookTab from '../../src/lib/components/library/NotebookTab.svelte';

const AT = '2026-11-12T22:20:24.000Z';

function entry(id: string): DocumentListEntry {
	return { id, title: `Title ${id}`, domain: null } as unknown as DocumentListEntry;
}

function noteOf(documentId: string, body: string) {
	return {
		data: { id: `note_${documentId}`, body, created_at: AT, updated_at: AT },
		error: undefined
	};
}

describe('NotebookTab document note', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		getNote.mockImplementation(({ path }) =>
			Promise.resolve(noteOf(path.document_id, `note of ${path.document_id}`))
		);
	});

	it('drops a conflict that answers for the item the user left', async () => {
		let answerA: (value: unknown) => void = () => {};
		upsertNote.mockReturnValueOnce(new Promise((resolve) => (answerA = resolve)));
		const view = render(NotebookTab, { item: entry('doc_a') });
		await fireEvent.click(await screen.findByText('note of doc_a'));
		await fireEvent.click(screen.getByText('Save'));

		await view.rerender({ item: entry('doc_b') });
		await fireEvent.click(await screen.findByText('note of doc_b'));
		answerA({
			data: undefined,
			error: {
				code: 'note_conflict',
				note: { id: 'note_doc_a', body: 'theirs of doc_a', created_at: AT, updated_at: AT }
			}
		});

		await waitFor(() => expect(upsertNote).toHaveBeenCalledTimes(1));
		await new Promise((settle) => setTimeout(settle, 0));
		expect(screen.queryByText('theirs of doc_a')).toBeNull();
		expect((screen.getByRole('textbox') as HTMLTextAreaElement).value).toBe('note of doc_b');
	});
});
